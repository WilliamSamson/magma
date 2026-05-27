#![allow(dead_code)]
use std::{cell::RefCell, rc::Rc};

use gtk::{gdk, gio, prelude::*, Entry};
use vte4::{prelude::*, Format, Terminal};

#[derive(Default)]
pub(super) struct InputHistory {
    state: RefCell<InputHistoryState>,
}

#[derive(Default)]
struct InputHistoryState {
    commands: Vec<String>,
    cursor: Option<usize>,
    draft: String,
}

pub(super) fn handle_clipboard_shortcuts(
    entry: &Entry,
    terminal: &Terminal,
    key: gdk::Key,
    modifiers: gdk::ModifierType,
) -> bool {
    if handle_terminal_clipboard_shortcuts(terminal, key, modifiers) {
        return true;
    }

    let ctrl = modifiers.contains(gdk::ModifierType::CONTROL_MASK);
    let shift = modifiers.contains(gdk::ModifierType::SHIFT_MASK);
    let alt = modifiers.contains(gdk::ModifierType::ALT_MASK);

    // Ctrl+Shift combos: terminal clipboard operations
    if ctrl && shift && !alt {
        return match key.to_unicode() {
            Some('c') | Some('C') if terminal.has_selection() => {
                terminal.copy_clipboard_format(Format::Text);
                true
            }
            Some('v') | Some('V') => {
                paste_clipboard_into_entry(entry);
                true
            }
            _ => false,
        };
    }

    // Ctrl-only: forward terminal control characters
    if ctrl && !shift && !alt {
        if let Some(ctrl_byte) = ctrl_key_byte(key) {
            terminal.feed_child(&[ctrl_byte]);
            // Clear entry for signals that interrupt/kill the current input
            if matches!(ctrl_byte, 0x03 | 0x1A | 0x04) {
                entry.set_text("");
            }
            return true;
        }
    }

    false
}

pub(super) fn handle_terminal_clipboard_shortcuts(
    terminal: &Terminal,
    key: gdk::Key,
    modifiers: gdk::ModifierType,
) -> bool {
    let ctrl = modifiers.contains(gdk::ModifierType::CONTROL_MASK);
    let shift = modifiers.contains(gdk::ModifierType::SHIFT_MASK);
    let alt = modifiers.contains(gdk::ModifierType::ALT_MASK);

    if ctrl && shift && !alt {
        return match key.to_unicode() {
            Some('c') | Some('C') if terminal.has_selection() => {
                terminal.copy_clipboard_format(Format::Text);
                true
            }
            Some('v') | Some('V') => {
                terminal.paste_clipboard();
                true
            }
            Some('a') | Some('A') => {
                terminal.select_all();
                true
            }
            _ => false,
        };
    }

    if shift && !ctrl && !alt && key == gdk::Key::Insert {
        terminal.paste_clipboard();
        return true;
    }

    false
}

/// Map a key pressed with Ctrl to its terminal control byte.
/// Only includes shortcuts that make sense for the entry→terminal model.
/// Excludes Ctrl+W/T (workspace shortcuts) and Ctrl+A/E (GTK Entry defaults).
fn ctrl_key_byte(key: gdk::Key) -> Option<u8> {
    match key.to_unicode() {
        Some('c') | Some('C') => Some(0x03), // ETX  — SIGINT
        Some('z') | Some('Z') => Some(0x1A), // SUB  — SIGTSTP
        Some('d') | Some('D') => Some(0x04), // EOT  — EOF
        Some('l') | Some('L') => Some(0x0C), // FF   — clear screen
        Some('\\')             => Some(0x1C), // FS   — SIGQUIT
        _ => None,
    }
}

pub(super) fn handle_history_navigation(entry: &Entry, history: &Rc<InputHistory>, key: gdk::Key) -> bool {
    match key {
        gdk::Key::Up => {
            show_previous_history(entry, history);
            true
        }
        gdk::Key::Down => {
            show_next_history(entry, history);
            true
        }
        _ => false,
    }
}

pub(super) fn record_history(history: &Rc<InputHistory>, command: &str) {
    let mut history = history.state.borrow_mut();
    if history.commands.last().is_some_and(|last| last == command) {
        history.cursor = None;
        history.draft.clear();
        return;
    }

    history.commands.push(command.to_string());
    history.cursor = None;
    history.draft.clear();
}

pub(super) fn show_previous_history(entry: &Entry, history: &Rc<InputHistory>) {
    let mut history = history.state.borrow_mut();
    if history.commands.is_empty() {
        return;
    }

    if history.cursor.is_none() {
        history.draft = entry.text().to_string();
        history.cursor = Some(history.commands.len().saturating_sub(1));
    } else if let Some(cursor) = history.cursor {
        history.cursor = Some(cursor.saturating_sub(1));
    }

    if let Some(cursor) = history.cursor {
        entry.set_text(&history.commands[cursor]);
        entry.set_position(-1);
    }
}

pub(super) fn show_next_history(entry: &Entry, history: &Rc<InputHistory>) {
    let mut history = history.state.borrow_mut();
    let Some(cursor) = history.cursor else {
        return;
    };

    if cursor + 1 >= history.commands.len() {
        history.cursor = None;
        entry.set_text(&history.draft);
        entry.set_position(-1);
        return;
    }

    history.cursor = Some(cursor + 1);
    if let Some(next_cursor) = history.cursor {
        entry.set_text(&history.commands[next_cursor]);
        entry.set_position(-1);
    }
}

fn paste_clipboard_into_entry(entry: &Entry) {
    let Some(display) = gdk::Display::default() else {
        return;
    };

    let clipboard = display.clipboard();
    let entry = entry.clone();
    clipboard.read_text_async(None::<&gio::Cancellable>, move |result| {
        let Ok(Some(text)) = result else {
            return;
        };

        let mut combined = entry.text().to_string();
        combined.push_str(text.as_str());
        entry.set_text(&combined);
        entry.set_position(-1);
    });
}
