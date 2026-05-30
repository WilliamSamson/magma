use gtk::{Box as GtkBox, Button, Image, Label, ListBox, ListBoxRow, Orientation, prelude::*};

use super::files::{FileKind, ViewerFile, format_size, kind_label};

pub(super) fn build_header(refresh_button: &Button, count_label: &Label) -> GtkBox {
    let header = GtkBox::new(Orientation::Horizontal, 6);
    header.add_css_class("magma-view-header");

    let title = Label::new(Some("VIEW"));
    title.add_css_class("magma-view-title");
    title.set_xalign(0.0);
    header.append(&title);

    count_label.set_xalign(1.0);
    count_label.set_hexpand(true);
    header.append(count_label);

    refresh_button.add_css_class("magma-view-header-action");
    header.append(refresh_button);
    header
}

pub(super) fn build_empty_state(icon_name: &str, text: &str) -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 6);
    root.add_css_class("magma-view-empty-state");
    root.set_vexpand(true);
    root.set_valign(gtk::Align::Center);

    let icon = Image::builder()
        .icon_name(icon_name)
        .pixel_size(24)
        .css_classes(["magma-view-empty-icon"])
        .build();

    let label = Label::new(Some(text));
    label.add_css_class("magma-view-empty-text");
    label.set_justify(gtk::Justification::Center);
    label.set_wrap(true);

    root.append(&icon);
    root.append(&label);
    root
}

pub(super) fn file_row(file: &ViewerFile) -> ListBoxRow {
    let row = ListBoxRow::new();
    row.add_css_class("magma-view-file-row");
    let content = GtkBox::new(Orientation::Horizontal, 6);
    content.add_css_class("magma-view-file-card");
    let icon = Image::from_icon_name(file_icon(file.kind));
    icon.add_css_class("magma-view-file-icon");
    let title = Label::new(Some(&file.name));
    title.add_css_class("magma-view-file-name");
    title.set_xalign(0.0);
    title.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
    title.set_hexpand(true);
    let meta = Label::new(Some(&format!(
        "{} · {}",
        kind_label(file.kind),
        format_size(file.size_bytes)
    )));
    meta.add_css_class("magma-view-file-meta");
    meta.set_xalign(1.0);
    content.append(&icon);
    content.append(&title);
    content.append(&meta);
    row.set_child(Some(&content));
    row
}

pub(super) fn icon_button(icon_name: &str, tooltip: &str) -> Button {
    Button::builder()
        .icon_name(icon_name)
        .tooltip_text(tooltip)
        .css_classes(["magma-view-action"])
        .build()
}

pub(super) fn clear_list(list: &ListBox) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
}

fn file_icon(kind: FileKind) -> &'static str {
    match kind {
        FileKind::Image => "image-x-generic-symbolic",
        FileKind::Pdf => "application-pdf-symbolic",
        FileKind::Docx => "x-office-document-symbolic",
        FileKind::Code => "text-x-script-symbolic",
        FileKind::Office => "x-office-document-symbolic",
    }
}
