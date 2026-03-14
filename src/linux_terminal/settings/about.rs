use gtk::{prelude::*, Align, Box as GtkBox, Label, Orientation};

use crate::linux_terminal::meta::APP_VERSION;

use super::widgets::body_copy;

pub(super) fn build_about_page() -> GtkBox {
    let page = GtkBox::new(Orientation::Vertical, 0);
    page.add_css_class("obsidian-settings-about-page");
    page.set_vexpand(true);
    page.set_hexpand(true);
    page.set_halign(Align::Center);
    page.set_valign(Align::Center);

    let panel = GtkBox::new(Orientation::Vertical, 12);
    panel.add_css_class("obsidian-settings-about-panel");
    panel.set_halign(Align::Center);
    panel.set_valign(Align::Center);
    panel.set_size_request(640, -1);

    let title = about_label("obsidian", "obsidian-settings-about-title");
    let name = about_label("X.R.1.9 - Kayode", "obsidian-settings-about-name");
    let meta = about_label(APP_VERSION, "obsidian-settings-about-meta");

    let summary = centered_copy(
        "Obsidian is a stripped terminal workspace shaped around focus, compact control, and a quiet visual system.",
    );
    let developer = centered_copy(
        "Designed and developed by Kayode as a terminal interface where the shell stays primary and the chrome stays disciplined.",
    );

    panel.append(&title);
    panel.append(&name);
    panel.append(&meta);
    panel.append(&summary);
    panel.append(&developer);
    page.append(&panel);
    page
}

fn about_label(text: &str, class_name: &str) -> Label {
    let label = Label::new(Some(text));
    label.add_css_class(class_name);
    label.set_xalign(0.5);
    label.set_selectable(false);
    label
}

fn centered_copy(text: &str) -> Label {
    let copy = body_copy(text);
    copy.set_xalign(0.5);
    copy.set_justify(gtk::Justification::Center);
    copy.set_max_width_chars(72);
    copy
}
