use std::{cell::RefCell, rc::Rc};

use gtk::{
    prelude::*,
    Align, Box as GtkBox, Button, DropDown, Entry, Image, Label, Orientation, SpinButton,
    StringList, Switch,
};

use super::setup_label;
use crate::linux_terminal::{
    runtime,
    settings::{self, Settings},
};

pub(super) fn build_topbar() -> GtkBox {
    let topbar = GtkBox::new(Orientation::Horizontal, 8);
    topbar.add_css_class("magma-setup-topbar");
    topbar.append(&dot("magma-setup-dot red"));
    topbar.append(&dot("magma-setup-dot amber"));
    topbar.append(&dot("magma-setup-dot green"));

    let title = setup_label("boot magma", "magma-setup-topbar-title");
    title.set_hexpand(true);
    title.set_xalign(1.0);
    topbar.append(&title);
    topbar
}

pub(super) fn build_hero() -> GtkBox {
    let hero = GtkBox::new(Orientation::Vertical, 8);
    hero.add_css_class("magma-setup-hero");

    hero.append(&setup_label("FIRST RUN SETUP", "magma-setup-eyebrow"));
    hero.append(&setup_label("prepare the terminal", "magma-setup-title"));

    let copy = setup_label(
        "magma needs a few first-run decisions before it becomes the active terminal on this device. set the runtime, choose the workspace defaults, then enter the shell.",
        "magma-setup-copy",
    );
    copy.set_wrap(true);
    hero.append(&copy);
    hero
}

pub(super) fn build_progress() -> (GtkBox, Vec<GtkBox>) {
    let progress = GtkBox::new(Orientation::Horizontal, 18);
    progress.add_css_class("magma-setup-progress");
    progress.set_homogeneous(true);

    let mut markers = Vec::new();
    for (index, name) in ["runtime", "workspace", "appearance"].iter().enumerate() {
        let marker = GtkBox::new(Orientation::Horizontal, 10);
        marker.add_css_class("magma-setup-step");
        marker.set_hexpand(true);

        let number = setup_label(&format!("{:02}", index + 1), "magma-setup-step-index");
        let label = setup_label(name, "magma-setup-step-label");
        label.set_hexpand(true);

        marker.append(&number);
        marker.append(&label);
        progress.append(&marker);
        markers.push(marker);
    }

    (progress, markers)
}

pub(super) fn build_nav_button(
    text: &str,
    icon_name: &str,
    icon_first: bool,
    class_name: &str,
) -> (Button, Label) {
    let button = Button::builder().css_classes([class_name]).build();
    let content = GtkBox::new(Orientation::Horizontal, 8);
    content.add_css_class("magma-setup-nav-content");

    let icon = Image::from_icon_name(icon_name);
    icon.add_css_class("magma-setup-nav-icon");

    let label = setup_label(text, "magma-setup-nav-label");

    if icon_first {
        content.append(&icon);
        content.append(&label);
    } else {
        content.append(&label);
        content.append(&icon);
    }

    button.set_child(Some(&content));
    (button, label)
}

pub(super) fn build_runtime_step(draft: &Rc<RefCell<Settings>>, on_checkpoint: &Rc<dyn Fn()>) -> GtkBox {
    let page = step_page(
        "runtime",
        "choose the shell command and confirm where magma will write its local configuration.",
    );
    let shell_command = draft.borrow().shell.clone();
    let resolved_shell = runtime::resolve_shell(&shell_command);
    let shell_entry = entry_field(&shell_command);
    let resolved_value = value_label(&resolved_shell);
    let config_value = value_label(&settings::settings_path().display().to_string());

    {
        let draft = draft.clone();
        let on_checkpoint = on_checkpoint.clone();
        let resolved_value = resolved_value.clone();
        shell_entry.connect_changed(move |entry| {
            let shell = entry.text().to_string();
            draft.borrow_mut().shell = shell.clone();
            resolved_value.set_text(&runtime::resolve_shell(&shell));
            on_checkpoint();
        });
    }

    page.append(&setting_row("shell command", "the shell magma should launch", &shell_entry));
    page.append(&display_row("resolved shell", &resolved_value));
    page.append(&display_row("config path", &config_value));
    page
}

pub(super) fn build_workspace_step(
    draft: &Rc<RefCell<Settings>>,
    on_checkpoint: &Rc<dyn Fn()>,
) -> GtkBox {
    let page = step_page(
        "workspace",
        "set the browser target for the web pane and decide whether the side pane should open on launch.",
    );
    let browsers = ["google", "duckduckgo", "bing", "brave"];
    let selected_browser = browsers
        .iter()
        .position(|browser| *browser == draft.borrow().default_browser)
        .unwrap_or(0) as u32;
    let browser_dropdown = dropdown_field(&browsers, selected_browser);
    let logr_switch = switch_field(draft.borrow().logr_panel_open);

    {
        let draft = draft.clone();
        let on_checkpoint = on_checkpoint.clone();
        browser_dropdown.connect_selected_notify(move |dropdown| {
            let browser = match dropdown.selected() {
                1 => "duckduckgo",
                2 => "bing",
                3 => "brave",
                _ => "google",
            };
            draft.borrow_mut().default_browser = browser.to_string();
            on_checkpoint();
        });
    }

    {
        let draft = draft.clone();
        let on_checkpoint = on_checkpoint.clone();
        logr_switch.connect_state_set(move |_, active| {
            draft.borrow_mut().logr_panel_open = active;
            on_checkpoint();
            gtk::glib::Propagation::Proceed
        });
    }

    page.append(&setting_row("default browser", "used by the web viewer pane", &browser_dropdown));
    page.append(&setting_row("open side pane", "start with logr and web pane available", &logr_switch));
    page
}

