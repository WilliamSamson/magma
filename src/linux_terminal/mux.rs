use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use gtk::{prelude::*, Box as GtkBox, Button, Orientation, Stack, StackTransitionType};

use super::{
    persist::{PaneFocus, PaneSnapshot, SessionSnapshot},
    profile::ProfileId,
    session::SessionView,
    settings::Settings,
};

struct SessionEntry {
    stack_name: String,
    // Rc<SessionView> lets the stack, actions, and focus callbacks share one live session without ownership fights.
    view: Rc<SessionView>,
}

struct MuxState {
    sessions: RefCell<Vec<SessionEntry>>,
    active_index: Cell<usize>,
    next_session_id: Cell<u32>,
    profile_id: Cell<ProfileId>,
}

struct FocusBinding {
    active_pane: Rc<Cell<PaneFocus>>,
    pane: PaneFocus,
}

pub(super) struct MuxPaneView {
    root: GtkBox,
    bar: GtkBox,
    stack: Stack,
    state: Rc<MuxState>,
    settings: Rc<RefCell<Settings>>,
    focus: Rc<FocusBinding>,
}

impl MuxPaneView {
    pub(super) fn new(
        snapshot: PaneSnapshot,
        profile_id: ProfileId,
        settings: Rc<RefCell<Settings>>,
        active_pane: Rc<Cell<PaneFocus>>,
        pane: PaneFocus,
    ) -> Self {
        let root = GtkBox::new(Orientation::Vertical, 8);
        root.add_css_class("magma-mux-root");
        root.set_hexpand(true);
        root.set_vexpand(true);

        let bar = GtkBox::new(Orientation::Horizontal, 6);
        bar.add_css_class("magma-mux-bar");
        bar.set_hexpand(true);

        let stack = Stack::new();
        stack.set_hexpand(true);
        stack.set_vexpand(true);
        stack.set_transition_type(StackTransitionType::Crossfade);
        stack.set_transition_duration(120);

        root.append(&bar);
        root.append(&stack);

        // Rc<MuxState> keeps pane-local multiplexer state shared across GTK callbacks on the main thread.
        let state = Rc::new(MuxState {
            sessions: RefCell::new(Vec::new()),
            active_index: Cell::new(0),
            next_session_id: Cell::new(0),
            profile_id: Cell::new(profile_id),
        });
        let focus = Rc::new(FocusBinding { active_pane, pane });

        let pane_view = Self {
            root,
            bar,
            stack,
            state,
            settings,
            focus,
        };

        let snapshot = snapshot.normalized();
        for session in &snapshot.sessions {
            append_session(
                &pane_view.state,
                &pane_view.stack,
                &pane_view.settings,
                &pane_view.focus,
                session,
            );
        }
        let _ = switch_to(
            &pane_view.state,
            &pane_view.stack,
            &pane_view.bar,
            &pane_view.settings,
            &pane_view.focus,
            snapshot.active_session,
        );
        pane_view
    }

    pub(super) fn root(&self) -> &GtkBox {
        &self.root
    }

    pub(super) fn to_snapshot(&self) -> PaneSnapshot {
        let sessions = self
            .state
            .sessions
            .borrow()
            .iter()
            .map(|entry| entry.view.to_snapshot())
            .collect();

        PaneSnapshot {
            sessions,
            active_session: self.state.active_index.get(),
        }
    }

    pub(super) fn current_cwd(&self) -> Option<String> {
        current_session(&self.state).and_then(|session| session.current_cwd())
    }

    pub(super) fn focus_terminal(&self) {
        if let Some(session) = current_session(&self.state) {
            self.focus.active_pane.set(self.focus.pane);
            session.focus_terminal();
        }
    }

    pub(super) fn apply_profile(&self, profile_id: ProfileId) {
        self.state.profile_id.set(profile_id);
        for entry in self.state.sessions.borrow().iter() {
            entry.view.apply_profile(profile_id);
        }
    }

    pub(super) fn apply_settings(&self, settings: &Settings) {
        let profile_id = self.state.profile_id.get();
        for entry in self.state.sessions.borrow().iter() {
            entry.view.apply_settings(settings, profile_id);
        }
    }

    pub(super) fn new_session(&self) {
        let cwd = self.current_cwd();
        let snapshot = SessionSnapshot::new(cwd);
        let index = append_session(
            &self.state,
            &self.stack,
            &self.settings,
            &self.focus,
            &snapshot,
        );
        let _ = switch_to(
            &self.state,
            &self.stack,
            &self.bar,
            &self.settings,
            &self.focus,
            index,
        );
    }

    pub(super) fn close_active_session(&self) -> bool {
        close_active_session(
            &self.state,
            &self.stack,
            &self.bar,
            &self.settings,
            &self.focus,
        )
    }

    pub(super) fn focus_next_session(&self) -> bool {
        let session_count = self.state.sessions.borrow().len();
        if session_count <= 1 {
            return false;
        }

        let next = (self.state.active_index.get() + 1) % session_count;
        switch_to(
            &self.state,
            &self.stack,
            &self.bar,
            &self.settings,
            &self.focus,
            next,
        )
    }

