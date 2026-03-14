use std::{cell::Cell, cell::RefCell, rc::Rc};

use gtk::{
    prelude::*, Align, Box as GtkBox, Button, Image, Orientation, Overflow, PolicyType,
    Revealer, RevealerTransitionType, ScrolledWindow,
};
use webkit6::WebContext;

use super::{logr, settings::Settings, view, web};

const PANE_WIDTH: i32 = 420;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SidePaneKind {
    None,
    Logr,
    Web,
    View,
}

#[derive(Clone)]
pub(super) struct SidePanes {
    buttons: PaneButtons,
    revealers: PaneRevealers,
    active_pane: Rc<Cell<SidePaneKind>>,
    web_host: web::WebPaneHost,
    view_host: view::ViewPaneHost,
}

#[derive(Clone)]
struct PaneButtons {
    handle: GtkBox,
    logr: Button,
    web: Button,
    view: Button,
}

#[derive(Clone)]
struct PaneRevealers {
    logr: Revealer,
    web: Revealer,
    view: Revealer,
}

pub(super) fn build_side_panes(
    settings: Rc<RefCell<Settings>>,
    cwd_provider: view::CwdProvider,
) -> SidePanes {
    // Rc<Cell<SidePaneKind>> shares the currently open side pane across the segmented-handle callbacks on the GTK thread.
    let active_pane = Rc::new(Cell::new(if settings.borrow().logr_panel_open {
        SidePaneKind::Logr
    } else {
        SidePaneKind::None
    }));

    let handle = GtkBox::new(Orientation::Vertical, 3);
    handle.add_css_class("obsidian-handle");
    handle.set_vexpand(false);
    handle.set_valign(Align::Center);

    let logr_button = handle_button("view-list-symbolic", "Open logr");
    let web_button = handle_button("applications-internet-symbolic", "Open web");
    let view_button = handle_button("image-x-generic-symbolic", "Open viewer");
    handle.append(&logr_button);
    handle.append(&web_button);
    handle.append(&view_button);

    let logr_revealer = build_revealer(&wrap_pane(&logr::build_logr_pane()));
    let web_host = web::WebPaneHost::new(settings);
    let web_revealer = build_revealer(&wrap_pane(web_host.widget()));
    let view_host = view::ViewPaneHost::new(cwd_provider, WebContext::new());
    let view_revealer = build_revealer(&wrap_pane(view_host.widget()));

    let buttons = PaneButtons {
        handle: handle.clone(),
        logr: logr_button.clone(),
        web: web_button.clone(),
        view: view_button.clone(),
    };
    let revealers = PaneRevealers {
        logr: logr_revealer.clone(),
        web: web_revealer.clone(),
        view: view_revealer.clone(),
    };

    let side_panes = SidePanes {
        buttons,
        revealers,
        active_pane,
        web_host,
        view_host,
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

    side_panes.sync();
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

    fn sync(&self) {
        sync_side_panes(
            self.active_pane.get(),
            &self.revealers,
            &self.buttons,
            &self.web_host,
            &self.view_host,
        );
    }
}

fn handle_button(icon_name: &str, tooltip: &str) -> Button {
    let button = Button::builder()
        .css_classes(["obsidian-handle-segment"])
        .tooltip_text(tooltip)
        .build();
    let icon = Image::from_icon_name(icon_name);
    icon.add_css_class("obsidian-handle-icon");
    button.set_child(Some(&icon));
    button
}

fn wrap_pane(child: &impl IsA<gtk::Widget>) -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.add_css_class("obsidian-right-pane");
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
) {
    let show_logr = active == SidePaneKind::Logr;
    let show_web = active == SidePaneKind::Web;
    let show_view = active == SidePaneKind::View;

    if show_web {
        web_host.ensure_loaded();
    }
    if show_view {
        view_host.ensure_loaded();
    }

    revealers.logr.set_visible(show_logr);
    revealers.web.set_visible(show_web);
    revealers.view.set_visible(show_view);
    revealers.logr.set_reveal_child(show_logr);
    revealers.web.set_reveal_child(show_web);
    revealers.view.set_reveal_child(show_view);

    set_active_button(&buttons.logr, show_logr);
    set_active_button(&buttons.web, show_web);
    set_active_button(&buttons.view, show_view);

    if active == SidePaneKind::None {
        buttons.handle.add_css_class("collapsed");
    } else {
        buttons.handle.remove_css_class("collapsed");
    }
}

fn set_active_button(button: &Button, active: bool) {
    if active {
        button.add_css_class("active");
    } else {
        button.remove_css_class("active");
    }
}
