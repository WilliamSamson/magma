use std::{cell::RefCell, rc::Rc};

use gtk::{glib, prelude::*, Box as GtkBox, Orientation, ScrolledWindow, Stack};

use crate::linux_terminal::meta::APP_VERSION;

use super::{
    browser::build_browser_section,
    save_settings, settings_path, Settings,
    widgets::{
        action_row, dropdown_row, info_row, section_label, spin_row, switch_row, text_row,
    },
};

pub(super) fn build_main_page(
    page_stack: &Stack,
    settings: &Rc<RefCell<Settings>>,
    on_apply: Rc<dyn Fn(&Settings)>,
) -> ScrolledWindow {
    let content = GtkBox::new(Orientation::Vertical, 0);
    content.add_css_class("obsidian-settings-content");
    content.set_margin_end(10);
    content.set_margin_bottom(12);

    build_terminal_section(&content, settings, &on_apply);
    build_appearance_section(&content, settings, &on_apply);
    build_browser_section(&content, settings, &on_apply);
    build_shell_section(&content, settings, &on_apply);
    build_logr_section(&content, settings, &on_apply);
    build_about_section(&content, page_stack);

    let scroller = ScrolledWindow::new();
    scroller.set_vexpand(true);
    scroller.set_hexpand(true);
    scroller.set_child(Some(&content));
    scroller
}

fn build_terminal_section(
    content: &GtkBox,
    settings: &Rc<RefCell<Settings>>,
    on_apply: &Rc<dyn Fn(&Settings)>,
) {
    content.append(&section_label("terminal"));

    let lines_spin = spin_row(
        content,
        "scrollback lines",
        settings.borrow().scrollback_lines as f64,
        500.0,
        100_000.0,
    );
    lines_spin.set_digits(0);
    lines_spin.set_increments(500.0, 5_000.0);
    let lines_settings = settings.clone();
    let lines_apply = on_apply.clone();
    lines_spin.connect_value_changed(move |spin| {
        lines_settings.borrow_mut().scrollback_lines = spin.value() as u32;
        preview_settings(&lines_settings, &lines_apply);
    });
}

fn bind_cursor_style(
    content: &GtkBox,
    settings: &Rc<RefCell<Settings>>,
    on_apply: &Rc<dyn Fn(&Settings)>,
) {
    let cursor_styles = ["ibeam", "block", "underline"];
    let selected = cursor_styles
        .iter()
        .position(|style| *style == settings.borrow().cursor_style)
        .unwrap_or(0) as u32;
    let cursor_dropdown = dropdown_row(content, "cursor style", &cursor_styles, selected);
    let cursor_settings = settings.clone();
    let cursor_apply = on_apply.clone();
    cursor_dropdown.connect_selected_notify(move |dropdown| {
        let cursor_style = match dropdown.selected() {
            1 => "block",
            2 => "underline",
            _ => "ibeam",
        };
        cursor_settings.borrow_mut().cursor_style = cursor_style.to_string();
        preview_settings(&cursor_settings, &cursor_apply);
    });
}

fn build_appearance_section(
    content: &GtkBox,
    settings: &Rc<RefCell<Settings>>,
    on_apply: &Rc<dyn Fn(&Settings)>,
) {
    content.append(&section_label("appearance"));

    let font_entry = text_row(content, "font family", &settings.borrow().font_family);
    let font_settings = settings.clone();
    let font_apply = on_apply.clone();
    font_entry.connect_changed(move |entry| {
        font_settings.borrow_mut().font_family = entry.text().to_string();
        preview_settings(&font_settings, &font_apply);
    });

    let size_spin = spin_row(
        content,
        "terminal font size",
        settings.borrow().font_size as f64,
        6.0,
        32.0,
    );
    let size_settings = settings.clone();
    let size_apply = on_apply.clone();
    size_spin.connect_value_changed(move |spin| {
        size_settings.borrow_mut().font_size = spin.value() as u32;
        preview_settings(&size_settings, &size_apply);
    });

    let app_size_spin = spin_row(
        content,
        "app font size",
        settings.borrow().app_font_size as f64,
        8.0,
        20.0,
    );
    let app_size_settings = settings.clone();
    let app_size_apply = on_apply.clone();
    app_size_spin.connect_value_changed(move |spin| {
        app_size_settings.borrow_mut().app_font_size = spin.value() as u32;
        preview_settings(&app_size_settings, &app_size_apply);
    });

    bind_cursor_style(content, settings, on_apply);

    let blink_switch = switch_row(content, "cursor blink", settings.borrow().cursor_blink);
    let blink_settings = settings.clone();
    let blink_apply = on_apply.clone();
    blink_switch.connect_state_set(move |_, active| {
        blink_settings.borrow_mut().cursor_blink = active;
        preview_settings(&blink_settings, &blink_apply);
        glib::Propagation::Proceed
    });
}

fn build_shell_section(
    content: &GtkBox,
    settings: &Rc<RefCell<Settings>>,
    on_apply: &Rc<dyn Fn(&Settings)>,
) {
    content.append(&section_label("shell"));
    let shell_entry = text_row(content, "shell", &settings.borrow().shell);
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
    let logr_switch = switch_row(content, "panel open on start", settings.borrow().logr_panel_open);
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

    let about_button = action_row(content, "obsidian", "open");
    let stack_ref = page_stack.clone();
    about_button.connect_clicked(move |_| {
        stack_ref.set_visible_child_name("about");
    });
}

pub(super) fn preview_settings(settings: &Rc<RefCell<Settings>>, on_apply: &Rc<dyn Fn(&Settings)>) {
    // Clone once so callbacks can apply a consistent snapshot without holding a RefCell borrow.
    let snapshot = settings.borrow().clone();
    save_settings(&snapshot);
    on_apply(&snapshot);
}
