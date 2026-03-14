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
    panel.set_valign(Align::Start);
    panel.set_size_request(600, -1);
    panel.set_margin_top(40);
    panel.set_margin_bottom(40);

    let title = about_label("obsidian", "obsidian-settings-about-title");
    let name = about_label("X.R.1.9 - Kayode", "obsidian-settings-about-name");
    let meta = about_label(APP_VERSION, "obsidian-settings-about-meta");

    let summary = centered_copy(
        "A GPU-accelerated terminal workspace shaped around focus, compact control, and a quiet visual system.",
    );

    // Links Section
    let links_header = section_header("Links");
    let links_box = GtkBox::new(Orientation::Vertical, 6);
    links_box.add_css_class("obsidian-about-credits-box");
    links_box.append(&linked_label("GitHub", "https://github.com/WilliamSamson/obsidian"));

    // Credits Section
    let credits_header = section_header("Credits");
    let credits_box = GtkBox::new(Orientation::Vertical, 10);
    credits_box.add_css_class("obsidian-about-credits-box");

    let engine_box = GtkBox::new(Orientation::Vertical, 2);
    let engine_label = about_label("Engine", "obsidian-about-category-label");
    let engine_value = centered_copy("Rust \u{2022} GTK4 \u{2022} VTE4 \u{2022} WebKitGTK");
    engine_box.append(&engine_label);
    engine_box.append(&engine_value);

    let design_box = GtkBox::new(Orientation::Vertical, 2);
    let design_label = about_label("Design & Development", "obsidian-about-category-label");
    let design_value = linked_label("Kayode (@WilliamSamson)", "https://itskayode.web.app");
    design_box.append(&design_label);
    design_box.append(&design_value);

    credits_box.append(&engine_box);
    credits_box.append(&design_box);

    // License Section
    let license_header = section_header("License");
    let license_label = Label::new(None);
    license_label.set_markup(
        "Copyright \u{00a9} 2026 Kayode.\n\n\
         This program is free software: you can redistribute it and/or modify it under the terms \
         of the <a href=\"https://www.gnu.org/licenses/gpl-3.0.html\">GNU General Public License v3.0</a> \
         as published by the Free Software Foundation.\n\n\
         This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; \
         without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.",
    );
    license_label.add_css_class("obsidian-about-license-text");
    license_label.set_wrap(true);
    license_label.set_max_width_chars(60);
    license_label.set_justify(gtk::Justification::Center);

    panel.append(&title);
    panel.append(&name);
    panel.append(&meta);
    panel.append(&summary);

    panel.append(&links_header);
    panel.append(&links_box);

    panel.append(&credits_header);
    panel.append(&credits_box);

    panel.append(&license_header);
    panel.append(&license_label);

    page.append(&panel);
    page
}

fn section_header(text: &str) -> Label {
    let label = Label::new(Some(text));
    label.add_css_class("obsidian-about-section-header");
    label.set_margin_top(24);
    label.set_margin_bottom(8);
    label.set_halign(Align::Center);
    label
}

fn about_label(text: &str, class_name: &str) -> Label {
    let label = Label::new(Some(text));
    label.add_css_class(class_name);
    label.set_xalign(0.5);
    label.set_selectable(false);
    label
}

fn linked_label(text: &str, url: &str) -> Label {
    let label = Label::new(None);
    label.set_markup(&format!("<a href=\"{url}\">{text}</a>"));
    label.add_css_class("obsidian-about-link");
    label.set_xalign(0.5);
    label
}

fn centered_copy(text: &str) -> Label {
    let copy = body_copy(text);
    copy.set_xalign(0.5);
    copy.set_justify(gtk::Justification::Center);
    copy.set_max_width_chars(72);
    copy
}
