use gtk::{
    prelude::*, Align, Box as GtkBox, Button, DropDown, Entry, Label, Orientation, SpinButton,
    StringList, Switch,
};

pub(super) fn section_label(text: &str) -> Label {
    let label = Label::new(Some(text));
    label.add_css_class("obsidian-settings-section");
    label.set_xalign(0.0);
    label
}

pub(super) fn text_row(parent: &GtkBox, label_text: &str, description: &str, value: &str) -> Entry {
    let entry = Entry::new();
    entry.set_text(value);
    entry.add_css_class("obsidian-settings-entry");
    entry.set_hexpand(true);
    entry.set_halign(Align::Fill);

    let row = setting_row(parent, label_text, description);
    row.append(&entry);
    entry
}

pub(super) fn spin_row(
    parent: &GtkBox,
    label_text: &str,
    description: &str,
    value: f64,
    min: f64,
    max: f64,
) -> SpinButton {
    let spin = SpinButton::with_range(min, max, 1.0);
    spin.set_value(value);
    spin.add_css_class("obsidian-settings-spin");
    spin.set_halign(Align::End);

    let row = setting_row(parent, label_text, description);
    row.append(&spin);
    spin
}

pub(super) fn switch_row(parent: &GtkBox, label_text: &str, description: &str, active: bool) -> Switch {
    let switch = Switch::new();
    switch.set_active(active);
    switch.add_css_class("obsidian-settings-switch");
    switch.set_halign(Align::End);
    switch.set_valign(Align::Center);

    let row = setting_row(parent, label_text, description);
    row.append(&switch);
    switch
}

pub(super) fn dropdown_row(
    parent: &GtkBox,
    label_text: &str,
    description: &str,
    items: &[&str],
    active: u32,
) -> DropDown {
    let model = StringList::new(items);
    let dropdown = DropDown::builder().model(&model).selected(active).build();
    dropdown.add_css_class("obsidian-settings-dropdown");
    dropdown.set_halign(Align::End);

    let row = setting_row(parent, label_text, description);
    row.append(&dropdown);
    dropdown
}

pub(super) fn info_row(parent: &GtkBox, label_text: &str, value: &str) {
    let row = setting_row(parent, label_text, "");
    let value_label = value_label(value);
    value_label.set_xalign(1.0);
    value_label.set_halign(Align::End);
    row.append(&value_label);
}

pub(super) fn action_row(parent: &GtkBox, label_text: &str, description: &str, action_text: &str) -> Button {
    let button = Button::builder()
        .label(action_text)
        .css_classes(["obsidian-settings-link"])
        .build();
    button.set_halign(Align::End);

    let row = setting_row(parent, label_text, description);
    row.append(&button);
    button
}

pub(super) fn body_copy(text: &str) -> Label {
    let copy = Label::new(Some(text));
    copy.add_css_class("obsidian-settings-about-copy");
    copy.set_xalign(0.0);
    copy.set_wrap(true);
    copy.set_selectable(false);
    copy
}

fn setting_row(parent: &GtkBox, label_text: &str, description: &str) -> GtkBox {
    let container = GtkBox::new(Orientation::Vertical, 10);
    container.add_css_class("obsidian-settings-row");

    let header = GtkBox::new(Orientation::Horizontal, 12);
    
    let title = Label::new(Some(label_text));
    title.add_css_class("obsidian-settings-label");
    title.set_xalign(0.0);
    title.set_hexpand(true);
    header.append(&title);
    
    container.append(&header);

    if !description.is_empty() {
        let copy = Label::new(Some(description));
        copy.add_css_class("obsidian-settings-copy");
        copy.set_xalign(0.0);
        copy.set_wrap(true);
        container.append(&copy);
    }

    parent.append(&container);
    header // Return the header box where the control will be appended
}

fn value_label(value: &str) -> Label {
    let label = Label::new(Some(value));
    label.add_css_class("obsidian-settings-value");
    label.set_wrap(true);
    label.set_selectable(false);
    label
}
