use std::{cell::Cell, cell::RefCell, rc::Rc};

use gtk::{
    prelude::*, Align, Box as GtkBox, Button, Image, Orientation, Overflow, PolicyType,
    Revealer, RevealerTransitionType, ScrolledWindow,
};
use webkit6::WebContext;

use super::{agent, git, logr, settings::Settings, view, web};
use crate::agent::{
    context::{load_ui_runtime_state, write_ui_runtime_state},
    effects::UiEffect,
    executor::{AgentRuntimeHandle, ExecutorConfig},
};

const PANE_WIDTH: i32 = 420;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SidePaneKind {
    None,
    Logr,
    Web,
    View,
    Git,
    Agent,
}

#[derive(Clone)]
pub(super) struct SidePanes {
    buttons: PaneButtons,
    revealers: PaneRevealers,
    active_pane: Rc<Cell<SidePaneKind>>,
    web_host: web::WebPaneHost,
    view_host: view::ViewPaneHost,
    git_host: git::GitPaneHost,
    agent_host: agent::AgentPaneHost,
}

#[derive(Clone)]
struct PaneButtons {
    handle: GtkBox,
    logr: Button,
    web: Button,
    view: Button,
    git: Button,
    agent: Button,
}

#[derive(Clone)]
struct PaneRevealers {
    logr: Revealer,
    web: Revealer,
    view: Revealer,
    git: Revealer,
    agent: Revealer,
}

pub(super) fn build_side_panes(
    settings: Rc<RefCell<Settings>>,
    cwd_provider: view::CwdProvider,
) -> SidePanes {
    let runtime = AgentRuntimeHandle::shared(ExecutorConfig {
        confidence_threshold: settings.borrow().agent_confidence_threshold,
        passive_mode: settings.borrow().agent_passive_mode,
    });
    // Rc<Cell<SidePaneKind>> shares the currently open side pane across the segmented-handle callbacks on the GTK thread.
    let active_pane = Rc::new(Cell::new(if settings.borrow().logr_panel_open {
        SidePaneKind::Logr
    } else {
        SidePaneKind::None
    }));

    let handle = GtkBox::new(Orientation::Vertical, 3);
    handle.add_css_class("magma-handle");
    handle.set_vexpand(false);
    handle.set_valign(Align::Center);

    let logr_button = handle_button("view-list-symbolic", "Open logr");
    let web_button = handle_button("network-workgroup-symbolic", "Open web");
    let view_button = handle_button("image-x-generic-symbolic", "Open viewer");
    let git_button = handle_button("emblem-shared-symbolic", "Open git");
    let agent_button = handle_button("applications-internet-symbolic", "Open agent");
    handle.append(&logr_button);
    handle.append(&web_button);
    handle.append(&view_button);
    handle.append(&git_button);
    handle.append(&agent_button);

    let logr_revealer = build_revealer(&wrap_pane(&logr::build_logr_pane()));
    let web_host = web::WebPaneHost::new(settings.clone());
    let web_revealer = build_revealer(&wrap_pane(web_host.widget()));
    let view_host = view::ViewPaneHost::new(cwd_provider.clone(), WebContext::new());
    let view_revealer = build_revealer(&wrap_pane(view_host.widget()));
    let git_host = git::GitPaneHost::new(cwd_provider);
    let git_revealer = build_revealer(&wrap_pane(git_host.widget()));
    let agent_host = agent::AgentPaneHost::new(settings.clone());
    let agent_revealer = build_revealer(&wrap_pane(agent_host.widget()));

    let buttons = PaneButtons {
        handle: handle.clone(),
        logr: logr_button.clone(),
        web: web_button.clone(),
        view: view_button.clone(),
        git: git_button.clone(),
        agent: agent_button.clone(),
    };
    let revealers = PaneRevealers {
        logr: logr_revealer.clone(),
        web: web_revealer.clone(),
        view: view_revealer.clone(),
        git: git_revealer.clone(),
        agent: agent_revealer.clone(),
    };

    let side_panes = SidePanes {
        buttons,
        revealers,
        active_pane,
        web_host,
        view_host,
        git_host,
        agent_host,
    };

    {
        let side_panes = side_panes.clone();
        logr_button.connect_clicked(move |_| side_panes.toggle(SidePaneKind::Logr));
    }

    {
        let side_panes = side_panes.clone();
        web_button.connect_clicked(move |_| side_panes.toggle(SidePaneKind::Web));
    }

    {
        let side_panes = side_panes.clone();
        view_button.connect_clicked(move |_| side_panes.toggle(SidePaneKind::View));
    }

    {
        let side_panes = side_panes.clone();
        git_button.connect_clicked(move |_| side_panes.toggle(SidePaneKind::Git));
    }

    {
        let side_panes = side_panes.clone();
        agent_button.connect_clicked(move |_| side_panes.toggle(SidePaneKind::Agent));
    }

    side_panes.sync();
    {
        let side_panes = side_panes.clone();
        gtk::glib::timeout_add_local(std::time::Duration::from_millis(150), move || {
            for effect in runtime.drain_ui_effects() {
                match effect {
                    UiEffect::OpenPane(pane) => side_panes.open(match pane {
                        crate::agent::actions::PaneType::Logr => SidePaneKind::Logr,
                        crate::agent::actions::PaneType::Web => SidePaneKind::Web,
                        crate::agent::actions::PaneType::View => SidePaneKind::View,
                        crate::agent::actions::PaneType::Git => SidePaneKind::Git,
                        crate::agent::actions::PaneType::Agent => SidePaneKind::Agent,
                    }),
                    UiEffect::Message(_) => {
                        // Messages are now surfaced in the agent pane's
                        // conversation log via runtime.log_entries().
                    }
                }
            }
            gtk::glib::ControlFlow::Continue
        });
    }
    side_panes
}