    pub(super) fn focus_previous_session(&self) -> bool {
        let session_count = self.state.sessions.borrow().len();
        if session_count <= 1 {
            return false;
        }

        let current = self.state.active_index.get();
        let previous = if current == 0 {
            session_count - 1
        } else {
            current - 1
        };
        switch_to(
            &self.state,
            &self.stack,
            &self.bar,
            &self.settings,
            &self.focus,
            previous,
        )
    }

    pub(super) fn focus_session(&self, index: usize) -> bool {
        switch_to(
            &self.state,
            &self.stack,
            &self.bar,
            &self.settings,
            &self.focus,
            index,
        )
    }
}

fn append_session(
    state: &Rc<MuxState>,
    stack: &Stack,
    settings: &Rc<RefCell<Settings>>,
    focus: &Rc<FocusBinding>,
    snapshot: &SessionSnapshot,
) -> usize {
    let session = Rc::new(SessionView::new(
        state.profile_id.get(),
        snapshot,
        settings.clone(),
    ));

    let focus_ref = focus.clone();
    session.connect_focus_enter(move || {
        focus_ref.active_pane.set(focus_ref.pane);
    });

    let session_id = state.next_session_id.get();
    state.next_session_id.set(session_id + 1);
    let stack_name = format!("mux-session-{session_id}");
    stack.add_named(session.root(), Some(&stack_name));
    state.sessions.borrow_mut().push(SessionEntry {
        stack_name,
        view: session,
    });

    state.sessions.borrow().len().saturating_sub(1)
}

fn switch_to(
    state: &Rc<MuxState>,
    stack: &Stack,
    bar: &GtkBox,
    settings: &Rc<RefCell<Settings>>,
    focus: &Rc<FocusBinding>,
    index: usize,
) -> bool {
    let (stack_name, session) = {
        let sessions = state.sessions.borrow();
        let Some(entry) = sessions.get(index) else {
            return false;
        };
        (entry.stack_name.clone(), entry.view.clone())
    };

    state.active_index.set(index);
    stack.set_visible_child_name(&stack_name);
    focus.active_pane.set(focus.pane);
    session.focus_terminal();
    rebuild_bar(bar, stack, state, settings, focus);
    true
}

fn close_active_session(
    state: &Rc<MuxState>,
    stack: &Stack,
    bar: &GtkBox,
    settings: &Rc<RefCell<Settings>>,
    focus: &Rc<FocusBinding>,
) -> bool {
    let session_count = state.sessions.borrow().len();
    if session_count <= 1 {
        return false;
    }

    let index = state.active_index.get().min(session_count.saturating_sub(1));
    let removed = state.sessions.borrow_mut().remove(index);
    stack.remove(removed.view.root());

    let next_index = index.min(session_count.saturating_sub(2));
    switch_to(state, stack, bar, settings, focus, next_index)
}

fn rebuild_bar(
    bar: &GtkBox,
    stack: &Stack,
    state: &Rc<MuxState>,
    settings: &Rc<RefCell<Settings>>,
    focus: &Rc<FocusBinding>,
) {
    clear_children(bar);

    let session_count = state.sessions.borrow().len();
    let current = state.active_index.get();

    for index in 0..session_count {
        let button = Button::with_label(&format!("{:02}", index + 1));
        button.add_css_class("magma-mux-session");
        button.set_focus_on_click(false);
        if index == current {
            button.add_css_class("active");
        }

        let bar_ref = bar.clone();
        let stack = stack.clone();
        let state = state.clone();
        let settings = settings.clone();
        let focus = focus.clone();
        button.connect_clicked(move |_| {
            let _ = switch_to(&state, &stack, &bar_ref, &settings, &focus, index);
        });
        bar.append(&button);
    }

    let spacer = GtkBox::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    bar.append(&spacer);

    let add_button = Button::builder()
        .icon_name("list-add-symbolic")
        .css_classes(["magma-mux-action"])
        .tooltip_text("New session")
        .build();
    let close_button = Button::builder()
        .icon_name("window-close-symbolic")
        .css_classes(["magma-mux-action"])
        .tooltip_text("Close session")
        .sensitive(session_count > 1)
        .build();

    let bar_ref = bar.clone();
    let stack_ref = stack.clone();
    let state_ref = state.clone();
    let settings_ref = settings.clone();
    let focus_ref = focus.clone();
    add_button.connect_clicked(move |_| {
        let cwd = current_session(&state_ref).and_then(|session| session.current_cwd());
        let snapshot = SessionSnapshot::new(cwd);
        let index = append_session(
            &state_ref,
            &stack_ref,
            &settings_ref,
            &focus_ref,
            &snapshot,
        );
        let _ = switch_to(&state_ref, &stack_ref, &bar_ref, &settings_ref, &focus_ref, index);
    });

    let bar_ref = bar.clone();
    let stack_ref = stack.clone();
    let state_ref = state.clone();
    let settings_ref = settings.clone();
    let focus_ref = focus.clone();
    close_button.connect_clicked(move |_| {
        let _ = close_active_session(&state_ref, &stack_ref, &bar_ref, &settings_ref, &focus_ref);
    });

    bar.append(&add_button);
    bar.append(&close_button);
}

fn current_session(state: &Rc<MuxState>) -> Option<Rc<SessionView>> {
    let sessions = state.sessions.borrow();
    let index = state.active_index.get();
    sessions.get(index).map(|entry| entry.view.clone())
}

fn clear_children(container: &GtkBox) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}
