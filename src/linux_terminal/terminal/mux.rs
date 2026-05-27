use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use gtk::{
    prelude::*, Box as GtkBox, Button, Orientation, PolicyType, ScrolledWindow, Stack,
    StackTransitionType,
};

use super::{
    profile::ProfileId,
    scaled_spacing,
    session::SessionView,
};
use super::super::{
    persist::{PaneFocus, PaneSnapshot, SessionSnapshot},
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
    session_bar_host: Option<GtkBox>, // Option<GtkBox> clone is cheap (GObject reference count) and lets callbacks access the top bar host.
}

struct FocusBinding {
    active_pane: Rc<Cell<PaneFocus>>,
    pane: PaneFocus,
}

pub(crate) struct MuxPaneView {
    root: GtkBox,
    bar: GtkBox,
    buttons_box: GtkBox,
    actions_box: GtkBox,
    stack: Stack,
    state: Rc<MuxState>,
    settings: Rc<RefCell<Settings>>,
    focus: Rc<FocusBinding>,
}

#[derive(Clone)]
struct MuxBarContext {
    bar: GtkBox,
    buttons_box: GtkBox,
    actions_box: GtkBox,
    stack: Stack,
    state: Rc<MuxState>,
    settings: Rc<RefCell<Settings>>,
    focus: Rc<FocusBinding>,
}

impl MuxPaneView {
    pub(crate) fn new(
        snapshot: PaneSnapshot,
        profile_id: ProfileId,
        settings: Rc<RefCell<Settings>>,
        active_pane: Rc<Cell<PaneFocus>>,
        pane: PaneFocus,
        session_bar_host: Option<GtkBox>,
    ) -> Self {
        let settings_ref = settings.borrow();
        let root_spacing = scaled_spacing(8, &settings_ref);
        let bar_spacing = scaled_spacing(6, &settings_ref);
        drop(settings_ref);

        let root = GtkBox::new(Orientation::Vertical, root_spacing);
        root.add_css_class("magma-mux-root");
        root.set_hexpand(true);
        root.set_vexpand(true);

        let bar = GtkBox::new(Orientation::Horizontal, bar_spacing);
        bar.add_css_class("magma-mux-bar");
        bar.set_hexpand(true);

        let buttons_box = GtkBox::new(Orientation::Horizontal, bar_spacing);
        buttons_box.add_css_class("magma-mux-buttons");

        let session_scroll = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Automatic)
            .vscrollbar_policy(PolicyType::Never)
            .css_classes(["magma-session-bar-scroller"])
            .propagate_natural_width(true)
            .max_content_width(240)
            .build();
        session_scroll.set_hexpand(false);
        session_scroll.set_vexpand(false);
        session_scroll.set_propagate_natural_height(true);
        session_scroll.set_child(Some(&buttons_box));

        let actions_box = GtkBox::new(Orientation::Horizontal, bar_spacing);
        actions_box.add_css_class("magma-mux-actions");

        bar.append(&session_scroll);
        bar.append(&actions_box);

        let stack = Stack::new();
        stack.set_hexpand(true);
        stack.set_vexpand(true);
        stack.set_transition_type(StackTransitionType::Crossfade);
        stack.set_transition_duration(120);

        // We do NOT append bar locally, as it is dynamically mounted in the top tab bar row.
        root.append(&stack);

        // Rc<MuxState> keeps pane-local multiplexer state shared across GTK callbacks on the main thread.
        let state = Rc::new(MuxState {
            sessions: RefCell::new(Vec::new()),
            active_index: Cell::new(0),
            next_session_id: Cell::new(0),
            profile_id: Cell::new(profile_id),
            session_bar_host, // Store the top-level session bar host GtkBox option in shared state.
        });
        let focus = Rc::new(FocusBinding { active_pane, pane });

        let pane_view = Self {
            root,
            bar,
            buttons_box,
            actions_box,
            stack,
            state,
            settings,
            focus,
        };

