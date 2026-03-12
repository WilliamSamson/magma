use gtk::{prelude::*, Box as GtkBox, Entry, Label, Orientation, Separator};
use vte4::{prelude::*, Terminal};

pub(super) fn append_input_row(container: &GtkBox, terminal: &Terminal) -> Entry {
    let separator = Separator::new(Orientation::Horizontal);
    separator.add_css_class("obsidian-separator");
    container.append(&separator);

    let input_container = GtkBox::new(Orientation::Horizontal, 8);
    input_container.add_css_class("obsidian-input-pill");

    let username = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
    let prompt_label = Label::new(Some(&format!("{username} >")));
    prompt_label.add_css_class("obsidian-prompt-label");
    input_container.append(&prompt_label);

    let entry = Entry::new();
    entry.add_css_class("obsidian-entry");
    entry.set_hexpand(true);
    entry.set_placeholder_text(Some("Enter command"));

    let terminal_clone = terminal.clone();
    entry.connect_activate(move |entry| {
        let text = entry.text();
        if text.is_empty() {
            return;
        }

        let mut input = text.to_string();
        input.push('\n');
        terminal_clone.feed_child(input.as_bytes());
        entry.set_text("");
        let _ = entry.grab_focus_without_selecting();
    });

    input_container.append(&entry);
    container.append(&input_container);
    entry
}
