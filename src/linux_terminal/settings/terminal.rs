use std::{cell::RefCell, rc::Rc};

use gtk::{glib, prelude::*, Box as GtkBox};

use super::{
    save_settings,
    widgets::{dropdown_row, section_label, spin_row, switch_row, text_row},
    Settings,
};

pub(super) fn build_terminal_section(
    content: &GtkBox,
    settings: &Rc<RefCell<Settings>>,
    on_apply: &Rc<dyn Fn(&Settings)>,
) {
    content.append(&section_label("terminal"));

    let lines_spin = spin_row(
        content,
        "scrollback lines",
        "the maximum number of lines preserved in the terminal scrollback buffer.",
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

    let image_switch = switch_row(
        content,
        "image rendering",
        "enable terminal-side sixel graphics for supported image-capable CLI tools.",
        settings.borrow().image_rendering,
    );
    let image_settings = settings.clone();
    let image_apply = on_apply.clone();
    image_switch.connect_state_set(move |_, active| {
        image_settings.borrow_mut().image_rendering = active;
        preview_settings(&image_settings, &image_apply);
        glib::Propagation::Proceed
    });

    let ligature_switch = switch_row(
        content,
        "ligatures",
        "allow glyph shaping so supported fonts can render ligatures in terminal text.",
        settings.borrow().ligatures,
    );
    let ligature_settings = settings.clone();
    let ligature_apply = on_apply.clone();
    ligature_switch.connect_state_set(move |_, active| {
        ligature_settings.borrow_mut().ligatures = active;
        preview_settings(&ligature_settings, &ligature_apply);
        glib::Propagation::Proceed
    });

    let notify_switch = switch_row(
        content,
        "desktop notifications",
        "show system notifications when a command finishes or fails.",
        settings.borrow().notifications,
    );
    let notify_settings = settings.clone();
    let notify_apply = on_apply.clone();
    notify_switch.connect_state_set(move |_, active| {
        notify_settings.borrow_mut().notifications = active;
        preview_settings(&notify_settings, &notify_apply);
        glib::Propagation::Proceed
    });
}

pub(super) fn build_appearance_section(
    content: &GtkBox,
    settings: &Rc<RefCell<Settings>>,
    on_apply: &Rc<dyn Fn(&Settings)>,
) {
    content.append(&section_label("appearance"));

    let font_entry = text_row(
        content,
        "font family",
        "the monospaced typeface used for terminal rendering.",
        &settings.borrow().font_family,
    );
    let font_settings = settings.clone();
    let font_apply = on_apply.clone();
    font_entry.connect_changed(move |entry| {
        font_settings.borrow_mut().font_family = entry.text().to_string();
        preview_settings(&font_settings, &font_apply);
    });

    let size_spin = spin_row(
        content,
        "terminal font size",
        "the scale of the shell text output.",
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
        "the global scale of the application interface and window widgets.",
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

    let blink_switch = switch_row(
        content,
        "cursor blink",
        "enable or disable the blinking animation for the active cursor.",
        settings.borrow().cursor_blink,
    );
    let blink_settings = settings.clone();
    let blink_apply = on_apply.clone();
    blink_switch.connect_state_set(move |_, active| {
        blink_settings.borrow_mut().cursor_blink = active;
        preview_settings(&blink_settings, &blink_apply);
        glib::Propagation::Proceed
    });
}

pub(super) fn preview_settings(settings: &Rc<RefCell<Settings>>, on_apply: &Rc<dyn Fn(&Settings)>) {
    // Clone once so callbacks can apply a consistent snapshot without holding a RefCell borrow.
    let snapshot = settings.borrow().clone();
    save_settings(&snapshot);
    on_apply(&snapshot);
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
    let cursor_dropdown = dropdown_row(
        content,
        "cursor style",
        "the visual pattern used for the terminal cursor indicator.",
        &cursor_styles,
        selected,
    );
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