        let ctx = pane_view.bar_context();
        let snapshot = snapshot.normalized();
        for session in &snapshot.sessions {
            append_session(&ctx, session);
        }
        let _ = switch_to(&ctx, snapshot.active_session);
        pane_view
    }

    fn bar_context(&self) -> MuxBarContext {
        MuxBarContext {
            bar: self.bar.clone(), // bar clone is needed to build the temporary bar context.
            buttons_box: self.buttons_box.clone(), // buttons_box clone is needed to build the temporary bar context.
            actions_box: self.actions_box.clone(), // actions_box clone is needed to build the temporary bar context.
            stack: self.stack.clone(), // stack clone is needed to build the temporary bar context.
            state: self.state.clone(), // state Rc clone is needed to build the temporary bar context.
            settings: self.settings.clone(), // settings Rc clone is needed to build the temporary bar context.
            focus: self.focus.clone(), // focus Rc clone is needed to build the temporary bar context.
        }
    }

    pub(crate) fn root(&self) -> &GtkBox {
        &self.root
    }

    pub(crate) fn to_snapshot(&self) -> PaneSnapshot {
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

    pub(crate) fn current_cwd(&self) -> Option<String> {
        current_session(&self.state).and_then(|session| session.current_cwd())
    }

    pub(crate) fn current_terminal(&self) -> Option<vte4::Terminal> {
        current_session(&self.state).map(|session| session.terminal().clone())
    }

    pub(crate) fn focus_terminal(&self) {
        if let Some(session) = current_session(&self.state) {
            self.focus.active_pane.set(self.focus.pane);
            if let Some(host) = &self.state.session_bar_host {
                self.mount_session_bar(host);
            }
            session.focus_terminal();
        }
    }

    pub(crate) fn mount_session_bar(&self, host: &GtkBox) {
        // Remove the bar GtkBox from its previous parent container
        self.bar.unparent();
        // Clear all existing child widgets from the session bar host
        while let Some(child) = host.first_child() {
            child.unparent();
        }
        // Append the pane's session bar to the host GtkBox
        host.append(&self.bar);
    }

    pub(crate) fn apply_profile(&self, profile_id: ProfileId) {
        self.state.profile_id.set(profile_id);
        for entry in self.state.sessions.borrow().iter() {
            entry.view.apply_profile(profile_id);
        }
    }

    pub(crate) fn apply_settings(&self, settings: &Settings) {
        let profile_id = self.state.profile_id.get();
        for entry in self.state.sessions.borrow().iter() {
            entry.view.apply_settings(settings, profile_id);
        }
    }

    pub(crate) fn new_session(&self) {
        let ctx = self.bar_context();
        let cwd = self.current_cwd();
        let snapshot = SessionSnapshot::new(cwd);
        let index = append_session(&ctx, &snapshot);
        let _ = switch_to(&ctx, index);
    }

    pub(crate) fn close_active_session(&self) -> bool {
        close_active_session(&self.bar_context())
    }

    pub(crate) fn focus_next_session(&self) -> bool {
        let session_count = self.state.sessions.borrow().len();
        if session_count <= 1 {
            return false;
        }

        let next = (self.state.active_index.get() + 1) % session_count;
        switch_to(&self.bar_context(), next)
    }

    pub(crate) fn focus_previous_session(&self) -> bool {
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
        switch_to(&self.bar_context(), previous)
    }

    pub(crate) fn focus_session(&self, index: usize) -> bool {
        switch_to(&self.bar_context(), index)
    }
}

