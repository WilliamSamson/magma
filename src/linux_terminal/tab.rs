use gtk::{glib, prelude::*, Box as GtkBox, Label, Orientation, Paned};

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use super::{
    persist::{PaneFocus, TabSnapshot},
    profile::{profile, ProfileId},
    session::SessionView,
    settings::Settings,
};

pub(super) struct TabView {
    root: GtkBox,
    title_label: Label,
    base_title: String,
    left: SessionView,
    right: Option<SessionView>,
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

        let settings_ref = settings.borrow();
        let left = SessionView::new(snapshot.profile, snapshot.left_cwd.as_deref(), &settings_ref);
        root.append(left.root());
        // Rc<Cell<PaneFocus>> tracks the last active pane across GTK focus callbacks without borrow overhead.
        let active_pane = Rc::new(Cell::new(snapshot.active_pane));
        bind_focus_tracking(&left, &active_pane, PaneFocus::Left);

        let mut right = None;
        let mut split_view = None;
        if let Some(cwd) = snapshot.right_cwd.as_deref() {
            let right_session = SessionView::new(snapshot.profile, Some(cwd), &settings_ref);
            bind_focus_tracking(&right_session, &active_pane, PaneFocus::Right);
            let paned = build_split_view(&left, &right_session, snapshot.split_position);
            root.remove(left.root());
            root.append(&paned);
            split_view = Some(paned);
            right = Some(right_session);
        }
        drop(settings_ref);

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
            left_cwd: self.left.current_cwd(),
            right_cwd: self.right.as_ref().and_then(SessionView::current_cwd),
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

        let cwd = self.left.current_cwd();
        let settings_ref = self.settings.borrow();
        let right = SessionView::new(self.profile_id, cwd.as_deref(), &settings_ref);
        bind_focus_tracking(&right, &self.active_pane, PaneFocus::Right);
        let split_view = build_split_view(&self.left, &right, None);
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

    pub(super) fn restore_focus(&self) {
        if self.right.is_some() && self.active_pane.get() == PaneFocus::Right {
            let _ = self.focus_right_pane();
            return;
        }
        self.left.focus_terminal();
    }

    pub(super) fn apply_settings(&self, settings: &Settings) {
        self.left.apply_settings(settings, self.profile_id);
        if let Some(right) = &self.right {
            right.apply_settings(settings, self.profile_id);
        }
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

fn build_split_view(left: &SessionView, right: &SessionView, split_position: Option<i32>) -> Paned {
    let split_view = Paned::new(Orientation::Horizontal);
    split_view.add_css_class("obsidian-split-pane");
    split_view.set_hexpand(true);
    split_view.set_vexpand(true);
    split_view.set_wide_handle(true);
    split_view.set_shrink_start_child(false);
    split_view.set_shrink_end_child(false);
    split_view.set_resize_start_child(true);
    split_view.set_resize_end_child(true);
    split_view.set_start_child(Some(left.root()));
    split_view.set_end_child(Some(right.root()));

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

fn bind_focus_tracking(session: &SessionView, active_pane: &Rc<Cell<PaneFocus>>, pane: PaneFocus) {
    let active_pane_ref = active_pane.clone();
    session.connect_focus_enter(move || {
        active_pane_ref.set(pane);
    });
}
