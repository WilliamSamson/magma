mod browser;
mod host;
mod navigation;
mod persist;
mod tabs;

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use gtk::{
    pango, prelude::*, Box as GtkBox, Button, Entry, Image, Label, Orientation, Overflow,
    PolicyType, ProgressBar, ScrolledWindow, Stack,
};
use webkit6::{NetworkSession, WebContext};

use super::settings::Settings;
pub(super) use host::WebPaneHost;
use navigation::{bind_navigation, NavigationButtons};

pub(super) fn build_web_pane(
    settings: Rc<RefCell<Settings>>,
    context: WebContext,
    network_session: NetworkSession,
) -> Rc<WebPaneState> {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.set_vexpand(true);
    root.set_hexpand(true);
    root.set_width_request(0);
    root.set_overflow(Overflow::Hidden);
    root.add_css_class("magma-web-root");
    root.set_focusable(true);

    let tab_row = build_tab_bar();
    let controls = build_controls();
    let progress_bar = build_progress_bar();
    let status = build_status();
    let find_bar = build_find_bar();

    let content_stack = Stack::new();
    content_stack.set_vexpand(true);
    content_stack.set_hexpand(true);
    content_stack.set_width_request(0);
    content_stack.set_overflow(Overflow::Hidden);

    root.append(&tab_row.row);
    root.append(&controls.row);
    root.append(&progress_bar);
    root.append(&status);
    root.append(&find_bar.row);
    root.append(&content_stack);

    let state = Rc::new(WebPaneState {
        settings: settings.clone(),
        context,
        network_session,
        root: root.clone(),
        tabs: RefCell::new(Vec::new()),
        active_index: Cell::new(0),
        next_tab_id: Cell::new(0),
        tab_bar: tab_row.tab_bar,
        add_tab_button: tab_row.add_button,
        content_stack,
        address: controls.address,
        status,
        ssl_icon: controls.ssl_icon,
        progress_bar,
        back_button: controls.back_button,
        forward_button: controls.forward_button,
        reload_button: controls.reload_button,
        stop_button: controls.stop_button,
        find_bar: find_bar.row,
        find_entry: find_bar.entry,
        find_matches: find_bar.matches_label,
    });

    bind_navigation(
        &state,
        NavigationButtons {
            home_button: controls.home_button.clone(),
            zoom_out_button: controls.zoom_out_button.clone(),
            zoom_reset_button: controls.zoom_reset_button.clone(),
            go_button: controls.go_button.clone(),
            find_prev: find_bar.prev_button.clone(),
            find_next: find_bar.next_button.clone(),
            find_close: find_bar.close_button.clone(),
        },
    );

    tabs::restore_tabs(&state);

    state
}

pub(super) struct WebPaneState {
    pub(super) settings: Rc<RefCell<Settings>>,
    pub(super) context: WebContext,
    pub(super) network_session: NetworkSession,
    pub(super) root: GtkBox,
    // Tabs
    pub(super) tabs: RefCell<Vec<tabs::TabInfo>>,
    pub(super) active_index: Cell<usize>,
    pub(super) next_tab_id: Cell<u32>,
    pub(super) tab_bar: GtkBox,
    pub(super) add_tab_button: Button,
    pub(super) content_stack: Stack,
    // Controls
    pub(super) address: Entry,
    pub(super) status: Label,
    pub(super) ssl_icon: Image,
    pub(super) progress_bar: ProgressBar,
    pub(super) back_button: Button,
    pub(super) forward_button: Button,
    pub(super) reload_button: Button,
    pub(super) stop_button: Button,
    // Find
    pub(super) find_bar: GtkBox,
    pub(super) find_entry: Entry,
    pub(super) find_matches: Label,
}

struct TabBarWidgets {
    row: GtkBox,
    tab_bar: GtkBox,
    add_button: Button,
}

struct ControlWidgets {
    row: GtkBox,
    ssl_icon: Image,
    back_button: Button,
    forward_button: Button,
    reload_button: Button,
    stop_button: Button,
    home_button: Button,
    zoom_out_button: Button,
    zoom_reset_button: Button,
    go_button: Button,
    address: Entry,
}

struct FindBarWidgets {
    row: GtkBox,
    entry: Entry,
    matches_label: Label,
    prev_button: Button,
    next_button: Button,
    close_button: Button,
}

