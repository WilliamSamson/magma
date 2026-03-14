use std::{cell::RefCell, rc::Rc};

use gtk::{glib, prelude::*, Box as GtkBox, Orientation, ScrolledWindow, Stack};

use crate::linux_terminal::meta::APP_VERSION;

use super::{
    browser::build_browser_section,
    settings_path,
    terminal::{build_appearance_section, build_terminal_section, preview_settings},
    widgets::{action_row, info_row, section_label, switch_row, text_row},
    Settings,
};

pub(super) fn build_main_page(
    page_stack: &Stack,
    settings: &Rc<RefCell<Settings>>,
    on_apply: Rc<dyn Fn(&Settings)>,
    on_clear_browser_data: Rc<dyn Fn()>,
) -> ScrolledWindow {
    let content = GtkBox::new(Orientation::Vertical, 0);
    content.add_css_class("obsidian-settings-content");
    content.set_margin_end(10);
    content.set_margin_bottom(12);

    build_terminal_section(&content, settings, &on_apply);
    build_appearance_section(&content, settings, &on_apply);
    build_browser_section(&content, settings, &on_apply, &on_clear_browser_data);
    build_shell_section(&content, settings, &on_apply);
    build_logr_section(&content, settings, &on_apply);
    build_about_section(&content, page_stack);

    let scroller = ScrolledWindow::new();
    scroller.set_vexpand(true);
    scroller.set_hexpand(true);
    scroller.set_child(Some(&content));
    scroller
}

fn build_shell_section(
    content: &GtkBox,
    settings: &Rc<RefCell<Settings>>,
    on_apply: &Rc<dyn Fn(&Settings)>,
) {
    content.append(&section_label("shell"));
    let shell_entry = text_row(
        content,
        "shell command",
        "the executable shell launched when a new obsidian session starts.",
        &settings.borrow().shell,
    );
    let shell_settings = settings.clone();
    let shell_apply = on_apply.clone();
    shell_entry.connect_changed(move |entry| {
        shell_settings.borrow_mut().shell = entry.text().to_string();
        preview_settings(&shell_settings, &shell_apply);
    });
}
fn build_logr_section(
    content: &GtkBox,
    settings: &Rc<RefCell<Settings>>,
    on_apply: &Rc<dyn Fn(&Settings)>,
) {
    content.append(&section_label("logr"));
    let logr_switch = switch_row(
        content,
        "panel open on start",
        "automatically reveal the logr and web pane when obsidian boots.",
        settings.borrow().logr_panel_open,
    );
    let logr_settings = settings.clone();
    let logr_apply = on_apply.clone();
    logr_switch.connect_state_set(move |_, active| {
        logr_settings.borrow_mut().logr_panel_open = active;
        preview_settings(&logr_settings, &logr_apply);
        glib::Propagation::Proceed
    });
}

fn build_about_section(content: &GtkBox, page_stack: &Stack) {
    content.append(&section_label("about"));
    info_row(content, "version", APP_VERSION);
    info_row(content, "config", &settings_path().display().to_string());

    let about_button = action_row(
        content,
        "obsidian",
        "view credits, licenses, and core engine details.",
        "open",
    );
    let stack_ref = page_stack.clone();
    about_button.connect_clicked(move |_| {
        stack_ref.set_visible_child_name("about");
    });
}
