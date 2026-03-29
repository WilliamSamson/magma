use std::os::fd::AsRawFd;

use gtk::{
    glib, prelude::*, Box as GtkBox, Label, Orientation,
};
use vte4::{prelude::*, Terminal};

use crate::linux_terminal::settings::Settings;

/// Builds a snapshot of the terminal inspector panel.
/// Each call returns a fresh widget tree with the current terminal state.
pub(in crate::linux_terminal) fn build_inspector_panel(terminal: &Terminal, settings: &Settings) -> GtkBox {
    let content = GtkBox::new(Orientation::Vertical, 10);
    content.add_css_class("magma-inspector-panel");

    let title = Label::new(Some("terminal inspector"));
    title.add_css_class("magma-inspector-title");
    title.set_xalign(0.0);
    content.append(&title);

    inspector_row_with_value(&content, "cwd", &display_uri(terminal.current_directory_uri()));
    inspector_row_with_value(&content, "file", &display_uri(terminal.current_file_uri()));
    inspector_row_with_value(
        &content,
        "title",
        terminal
            .window_title()
            .as_deref()
            .filter(|t| !t.is_empty())
            .unwrap_or("none"),
    );
    inspector_row_with_value(
        &content,
        "grid",
        &format!("{} cols \u{00d7} {} rows", terminal.column_count(), terminal.row_count()),
    );
    inspector_row_with_value(&content, "font", &display_font(terminal));
    inspector_row_with_value(&content, "pty", &display_pty(terminal));
    inspector_row_with_value(
        &content,
        "selection",
        if terminal.has_selection() { "active" } else { "none" },
    );
    inspector_row_with_value(
        &content,
        "image rendering",
        if settings.image_rendering { "enabled" } else { "disabled" },
    );
    inspector_row_with_value(
        &content,
        "ligatures",
        if settings.ligatures { "enabled" } else { "disabled" },
    );

    content
}

fn inspector_row_with_value(content: &GtkBox, key: &str, value: &str) {
    let row = GtkBox::new(Orientation::Vertical, 3);
    row.add_css_class("magma-inspector-row");

    let key_label = Label::new(Some(key));
    key_label.add_css_class("magma-inspector-key");
    key_label.set_xalign(0.0);

    let value_label = Label::new(Some(value));
    value_label.add_css_class("magma-inspector-value");
    value_label.set_xalign(0.0);
    value_label.set_wrap(true);
    value_label.set_selectable(false);

    row.append(&key_label);
    row.append(&value_label);
    content.append(&row);
}

fn display_uri(uri: Option<glib::GString>) -> String {
    uri.as_deref()
        .and_then(|uri| gtk::gio::File::for_uri(uri).path())
        .map(|path| path.display().to_string())
        .or_else(|| uri.map(|value| value.to_string()))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "none".to_string())
}

fn display_font(terminal: &Terminal) -> String {
    let desc = terminal
        .font_desc()
        .map(|font| font.to_string())
        .unwrap_or_else(|| "default".to_string());
    format!("{desc} \u{00b7} scale {:.2}", terminal.font_scale())
}

fn display_pty(terminal: &Terminal) -> String {
    terminal
        .pty()
        .map(|pty| format!("attached \u{00b7} fd {}", pty.fd().as_raw_fd()))
        .unwrap_or_else(|| "detached".to_string())
}
