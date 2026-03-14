mod history;
mod inspector;
mod prompt;
mod status;

use std::{cell::Cell, path::Path, rc::Rc};

use gtk::{
    gdk, prelude::*, Box as GtkBox, Button, Entry, EventControllerFocus, EventControllerKey,
    Orientation, Separator,
};
use vte4::{prelude::*, Regex, Terminal};

use self::{
    history::{record_history, InputHistory},
    prompt::build_prompt_box,
    status::build_status_widgets,
};
use crate::linux_terminal::settings::Settings;

/// Widgets that toggle visibility between command and search mode.
#[derive(Clone)]
struct SearchWidgets {
    prompt: GtkBox,
    status: gtk::Label,
    notice: gtk::Label,
    prev_btn: Button,
    next_btn: Button,
}

pub(super) fn append_input_row(
    container: &GtkBox,
    terminal: &Terminal,
    status_path: &Path,
    settings: &Rc<std::cell::RefCell<Settings>>,
) -> Entry {
    let separator = Separator::new(Orientation::Horizontal);
    separator.add_css_class("obsidian-separator");
    container.append(&separator);

    let input_container = GtkBox::new(Orientation::Horizontal, 8);
    input_container.add_css_class("obsidian-input-pill");

    let prompt_container = build_prompt_box(terminal);
    input_container.append(&prompt_container);

    let notifications = Rc::new(Cell::new(settings.borrow().notifications));
    let status_widgets = build_status_widgets(status_path, notifications.clone());
    input_container.append(&status_widgets.status);
    input_container.append(&status_widgets.notice);

    let entry = Entry::new();
    entry.add_css_class("obsidian-entry");
    entry.set_hexpand(true);
    entry.set_placeholder_text(Some("Enter command"));
    input_container.append(&entry);

    let prev_button = tool_button("go-up-symbolic");
    let next_button = tool_button("go-down-symbolic");
    prev_button.set_visible(false);
    next_button.set_visible(false);
    input_container.append(&prev_button);
    input_container.append(&next_button);

    let inspector_button = inspector::build_inspector_button(terminal, settings.clone());
    input_container.append(&inspector_button);

    let search_button = Button::builder()
        .icon_name("system-search-symbolic")
        .css_classes(["obsidian-search-toggle"])
        .build();
    input_container.append(&search_button);

    container.append(&input_container);

    let search_mode = Rc::new(Cell::new(false));
    let history = Rc::new(InputHistory::default());
    let sw = SearchWidgets {
        prompt: prompt_container,
        status: status_widgets.status,
        notice: status_widgets.notice,
        prev_btn: prev_button,
        next_btn: next_button,
    };

    wire_activate(&entry, terminal, &history, &search_mode);
    wire_keys(&entry, terminal, &history, &search_mode, &sw);
    wire_search_on_change(&entry, terminal, &search_mode);
    wire_search_toggle(&search_button, &entry, terminal, &search_mode, &sw);
    wire_search_nav(terminal, &sw.prev_btn, &sw.next_btn);
    wire_focus_tracking(&input_container, &entry, terminal);

    let entry_ref = entry.clone();
    gtk::glib::idle_add_local_once(move || {
        let _ = entry_ref.grab_focus();
    });

    entry
}

pub(super) fn handle_terminal_clipboard_shortcuts(
    terminal: &Terminal,
    key: gdk::Key,
    modifiers: gdk::ModifierType,
) -> bool {
    history::handle_terminal_clipboard_shortcuts(terminal, key, modifiers)
}

fn tool_button(icon_name: &str) -> Button {
    Button::builder()
        .icon_name(icon_name)
        .css_classes(["obsidian-tool-button"])
        .build()
}

// --- Activate (Enter key) ---

fn wire_activate(
    entry: &Entry,
    terminal: &Terminal,
    history: &Rc<InputHistory>,
    search_mode: &Rc<Cell<bool>>,
) {
    let terminal = terminal.clone();
    let history = history.clone();
    let search_mode = search_mode.clone();

    entry.connect_activate(move |entry| {
        let text = entry.text();

        if search_mode.get() {
            if !text.is_empty() {
                let _ = terminal.search_find_next();
            }
            return;
        }

        record_history(&history, text.as_str());
        let mut input = text.to_string();
        input.push('\n');
        terminal.feed_child(input.as_bytes());
        entry.set_text("");
    });
}

// --- Keyboard handler ---

