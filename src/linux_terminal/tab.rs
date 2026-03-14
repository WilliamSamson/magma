use gtk::{glib, prelude::*, Box as GtkBox, Label, Orientation, Paned};

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use super::{
    mux::MuxPaneView,
    persist::{PaneFocus, PaneSnapshot, TabSnapshot},
    profile::{profile, ProfileId},
    settings::Settings,
};

pub(super) struct TabView {
    root: GtkBox,
    title_label: Label,
    base_title: String,
    left: MuxPaneView,
    right: Option<MuxPaneView>,
    split_view: Option<Paned>,
    active_pane: Rc<Cell<PaneFocus>>,
    profile_id: ProfileId,
    settings: Rc<RefCell<Settings>>,
}

impl TabView {
    pub(super) fn new(snapshot: TabSnapshot, settings: Rc<RefCell<Settings>>) -> Self {
        let root = GtkBox::new(Orientation::Horizontal, 12);
        root.set_hexpand(true);
        root.set_vexpand(true);

        // Rc<Cell<PaneFocus>> tracks the active split side across session-focus changes without borrow overhead.
        let active_pane = Rc::new(Cell::new(snapshot.active_pane));
        let left_snapshot = snapshot.left_pane.unwrap_or_default().normalized();
        let left = MuxPaneView::new(
            left_snapshot,
            snapshot.profile,
            settings.clone(),
            active_pane.clone(),
            PaneFocus::Left,
        );
        root.append(left.root());

        let mut right = None;
        let mut split_view = None;
        if let Some(right_snapshot) = snapshot.right_pane.map(PaneSnapshot::normalized) {
            let right_pane = MuxPaneView::new(
                right_snapshot,
                snapshot.profile,
                settings.clone(),
                active_pane.clone(),
                PaneFocus::Right,
            );
            let paned = build_split_view(left.root(), right_pane.root(), snapshot.split_position);
            root.remove(left.root());
            root.append(&paned);
            split_view = Some(paned);
            right = Some(right_pane);
        }

        let base_title = stored_base_title(&snapshot.title, snapshot.profile);
        let title_label = Label::new(Some(&display_title(&base_title, snapshot.profile)));
        title_label.add_css_class("obsidian-tab-label");

        Self {
            root,
            title_label,
            base_title,
            left,
            right,
            split_view,
            active_pane,
            profile_id: snapshot.profile,
            settings,
        }
    }

    pub(super) fn root(&self) -> &GtkBox {
        &self.root
    }

    pub(super) fn title_label(&self) -> &Label {
        &self.title_label
    }

    pub(super) fn base_title(&self) -> &str {
        &self.base_title
    }

    pub(super) fn profile_id(&self) -> ProfileId {
        self.profile_id
    }

    pub(super) fn to_snapshot(&self) -> TabSnapshot {
        TabSnapshot {
            title: self.base_title.clone(),
            profile: self.profile_id,
            left_pane: Some(self.left.to_snapshot()),
            right_pane: self.right.as_ref().map(MuxPaneView::to_snapshot),
            split_position: self.split_view.as_ref().map(Paned::position),
            active_pane: self.active_pane.get(),
        }
    }

    pub(super) fn cycle_profile(&mut self, next_profile: ProfileId) {
        self.profile_id = next_profile;
        self.left.apply_profile(next_profile);
        if let Some(right) = &self.right {
            right.apply_profile(next_profile);
        }
        self.sync_title_label();
    }

    pub(super) fn rename(&mut self, title: &str) {
        let trimmed = title.trim();
        if trimmed.is_empty() {
            return;
        }

        self.base_title = trimmed.to_string();
        self.sync_title_label();
    }