fn build_tab_bar() -> TabBarWidgets {
    let row = GtkBox::new(Orientation::Horizontal, 0);
    row.add_css_class("magma-web-tab-row");
    row.set_hexpand(true);
    row.set_overflow(Overflow::Hidden);

    let scroll = ScrolledWindow::new();
    scroll.add_css_class("magma-web-tab-scroll");
    scroll.set_hexpand(true);
    scroll.set_vexpand(false);
    scroll.set_policy(PolicyType::Automatic, PolicyType::Never);

    let tab_bar = GtkBox::new(Orientation::Horizontal, 2);
    tab_bar.add_css_class("magma-web-tabs");
    scroll.set_child(Some(&tab_bar));

    let add_button = Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text("New tab (Ctrl+T)")
        .css_classes(["magma-web-tab-add"])
        .build();

    row.append(&scroll);
    row.append(&add_button);

    TabBarWidgets {
        row,
        tab_bar,
        add_button,
    }
}

fn build_controls() -> ControlWidgets {
    let row = GtkBox::new(Orientation::Horizontal, 0);
    row.add_css_class("magma-web-bar");
    row.add_css_class("magma-web-controls");
    row.set_hexpand(true);
    row.set_overflow(Overflow::Hidden);

    let nav = GtkBox::new(Orientation::Horizontal, 2);
    nav.add_css_class("magma-web-nav");

    let back_button = icon_button("go-previous-symbolic", "Back");
    let forward_button = icon_button("go-next-symbolic", "Forward");
    let reload_button = icon_button("view-refresh-symbolic", "Reload");
    let stop_button = icon_button("process-stop-symbolic", "Stop loading");
    stop_button.set_visible(false);
    let home_button = icon_button("go-home-symbolic", "Home");
    let zoom_out_button = icon_button("zoom-out-symbolic", "Zoom out");
    let zoom_reset_button = text_button("100%", "Reset zoom");
    let go_button = icon_button("go-jump-symbolic", "Open");

    nav.append(&back_button);
    nav.append(&forward_button);
    nav.append(&reload_button);
    nav.append(&stop_button);
    nav.append(&home_button);
    nav.append(&zoom_out_button);
    nav.append(&zoom_reset_button);

    let ssl_icon = Image::from_icon_name("channel-insecure-symbolic");
    ssl_icon.add_css_class("magma-web-ssl");
    ssl_icon.set_visible(false);

    let address = Entry::new();
    address.add_css_class("magma-web-entry");
    address.set_placeholder_text(Some("search or enter address"));
    address.set_hexpand(true);
    address.set_width_request(0);

    let address_shell = GtkBox::new(Orientation::Horizontal, 0);
    address_shell.add_css_class("magma-web-address-shell");
    address_shell.set_hexpand(true);
    address_shell.set_overflow(Overflow::Hidden);
    address_shell.append(&ssl_icon);
    address_shell.append(&address);
    address_shell.append(&go_button);

    row.append(&nav);
    row.append(&address_shell);

    ControlWidgets {
        row,
        ssl_icon,
        back_button,
        forward_button,
        reload_button,
        stop_button,
        home_button,
        zoom_out_button,
        zoom_reset_button,
        go_button,
        address,
    }
}

fn build_progress_bar() -> ProgressBar {
    let bar = ProgressBar::new();
    bar.add_css_class("magma-web-progress");
    bar.set_fraction(0.0);
    bar.set_visible(false);
    bar
}

fn build_status() -> Label {
    let status = Label::new(Some("ready"));
    status.add_css_class("magma-web-status");
    status.set_xalign(0.0);
    status.set_ellipsize(pango::EllipsizeMode::End);
    status
}

fn build_find_bar() -> FindBarWidgets {
    let row = GtkBox::new(Orientation::Horizontal, 4);
    row.add_css_class("magma-web-find-bar");
    row.set_visible(false);

    let entry = Entry::new();
    entry.add_css_class("magma-web-find-entry");
    entry.set_placeholder_text(Some("find in page"));
    entry.set_hexpand(true);
    entry.set_width_request(0);

    let matches_label = Label::new(None);
    matches_label.add_css_class("magma-web-find-matches");

    let prev_button = icon_button("go-up-symbolic", "Previous match");
    let next_button = icon_button("go-down-symbolic", "Next match");
    let close_button = icon_button("window-close-symbolic", "Close (Esc)");

    row.append(&entry);
    row.append(&matches_label);
    row.append(&prev_button);
    row.append(&next_button);
    row.append(&close_button);

    FindBarWidgets {
        row,
        entry,
        matches_label,
        prev_button,
        next_button,
        close_button,
    }
}

fn icon_button(icon_name: &str, tooltip: &str) -> Button {
    Button::builder()
        .icon_name(icon_name)
        .tooltip_text(tooltip)
        .css_classes(["magma-web-button"])
        .build()
}

fn text_button(label: &str, tooltip: &str) -> Button {
    Button::builder()
        .label(label)
        .tooltip_text(tooltip)
        .css_classes(["magma-web-button", "magma-web-text-button"])
        .build()
}