pub(super) fn build_appearance_step(
    draft: &Rc<RefCell<Settings>>,
    on_checkpoint: &Rc<dyn Fn()>,
) -> GtkBox {
    let page = step_page(
        "appearance",
        "dial in the terminal and app scale before the first workspace session opens.",
    );
    let terminal_font = spin_field(draft.borrow().font_size as f64, 6.0, 32.0);
    let app_font = spin_field(draft.borrow().app_font_size as f64, 8.0, 20.0);
    let cursor_styles = ["ibeam", "block", "underline"];
    let selected_cursor = cursor_styles
        .iter()
        .position(|style| *style == draft.borrow().cursor_style)
        .unwrap_or(0) as u32;
    let cursor_dropdown = dropdown_field(&cursor_styles, selected_cursor);
    let blink_switch = switch_field(draft.borrow().cursor_blink);

    {
        let draft = draft.clone();
        let on_checkpoint = on_checkpoint.clone();
        terminal_font.connect_value_changed(move |spin| {
            draft.borrow_mut().font_size = spin.value() as u32;
            on_checkpoint();
        });
    }

    {
        let draft = draft.clone();
        let on_checkpoint = on_checkpoint.clone();
        app_font.connect_value_changed(move |spin| {
            draft.borrow_mut().app_font_size = spin.value() as u32;
            on_checkpoint();
        });
    }

    {
        let draft = draft.clone();
        let on_checkpoint = on_checkpoint.clone();
        cursor_dropdown.connect_selected_notify(move |dropdown| {
            let style = match dropdown.selected() {
                1 => "block",
                2 => "underline",
                _ => "ibeam",
            };
            draft.borrow_mut().cursor_style = style.to_string();
            on_checkpoint();
        });
    }

    {
        let draft = draft.clone();
        let on_checkpoint = on_checkpoint.clone();
        blink_switch.connect_state_set(move |_, active| {
            draft.borrow_mut().cursor_blink = active;
            on_checkpoint();
            gtk::glib::Propagation::Proceed
        });
    }

    page.append(&setting_row("terminal font size", "shell text scale", &terminal_font));
    page.append(&setting_row("app font size", "window and interface scale", &app_font));
    page.append(&setting_row("cursor style", "terminal cursor shape", &cursor_dropdown));
    page.append(&setting_row("cursor blink", "blink active cursor", &blink_switch));
    page
}

fn step_page(title: &str, copy: &str) -> GtkBox {
    let page = GtkBox::new(Orientation::Vertical, 14);
    page.add_css_class("magma-setup-page");

    page.append(&setup_label(title, "magma-setup-page-title"));

    let copy = setup_label(copy, "magma-setup-page-copy");
    copy.set_wrap(true);
    page.append(&copy);

    page
}

fn setting_row(title: &str, copy: &str, control: &impl IsA<gtk::Widget>) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 16);
    row.add_css_class("magma-setup-setting");

    let copy_box = GtkBox::new(Orientation::Vertical, 4);
    copy_box.set_hexpand(true);
    copy_box.append(&setup_label(title, "magma-setup-setting-title"));

    let hint = setup_label(copy, "magma-setup-setting-copy");
    hint.set_wrap(true);
    copy_box.append(&hint);

    row.append(&copy_box);
    row.append(control);
    row
}

fn display_row(title: &str, value: &Label) -> GtkBox {
    let row = GtkBox::new(Orientation::Vertical, 6);
    row.add_css_class("magma-setup-setting");
    row.append(&setup_label(title, "magma-setup-setting-title"));
    row.append(value);
    row
}

fn entry_field(value: &str) -> Entry {
    let entry = Entry::new();
    entry.set_text(value);
    entry.add_css_class("magma-settings-entry");
    entry.set_hexpand(false);
    entry.set_width_chars(24);
    entry
}

fn spin_field(value: f64, min: f64, max: f64) -> SpinButton {
    let spin = SpinButton::with_range(min, max, 1.0);
    spin.set_value(value);
    spin.add_css_class("magma-settings-spin");
    spin.set_digits(0);
    spin.set_halign(Align::End);
    spin
}

fn switch_field(active: bool) -> Switch {
    let switch = Switch::new();
    switch.set_active(active);
    switch.add_css_class("magma-settings-switch");
    switch.set_halign(Align::End);
    switch.set_valign(Align::Center);
    switch
}

fn dropdown_field(items: &[&str], selected: u32) -> DropDown {
    let model = StringList::new(items);
    let dropdown = DropDown::builder().model(&model).selected(selected).build();
    dropdown.add_css_class("magma-settings-dropdown");
    dropdown.set_halign(Align::End);
    dropdown
}

fn value_label(text: &str) -> Label {
    let label = setup_label(text, "magma-setup-value");
    label.set_wrap(true);
    label.set_wrap_mode(gtk::pango::WrapMode::Char);
    label
}

fn dot(class_names: &str) -> Label {
    let dot = Label::new(Some("●"));
    for class_name in class_names.split_whitespace() {
        dot.add_css_class(class_name);
    }
    dot
}