impl SidePanes {
    pub(super) fn handle(&self) -> &GtkBox {
        &self.buttons.handle
    }

    pub(super) fn logr_revealer(&self) -> &Revealer {
        &self.revealers.logr
    }

    pub(super) fn web_revealer(&self) -> &Revealer {
        &self.revealers.web
    }

    pub(super) fn view_revealer(&self) -> &Revealer {
        &self.revealers.view
    }

    pub(super) fn git_revealer(&self) -> &Revealer {
        &self.revealers.git
    }

    pub(super) fn agent_revealer(&self) -> &Revealer {
        &self.revealers.agent
    }

    pub(super) fn apply_settings(&self, settings: &Settings) {
        let next = match (settings.logr_panel_open, self.active_pane.get()) {
            (true, SidePaneKind::None) => SidePaneKind::Logr,
            (false, SidePaneKind::Logr) => SidePaneKind::None,
            _ => self.active_pane.get(),
        };
        if next != self.active_pane.get() {
            self.active_pane.set(next);
            self.sync();
        }
    }

    pub(super) fn clear_web_data(&self) {
        self.web_host.clear_persistent_data();
    }

    fn toggle(&self, pane: SidePaneKind) {
        let next = if self.active_pane.get() == pane {
            SidePaneKind::None
        } else {
            pane
        };
        self.active_pane.set(next);
        self.sync();
    }

    pub(crate) fn open(&self, pane: SidePaneKind) {
        self.active_pane.set(pane);
        self.sync();
    }

    fn sync(&self) {
        sync_side_panes(
            self.active_pane.get(),
            &self.revealers,
            &self.buttons,
            &self.web_host,
            &self.view_host,
            &self.git_host,
            &self.agent_host,
        );
    }
}

fn handle_button(icon_name: &str, tooltip: &str) -> Button {
    let button = Button::builder()
        .css_classes(["magma-handle-segment"])
        .tooltip_text(tooltip)
        .build();
    let icon = Image::from_icon_name(icon_name);
    icon.add_css_class("magma-handle-icon");
    button.set_child(Some(&icon));
    button
}

fn wrap_pane(child: &impl IsA<gtk::Widget>) -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.add_css_class("magma-right-pane");
    root.set_size_request(PANE_WIDTH, -1);
    root.set_width_request(PANE_WIDTH);
    root.set_hexpand(false);
    root.set_vexpand(true);
    root.set_valign(Align::Fill);
    root.set_overflow(Overflow::Hidden);
    root.append(child);
    root
}

fn build_revealer(child: &impl IsA<gtk::Widget>) -> Revealer {
    let revealer = Revealer::builder()
        .transition_type(RevealerTransitionType::SlideLeft)
        .transition_duration(250)
        .build();
    revealer.set_visible(false);
    revealer.set_hexpand(false);
    revealer.set_vexpand(true);
    revealer.set_halign(Align::End);
    revealer.set_width_request(PANE_WIDTH);

    let frame = ScrolledWindow::new();
    frame.set_hexpand(false);
    frame.set_vexpand(true);
    frame.set_min_content_width(PANE_WIDTH);
    frame.set_max_content_width(PANE_WIDTH);
    frame.set_propagate_natural_height(false);
    frame.set_propagate_natural_width(false);
    frame.set_policy(PolicyType::Never, PolicyType::Never);
    frame.set_child(Some(child));

    revealer.set_child(Some(&frame));
    revealer
}

fn sync_side_panes(
    active: SidePaneKind,
    revealers: &PaneRevealers,
    buttons: &PaneButtons,
    web_host: &web::WebPaneHost,
    view_host: &view::ViewPaneHost,
    git_host: &git::GitPaneHost,
    agent_host: &agent::AgentPaneHost,
) {
    let show_logr = active == SidePaneKind::Logr;
    let show_web = active == SidePaneKind::Web;
    let show_view = active == SidePaneKind::View;
    let show_git = active == SidePaneKind::Git;
    let show_agent = active == SidePaneKind::Agent;

    if show_web {
        web_host.ensure_loaded();
    }
    if show_view {
        view_host.ensure_loaded();
    }
    if show_git {
        git_host.ensure_loaded();
    }
    if show_agent {
        agent_host.ensure_loaded();
    }

    revealers.logr.set_visible(show_logr);
    revealers.web.set_visible(show_web);
    revealers.view.set_visible(show_view);
    revealers.git.set_visible(show_git);
    revealers.agent.set_visible(show_agent);
    revealers.logr.set_reveal_child(show_logr);
    revealers.web.set_reveal_child(show_web);
    revealers.view.set_reveal_child(show_view);
    revealers.git.set_reveal_child(show_git);
    revealers.agent.set_reveal_child(show_agent);

    set_active_button(&buttons.logr, show_logr);
    set_active_button(&buttons.web, show_web);
    set_active_button(&buttons.view, show_view);
    set_active_button(&buttons.git, show_git);
    set_active_button(&buttons.agent, show_agent);

    if active == SidePaneKind::None {
        buttons.handle.add_css_class("collapsed");
    } else {
        buttons.handle.remove_css_class("collapsed");
    }

    let mut state = load_ui_runtime_state();
    state.side_pane = match active {
        SidePaneKind::None => None,
        SidePaneKind::Logr => Some("logr".to_string()),
        SidePaneKind::Web => Some("web".to_string()),
        SidePaneKind::View => Some("view".to_string()),
        SidePaneKind::Git => Some("git".to_string()),
        SidePaneKind::Agent => Some("agent".to_string()),
    };
    write_ui_runtime_state(&state);
}

fn set_active_button(button: &Button, active: bool) {
    if active {
        button.add_css_class("active");
    } else {
        button.remove_css_class("active");
    }
}
