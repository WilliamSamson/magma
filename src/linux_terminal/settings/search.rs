use gtk::{Box as GtkBox, Button, Entry, Label, Stack, Widget, prelude::*};

#[derive(Clone)]
pub(super) struct SearchSection {
    pub(super) id: &'static str,
    pub(super) title: &'static str,
    pub(super) button: Button,
    pub(super) content: GtkBox,
}

impl SearchSection {
    pub(super) fn new(
        id: &'static str,
        title: &'static str,
        button: &Button,
        content: &GtkBox,
    ) -> Self {
        Self {
            id,
            title,
            button: button.clone(),
            content: content.clone(),
        }
    }
}

pub(super) fn bind_settings_search(
    entry: &Entry,
    detail_stack: &Stack,
    empty_id: &'static str,
    sections: Vec<SearchSection>,
) {
    apply_search(detail_stack, empty_id, &sections, "");

    let stack_ref = detail_stack.clone();
    entry.connect_changed(move |entry| {
        let query = entry.text();
        apply_search(&stack_ref, empty_id, &sections, query.as_str());
    });
}

fn apply_search(detail_stack: &Stack, empty_id: &str, sections: &[SearchSection], query: &str) {
    let current = detail_stack.visible_child_name();
    let query = query.trim().to_lowercase();
    let mut matches = Vec::new();

    for section in sections {
        let section_match = query.is_empty() || contains_ignore_case(section.title, &query);
        let mut any_row_match = false;
        let mut child = section.content.first_child();
        let mut section_label: Option<Widget> = None;

        while let Some(widget) = child {
            child = widget.next_sibling();

            if widget.has_css_class("magma-settings-section") {
                section_label = Some(widget);
                continue;
            }

            let row_match = query.is_empty() || section_match || widget_matches(&widget, &query);
            widget.set_visible(row_match);
            any_row_match |= row_match;
        }

        let show_section = query.is_empty() || section_match || any_row_match;
        if let Some(label) = section_label {
            label.set_visible(show_section);
        }
        section.button.set_visible(show_section);

        if show_section {
            matches.push(section.id);
        }
    }

    if matches.is_empty() {
        detail_stack.set_visible_child_name(empty_id);
        return;
    }

    if current.as_deref() == Some(empty_id)
        || !matches.iter().any(|id| current.as_deref() == Some(*id))
    {
        detail_stack.set_visible_child_name(matches[0]);
    }
}

fn widget_matches(widget: &Widget, query: &str) -> bool {
    let mut text = String::new();
    collect_text(widget, &mut text);
    contains_ignore_case(&text, query)
}

fn collect_text(widget: &Widget, buffer: &mut String) {
    if let Ok(label) = widget.clone().downcast::<Label>() {
        buffer.push_str(label.text().as_str());
        buffer.push(' ');
    }

    if let Ok(button) = widget.clone().downcast::<Button>() {
        if let Some(text) = button.label() {
            buffer.push_str(text.as_str());
            buffer.push(' ');
        }
    }

    let mut child = widget.first_child();
    while let Some(current) = child {
        child = current.next_sibling();
        collect_text(&current, buffer);
    }
}

fn contains_ignore_case(text: &str, needle: &str) -> bool {
    text.to_lowercase().contains(needle)
}
