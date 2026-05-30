use gtk::{Box as GtkBox, Label, Orientation, Paned, glib, prelude::*};

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use super::super::{
    persist::{PaneFocus, PaneSnapshot, TabSnapshot},
    settings::Settings,
};
use super::{
    mux::MuxPaneView,
    profile::{ProfileId, profile},
    scaled_spacing,
};

pub(crate) struct TabView {
    root: GtkBox,
    title_label: Label,
    base_title: String,
    left: MuxPaneView,
    right: Option<MuxPaneView>,
    split_view: Option<Paned>,
    active_pane: Rc<Cell<PaneFocus>>,
    profile_id: ProfileId,
    settings: Rc<RefCell<Settings>>,
    session_bar_host: GtkBox, // Stored to dynamically mount terminal session tabs into the top workspace bar.
}

impl TabView {
    pub(crate) fn new(
        snapshot: TabSnapshot,
        settings: Rc<RefCell<Settings>>, // settings Rc wrapper is cloned to share settings across components.
        session_bar_host: GtkBox, // session_bar_host GtkBox is passed to dynamically mount session tabs at the top.
    ) -> Self {
        let settings_ref = settings.borrow();
        let spacing = scaled_spacing(12, &settings_ref);
        drop(settings_ref);

        let root = GtkBox::new(Orientation::Horizontal, spacing);
        root.set_hexpand(true);
        root.set_vexpand(true);

        // Rc<Cell<PaneFocus>> tracks the active split side across session-focus changes without borrow overhead.
        let active_pane = Rc::new(Cell::new(snapshot.active_pane));
        let left_snapshot = snapshot.left_pane.unwrap_or_default().normalized();
        let left = MuxPaneView::new(
            left_snapshot,
            snapshot.profile,
            settings.clone(), // settings clone is needed to share RefCell ownership with MuxPaneView.
            active_pane.clone(), // active_pane clone is needed to share the pane focus Cell with the left pane.
            PaneFocus::Left,
            Some(session_bar_host.clone()), // session_bar_host clone is needed to pass a GObject reference to the left pane.
        );
        root.append(left.root());

        let mut right = None;
        let mut split_view = None;
        if let Some(right_snapshot) = snapshot.right_pane.map(PaneSnapshot::normalized) {
            let right_pane = MuxPaneView::new(
                right_snapshot,
                snapshot.profile,
                settings.clone(), // settings clone is needed to share RefCell ownership with right pane.
                active_pane.clone(), // active_pane clone is needed to share the pane focus Cell with the right pane.
                PaneFocus::Right,
                Some(session_bar_host.clone()), // session_bar_host clone is needed to pass a GObject reference to the right pane.
            );
            let paned = build_split_view(left.root(), right_pane.root(), snapshot.split_position);
            root.remove(left.root());
            root.append(&paned);
            split_view = Some(paned);
            right = Some(right_pane);
        }

        let base_title = stored_base_title(&snapshot.title, snapshot.profile);
        let title_label = Label::new(Some(&display_title(&base_title, snapshot.profile)));
        title_label.add_css_class("magma-tab-label");

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
            session_bar_host,
        }
    }

    pub(crate) fn root(&self) -> &GtkBox {
        &self.root
    }

    pub(crate) fn title_label(&self) -> &Label {
        &self.title_label
    }

    pub(crate) fn base_title(&self) -> &str {
        &self.base_title
    }

    pub(crate) fn profile_id(&self) -> ProfileId {
        self.profile_id
    }

    pub(crate) fn to_snapshot(&self) -> TabSnapshot {
        TabSnapshot {
            title: self.base_title.clone(),
            profile: self.profile_id,
            left_pane: Some(self.left.to_snapshot()),
            right_pane: self.right.as_ref().map(MuxPaneView::to_snapshot),
            split_position: self.split_view.as_ref().map(Paned::position),
            active_pane: self.active_pane.get(),
        }
    }

    pub(crate) fn cycle_profile(&mut self, next_profile: ProfileId) {
        self.profile_id = next_profile;
        self.left.apply_profile(next_profile);
        if let Some(right) = &self.right {
            right.apply_profile(next_profile);
        }
        self.sync_title_label();
    }

    pub(crate) fn rename(&mut self, title: &str) {
        let trimmed = title.trim();
        if trimmed.is_empty() {
            return;
        }

        self.base_title = trimmed.to_string();
        self.sync_title_label();
    }

    pub(crate) fn toggle_split(&mut self) {
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
            self.settings.clone(), // settings clone is needed to share the RefCell reference with the newly split right pane.
            self.active_pane.clone(), // active_pane clone is needed to share the pane focus Cell with the new right pane.
            PaneFocus::Right,
            Some(self.session_bar_host.clone()), // session_bar_host clone is needed to pass a GObject reference to the new split pane.
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

    pub(crate) fn focus_left_pane(&self) -> bool {
        if self.right.is_none() {
            return false;
        }
        self.active_pane.set(PaneFocus::Left);
        self.left.focus_terminal();
        true
    }

    pub(crate) fn focus_right_pane(&self) -> bool {
        let Some(right) = &self.right else {
            return false;
        };
        self.active_pane.set(PaneFocus::Right);
        right.focus_terminal();
        true
    }

    pub(crate) fn focus_next_session(&self) -> bool {
        self.active_mux_pane().focus_next_session()
    }

    pub(crate) fn focus_previous_session(&self) -> bool {
        self.active_mux_pane().focus_previous_session()
    }

    pub(crate) fn new_mux_session(&self) {
        self.active_mux_pane().new_session();
    }

    pub(crate) fn close_active_session(&self) -> bool {
        self.active_mux_pane().close_active_session()
    }

    pub(crate) fn focus_session(&self, index: usize) -> bool {
        self.active_mux_pane().focus_session(index)
    }

    pub(crate) fn restore_focus(&self) {
        if self.right.is_some() && self.active_pane.get() == PaneFocus::Right {
            if let Some(right) = &self.right {
                right.focus_terminal();
            }
            return;
        }
        self.left.focus_terminal();
    }

    pub(crate) fn apply_settings(&self, settings: &Settings) {
        self.left.apply_settings(settings);
        if let Some(right) = &self.right {
            right.apply_settings(settings);
        }
    }

    pub(crate) fn current_cwd(&self) -> Option<String> {
        self.active_mux_pane().current_cwd()
    }

    pub(crate) fn current_terminal(&self) -> Option<vte4::Terminal> {
        self.active_mux_pane().current_terminal()
    }

    fn active_mux_pane(&self) -> &MuxPaneView {
        if self.active_pane.get() == PaneFocus::Right {
            if let Some(right) = &self.right {
                return right;
            }
        }
        &self.left
    }

    pub(crate) fn mount_active_session_bar(&self) {
        self.active_mux_pane()
            .mount_session_bar(&self.session_bar_host);
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
    title.strip_suffix(&suffix).unwrap_or(title).to_string()
}

fn build_split_view(left: &GtkBox, right: &GtkBox, split_position: Option<i32>) -> Paned {
    let split_view = Paned::new(Orientation::Horizontal);
    split_view.add_css_class("magma-split-pane");
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