fn append_session(ctx: &MuxBarContext, snapshot: &SessionSnapshot) -> usize {
    let session = Rc::new(SessionView::new(
        ctx.state.profile_id.get(),
        snapshot,
        ctx.settings.clone(), // settings clone is needed to share RefCell settings with SessionView.
    ));

    let focus_ref = ctx.focus.clone(); // focus clone is needed to pass focus status inside connect_focus_enter.
    let host_ref = ctx.state.session_bar_host.clone(); // host clone is needed to access session_bar_host inside focus callback.
    let bar_ref = ctx.bar.clone(); // bar clone is needed to mount GtkBox inside focus callback.
    session.connect_focus_enter(move || {
        focus_ref.active_pane.set(focus_ref.pane);
        if let Some(host) = &host_ref {
            // Widget unparent removes the bar GtkBox from its current parent
            bar_ref.unparent();
            // Clear current top session bar host children
            while let Some(child) = host.first_child() {
                child.unparent();
            }
            // Mount this active pane's session bar GtkBox to the top host
            host.append(&bar_ref);
        }
    });

    let session_id = ctx.state.next_session_id.get();
    ctx.state.next_session_id.set(session_id + 1);
    let stack_name = format!("mux-session-{session_id}");
    ctx.stack.add_named(session.root(), Some(&stack_name));
    ctx.state.sessions.borrow_mut().push(SessionEntry {
        stack_name,
        view: session,
    });

    ctx.state.sessions.borrow().len().saturating_sub(1)
}

fn switch_to(ctx: &MuxBarContext, index: usize) -> bool {
    let (stack_name, session) = {
        let sessions = ctx.state.sessions.borrow();
        let Some(entry) = sessions.get(index) else {
            return false;
        };
        (entry.stack_name.clone(), entry.view.clone())
    };

    ctx.state.active_index.set(index);
    ctx.stack.set_visible_child_name(&stack_name);
    ctx.focus.active_pane.set(ctx.focus.pane);
    session.focus_terminal();
    rebuild_bar(ctx);
    true
}

fn close_active_session(ctx: &MuxBarContext) -> bool {
    let session_count = ctx.state.sessions.borrow().len();
    if session_count <= 1 {
        return false;
    }

    let index = ctx.state.active_index.get().min(session_count.saturating_sub(1));
    let removed = ctx.state.sessions.borrow_mut().remove(index);
    ctx.stack.remove(removed.view.root());

    let next_index = index.min(session_count.saturating_sub(2));
    switch_to(ctx, next_index)
}

fn rebuild_bar(ctx: &MuxBarContext) {
    clear_children(&ctx.buttons_box);
    clear_children(&ctx.actions_box);

    let session_count = ctx.state.sessions.borrow().len();
    let current = ctx.state.active_index.get();

    for index in 0..session_count {
        let button = Button::with_label(&format!("{:02}", index + 1));
        button.add_css_class("magma-mux-session");
        button.set_focus_on_click(false);
        if index == current {
            button.add_css_class("active");
        }

        let ctx_ref = ctx.clone(); // ctx_ref clone is needed to switch to tab index inside click handler.
        button.connect_clicked(move |_| {
            let _ = switch_to(&ctx_ref, index);
        });
        ctx.buttons_box.append(&button);
    }

    let add_button = Button::builder()
        .icon_name("list-add-symbolic")
        .css_classes(["magma-mux-action"])
        .tooltip_text("New session")
        .build();
    let close_button = Button::builder()
        .icon_name("window-close-symbolic")
        .css_classes(["magma-mux-action", "close-session"])
        .tooltip_text("Close session")
        .sensitive(session_count > 1)
        .build();

    let ctx_ref = ctx.clone(); // ctx_ref clone is needed to create new session in clicked callback.
    add_button.connect_clicked(move |_| {
        let cwd = current_session(&ctx_ref.state).and_then(|session| session.current_cwd());
        let snapshot = SessionSnapshot::new(cwd);
        let index = append_session(&ctx_ref, &snapshot);
        let _ = switch_to(&ctx_ref, index);
    });

    let ctx_ref = ctx.clone(); // ctx_ref clone is needed to close active session in clicked callback.
    close_button.connect_clicked(move |_| {
        let _ = close_active_session(&ctx_ref);
    });

    ctx.actions_box.append(&add_button);
    ctx.actions_box.append(&close_button);
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