fn wire_keys(
    entry: &Entry,
    terminal: &Terminal,
    history: &Rc<InputHistory>,
    search_mode: &Rc<Cell<bool>>,
    sw: &SearchWidgets,
) {
    let controller = EventControllerKey::new();

    let entry_ref = entry.clone();
    let terminal_ref = terminal.clone();
    let history_ref = history.clone();
    let search_mode_ref = search_mode.clone();
    let sw = sw.clone();

    controller.connect_key_pressed(move |_, key, _, modifiers| {
        let ctrl = modifiers.contains(gdk::ModifierType::CONTROL_MASK);
        let shift = modifiers.contains(gdk::ModifierType::SHIFT_MASK);
        let alt = modifiers.contains(gdk::ModifierType::ALT_MASK);

        if ctrl && !shift && !alt && matches!(key.to_unicode(), Some('f') | Some('F')) {
            let entering = !search_mode_ref.get();
            set_search_mode(entering, &search_mode_ref, &entry_ref, &terminal_ref, &sw);
            return gtk::glib::Propagation::Stop;
        }

        if key == gdk::Key::Escape {
            if search_mode_ref.get() {
                set_search_mode(false, &search_mode_ref, &entry_ref, &terminal_ref, &sw);
            } else {
                let _ = terminal_ref.grab_focus();
            }
            return gtk::glib::Propagation::Stop;
        }

        if history::handle_clipboard_shortcuts(&entry_ref, &terminal_ref, key, modifiers) {
            return gtk::glib::Propagation::Stop;
        }

        if !search_mode_ref.get()
            && history::handle_history_navigation(&entry_ref, &history_ref, key)
        {
            return gtk::glib::Propagation::Stop;
        }

        gtk::glib::Propagation::Proceed
    });

    entry.add_controller(controller);
}

// --- Focus tracking ---

fn wire_focus_tracking(input_container: &GtkBox, entry: &Entry, terminal: &Terminal) {
    let terminal_focus = EventControllerFocus::new();
    {
        let pill = input_container.clone();
        terminal_focus.connect_enter(move |_| {
            pill.add_css_class("terminal-active");
        });
    }
    {
        let pill = input_container.clone();
        terminal_focus.connect_leave(move |_| {
            pill.remove_css_class("terminal-active");
        });
    }
    terminal.add_controller(terminal_focus);

    let terminal_keys = EventControllerKey::new();
    {
        let entry_ref = entry.clone();
        let terminal_ref = terminal.clone();
        terminal_keys.connect_key_pressed(move |_, key, _, modifiers| {
            if history::handle_terminal_clipboard_shortcuts(&terminal_ref, key, modifiers) {
                return gtk::glib::Propagation::Stop;
            }
            if key == gdk::Key::Escape {
                let _ = entry_ref.grab_focus();
                return gtk::glib::Propagation::Stop;
            }
            gtk::glib::Propagation::Proceed
        });
    }
    terminal.add_controller(terminal_keys);
}

// --- Search mode ---

fn set_search_mode(
    enabled: bool,
    search_mode: &Rc<Cell<bool>>,
    entry: &Entry,
    terminal: &Terminal,
    sw: &SearchWidgets,
) {
    search_mode.set(enabled);

    if let Some(pill) = entry.parent().and_then(|p| p.downcast::<GtkBox>().ok()) {
        if enabled {
            pill.add_css_class("search-active");
        } else {
            pill.remove_css_class("search-active");
        }
    }

    if enabled {
        entry.set_placeholder_text(Some("Search output..."));
        entry.add_css_class("search-active");
        sw.prompt.set_visible(false);
        sw.status.set_visible(false);
        sw.notice.set_visible(false);
        sw.prev_btn.set_visible(true);
        sw.next_btn.set_visible(true);
        apply_search(terminal, entry.text().as_str());
    } else {
        entry.set_placeholder_text(Some("Enter command"));
        entry.remove_css_class("search-active");
        sw.prompt.set_visible(true);
        sw.status.set_visible(true);
        sw.prev_btn.set_visible(false);
        sw.next_btn.set_visible(false);
        terminal.search_set_regex(None, 0);
    }

    entry.set_text("");
    let _ = entry.grab_focus_without_selecting();
}

fn wire_search_toggle(
    search_button: &Button,
    entry: &Entry,
    terminal: &Terminal,
    search_mode: &Rc<Cell<bool>>,
    sw: &SearchWidgets,
) {
    let entry = entry.clone();
    let terminal = terminal.clone();
    let search_mode = search_mode.clone();
    let sw = sw.clone();

    search_button.connect_clicked(move |_| {
        let entering = !search_mode.get();
        set_search_mode(entering, &search_mode, &entry, &terminal, &sw);
    });
}

fn wire_search_on_change(entry: &Entry, terminal: &Terminal, search_mode: &Rc<Cell<bool>>) {
    let terminal = terminal.clone();
    let search_mode = search_mode.clone();

    entry.connect_changed(move |entry| {
        if search_mode.get() {
            apply_search(&terminal, entry.text().as_str());
        }
    });
}

fn apply_search(terminal: &Terminal, query: &str) {
    if query.is_empty() {
        terminal.search_set_regex(None, 0);
        return;
    }

    match Regex::for_search(query, 0) {
        Ok(regex) => {
            terminal.search_set_wrap_around(true);
            terminal.search_set_regex(Some(&regex), 0);
            let _ = terminal.search_find_next();
        }
        Err(error) => {
            eprintln!("search regex invalid: {error}");
        }
    }
}

fn wire_search_nav(terminal: &Terminal, prev_button: &Button, next_button: &Button) {
    let t = terminal.clone();
    prev_button.connect_clicked(move |_| {
        let _ = t.search_find_previous();
    });

    let t = terminal.clone();
    next_button.connect_clicked(move |_| {
        let _ = t.search_find_next();
    });
}