    pub(super) fn toggle_split(&mut self) {
        if self.right.is_some() {
            if let Some(split_view) = self.split_view.take() {
                self.root.remove(&split_view);
            }
            self.right.take();
            self.active_pane.set(PaneFocus::Left);
            self.root.append(self.left.root());
            self.left.focus_terminal();
            return;
        }

        let right = MuxPaneView::new(
            PaneSnapshot::from_cwd(self.left.current_cwd()),
            self.profile_id,
            self.settings.clone(),
            self.active_pane.clone(),
            PaneFocus::Right,
        );
        let split_view = build_split_view(self.left.root(), right.root(), None);
        self.root.remove(self.left.root());
        self.root.append(&split_view);
        self.split_view = Some(split_view);
        self.right = Some(right);
        self.active_pane.set(PaneFocus::Right);
        if let Some(right) = &self.right {
            right.focus_terminal();
        }
    }

    pub(super) fn focus_left_pane(&self) -> bool {
        if self.right.is_none() {
            return false;
        }
        self.active_pane.set(PaneFocus::Left);
        self.left.focus_terminal();
        true
    }

    pub(super) fn focus_right_pane(&self) -> bool {
        let Some(right) = &self.right else {
            return false;
        };
        self.active_pane.set(PaneFocus::Right);
        right.focus_terminal();
        true
    }

    pub(super) fn focus_next_session(&self) -> bool {
        self.active_mux_pane().focus_next_session()
    }

    pub(super) fn focus_previous_session(&self) -> bool {
        self.active_mux_pane().focus_previous_session()
    }

    pub(super) fn new_mux_session(&self) {
        self.active_mux_pane().new_session();
    }

    pub(super) fn close_active_session(&self) -> bool {
        self.active_mux_pane().close_active_session()
    }

    pub(super) fn focus_session(&self, index: usize) -> bool {
        self.active_mux_pane().focus_session(index)
    }

    pub(super) fn restore_focus(&self) {
        if self.right.is_some() && self.active_pane.get() == PaneFocus::Right {
            if let Some(right) = &self.right {
                right.focus_terminal();
            }
            return;
        }
        self.left.focus_terminal();
    }

    pub(super) fn apply_settings(&self, settings: &Settings) {
        self.left.apply_settings(settings);
        if let Some(right) = &self.right {
            right.apply_settings(settings);
        }
    }

    pub(super) fn current_cwd(&self) -> Option<String> {
        self.active_mux_pane().current_cwd()
    }

    fn active_mux_pane(&self) -> &MuxPaneView {
        if self.active_pane.get() == PaneFocus::Right {
            if let Some(right) = &self.right {
                return right;
            }
        }
        &self.left
    }

    fn sync_title_label(&self) {
        self.title_label
            .set_text(&display_title(&self.base_title, self.profile_id));
    }
}

fn display_title(base_title: &str, profile_id: ProfileId) -> String {
    if profile_id == ProfileId::Default {
        return base_title.to_string();
    }

    format!("{base_title} ({})", profile(profile_id).label)
}

fn stored_base_title(title: &str, profile_id: ProfileId) -> String {
    if profile_id == ProfileId::Default {
        return title.to_string();
    }

    let suffix = format!(" ({})", profile(profile_id).label);
    title
        .strip_suffix(&suffix)
        .unwrap_or(title)
        .to_string()
}

fn build_split_view(left: &GtkBox, right: &GtkBox, split_position: Option<i32>) -> Paned {
    let split_view = Paned::new(Orientation::Horizontal);
    split_view.add_css_class("obsidian-split-pane");
    split_view.set_hexpand(true);
    split_view.set_vexpand(true);
    split_view.set_wide_handle(true);
    split_view.set_shrink_start_child(false);
    split_view.set_shrink_end_child(false);
    split_view.set_resize_start_child(true);
    split_view.set_resize_end_child(true);
    split_view.set_start_child(Some(left));
    split_view.set_end_child(Some(right));

    let split_view_ref = split_view.clone();
    glib::idle_add_local_once(move || {
        if let Some(position) = split_position.filter(|position| *position > 0) {
            split_view_ref.set_position(position);
            return;
        }

        let width = split_view_ref.allocation().width();
        if width > 0 {
            split_view_ref.set_position(width / 2);
        }
    });

    split_view
}
