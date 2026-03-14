mod browser;
mod navigation;

use std::{cell::RefCell, rc::Rc};

use gtk::{
    pango, prelude::*, Box as GtkBox, Button, Entry, Label, Orientation, Overflow, PolicyType,
    ScrolledWindow,
};
use webkit6::{Settings as WebSettings, WebView};

use super::settings::Settings;
use browser::load_home_page;
use navigation::bind_navigation;

pub(super) fn build_web_pane(settings: Rc<RefCell<Settings>>) -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.set_vexpand(true);
    root.set_hexpand(true);
    root.set_width_request(0);
    root.set_overflow(Overflow::Hidden);
    root.add_css_class("obsidian-web-root");

    let header = build_header();
    let controls = build_controls();
    let status = build_status();
    let (frame, web_view) = build_web_frame();

    root.append(&header.row);
    root.append(&controls.row);
    root.append(&status);
    root.append(&frame);

    let state = Rc::new(WebPaneState {
        settings,
        web_view,
        title: header.title,
        address: controls.address,
        status,
        back_button: controls.back_button,
        forward_button: controls.forward_button,
    });

    bind_navigation(&state, &controls.reload_button, &controls.home_button, &controls.go_button);
    load_home_page(&state);

    root
}

pub(super) struct WebPaneState {
    pub(super) settings: Rc<RefCell<Settings>>,
    pub(super) web_view: WebView,
    pub(super) title: Label,
    pub(super) address: Entry,
    pub(super) status: Label,
    pub(super) back_button: Button,
    pub(super) forward_button: Button,
}

struct HeaderWidgets {
    row: GtkBox,
    title: Label,
}

struct ControlWidgets {
    row: GtkBox,
    back_button: Button,
    forward_button: Button,
    reload_button: Button,
    home_button: Button,
    go_button: Button,
    address: Entry,
}

fn build_header() -> HeaderWidgets {
    let row = GtkBox::new(Orientation::Horizontal, 0);
    row.add_css_class("obsidian-web-header");

    let title = Label::new(Some("browser"));
    title.add_css_class("obsidian-web-title");
    title.set_xalign(0.0);
    title.set_hexpand(true);
    title.set_ellipsize(pango::EllipsizeMode::End);

    row.append(&title);

    HeaderWidgets { row, title }
}

fn build_controls() -> ControlWidgets {
    let row = GtkBox::new(Orientation::Horizontal, 0);
    row.add_css_class("obsidian-web-bar");
    row.add_css_class("obsidian-web-controls");
    row.set_hexpand(true);
    row.set_overflow(Overflow::Hidden);

    let nav = GtkBox::new(Orientation::Horizontal, 2);
    nav.add_css_class("obsidian-web-nav");

    let back_button = icon_button("go-previous-symbolic", "Back");
    let forward_button = icon_button("go-next-symbolic", "Forward");
    let reload_button = icon_button("view-refresh-symbolic", "Reload");
    let home_button = icon_button("go-home-symbolic", "Home");
    let go_button = icon_button("go-jump-symbolic", "Open");

    nav.append(&back_button);
    nav.append(&forward_button);
    nav.append(&reload_button);
    nav.append(&home_button);

    let address = Entry::new();
    address.add_css_class("obsidian-web-entry");
    address.set_placeholder_text(Some("search or enter address"));
    address.set_hexpand(true);
    address.set_width_request(0);

    let address_shell = GtkBox::new(Orientation::Horizontal, 0);
    address_shell.add_css_class("obsidian-web-address-shell");
    address_shell.set_hexpand(true);
    address_shell.set_overflow(Overflow::Hidden);
    address_shell.append(&address);
    address_shell.append(&go_button);

    row.append(&nav);
    row.append(&address_shell);

    ControlWidgets {
        row,
        back_button,
        forward_button,
        reload_button,
        home_button,
        go_button,
        address,
    }
}

fn build_status() -> Label {
    let status = Label::new(Some("ready"));
    status.add_css_class("obsidian-web-status");
    status.set_xalign(0.0);
    status.set_ellipsize(pango::EllipsizeMode::End);
    status
}

fn build_web_frame() -> (ScrolledWindow, WebView) {
    let web_settings = WebSettings::new();
    web_settings.set_enable_developer_extras(true);
    web_settings.set_enable_back_forward_navigation_gestures(true);
    web_settings.set_enable_smooth_scrolling(true);

    let web_view = WebView::builder()
        .settings(&web_settings)
        .hexpand(true)
        .vexpand(true)
        .build();
    web_view.add_css_class("obsidian-webview");
    web_view.set_hexpand(false);
    web_view.set_vexpand(true);
    web_view.set_width_request(0);
    web_view.set_overflow(Overflow::Hidden);

    let frame = ScrolledWindow::new();
    frame.add_css_class("obsidian-web-frame");
    frame.set_hexpand(true);
    frame.set_vexpand(true);
    frame.set_min_content_width(0);
    frame.set_max_content_width(392);
    frame.set_propagate_natural_width(false);
    frame.set_policy(PolicyType::Automatic, PolicyType::Automatic);
    frame.set_vscrollbar_policy(PolicyType::Automatic);
    frame.set_width_request(0);
    frame.set_overflow(Overflow::Hidden);
    frame.set_child(Some(&web_view));

    (frame, web_view)
}

fn icon_button(icon_name: &str, tooltip: &str) -> Button {
    Button::builder()
        .icon_name(icon_name)
        .tooltip_text(tooltip)
        .css_classes(["obsidian-web-button"])
        .build()
}
