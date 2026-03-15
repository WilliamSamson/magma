use gtk::{
    prelude::*, Box as GtkBox, Button, Image, Label, ListBox, ListBoxRow, Orientation,
};

use super::files::{format_size, kind_label, FileKind, ViewerFile};

pub(super) fn build_header(refresh_button: &Button, count_label: &Label) -> GtkBox {
    let header = GtkBox::new(Orientation::Horizontal, 6);
    header.add_css_class("magma-view-header");

    let title_block = GtkBox::new(Orientation::Vertical, 2);
    title_block.add_css_class("magma-view-heading");
    title_block.set_hexpand(true);

    let title = Label::new(Some("view"));
    title.add_css_class("magma-view-title");
    title.set_xalign(0.0);
    count_label.set_xalign(0.0);

    title_block.append(&title);
    title_block.append(count_label);
    header.append(&title_block);

    let spacer = GtkBox::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    header.append(&spacer);
    refresh_button.add_css_class("magma-view-header-action");
    header.append(refresh_button);
    header
}

pub(super) fn build_empty_state(icon_name: &str, text: &str) -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 12);
    root.add_css_class("magma-view-empty-state");
    root.set_vexpand(true);
    root.set_valign(gtk::Align::Center);

    let icon = Image::builder()
        .icon_name(icon_name)
        .pixel_size(48)
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
    let content = GtkBox::new(Orientation::Horizontal, 8);
    content.add_css_class("magma-view-file-card");
    let icon = Image::from_icon_name(file_icon(file.kind));
    icon.add_css_class("magma-view-file-icon");
    let title = Label::new(Some(&file.name));
    title.add_css_class("magma-view-file-name");
    title.set_xalign(0.0);
    title.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
    let meta = Label::new(Some(&format!(
        "{} · {}",
        kind_label(file.kind),
        format_size(file.size_bytes)
    )));
    meta.add_css_class("magma-view-file-meta");
    meta.set_xalign(0.0);
    let text = GtkBox::new(Orientation::Vertical, 1);
    text.set_hexpand(true);
    text.append(&title);
    text.append(&meta);
    content.append(&icon);
    content.append(&text);
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
