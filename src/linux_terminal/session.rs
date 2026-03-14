use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use gtk::{
    gdk, gio, prelude::*, Box as GtkBox, EventControllerFocus, EventControllerKey, Orientation,
};
use vte4::{prelude::*, CursorBlinkMode, CursorShape, Format, Terminal};

use super::{
    input,
    persist::SessionSnapshot,
    profile::{profile, ProfileId},
    settings::Settings,
    shell,
    terminal,
};

pub(super) struct SessionView {
    root: GtkBox,
    terminal: Terminal,
    snapshot: SessionSnapshot,
}

impl SessionView {
    pub(super) fn new(
        profile_id: ProfileId,
        snapshot: &SessionSnapshot,
        settings: Rc<RefCell<Settings>>,
    ) -> Self {
        let root = GtkBox::new(Orientation::Vertical, 8);
        root.set_hexpand(true);
        root.set_vexpand(true);

        let snapshot = snapshot.clone().normalized();
        let settings_ref = settings.borrow();
        let terminal = terminal::build_terminal(profile_id, &settings_ref);
        let runtime = shell::spawn_shell(&terminal, &snapshot, &settings_ref.shell);
        drop(settings_ref);

        root.append(&terminal);
        let _ = input::append_input_row(&root, &terminal, runtime.status_path(), &settings);
        wire_terminal_clipboard(&terminal);

        Self {
            root,
            terminal,
            snapshot,
        }
    }

    pub(super) fn root(&self) -> &GtkBox {
        &self.root
    }

    pub(super) fn focus_terminal(&self) {
        self.terminal.grab_focus();
    }

    pub(super) fn connect_focus_enter(&self, on_focus: impl Fn() + 'static) {
        let controller = EventControllerFocus::new();
        controller.connect_enter(move |_| on_focus());
        self.terminal.add_controller(controller);
    }

    pub(super) fn current_cwd(&self) -> Option<String> {
        self.terminal
            .current_directory_uri()
            .as_deref()
            .and_then(|uri| gio::File::for_uri(uri).path())
            .map(|path| path.display().to_string())
    }

    pub(super) fn to_snapshot(&self) -> SessionSnapshot {
        let mut snapshot = self.snapshot.clone();
        snapshot.cwd = self.current_cwd().or_else(|| snapshot.cwd.clone());
        snapshot
    }

    pub(super) fn apply_profile(&self, profile_id: ProfileId) {
        let config = profile(profile_id);
        self.terminal.set_font_scale(config.font_scale);
    }

    pub(super) fn apply_settings(&self, settings: &Settings, profile_id: ProfileId) {
        self.terminal
            .set_font(Some(&terminal::terminal_font_description(settings)));
        self.terminal.set_font_scale(profile(profile_id).font_scale);
        self.terminal.set_scrollback_lines(settings.scrollback_lines as i64);
        self.terminal.set_enable_sixel(settings.image_rendering);
        self.terminal.set_enable_shaping(settings.ligatures);

        let blink = if settings.cursor_blink {
            CursorBlinkMode::On
        } else {
            CursorBlinkMode::Off
        };
        self.terminal.set_cursor_blink_mode(blink);

        let shape = match settings.cursor_style.as_str() {
            "block" => CursorShape::Block,
            "underline" => CursorShape::Underline,
            _ => CursorShape::Ibeam,
        };
        self.terminal.set_cursor_shape(shape);
    }
}

fn wire_terminal_clipboard(terminal: &Terminal) {
    // Rc<Cell<bool>> debounces selection-copy work so dragging a selection does not spam clipboard writes every motion event.
    let selection_copy_pending = Rc::new(Cell::new(false));
    let pending_ref = selection_copy_pending.clone();
    terminal.connect_selection_changed(move |terminal| {
        if pending_ref.replace(true) {
            return;
        }

        let terminal_ref = terminal.clone();
        let pending_ref = pending_ref.clone();
        gtk::glib::idle_add_local_once(move || {
            pending_ref.set(false);
            if !terminal_ref.has_selection() {
                return;
            }

            copy_terminal_selection(&terminal_ref);
        });
    });

    let controller = EventControllerKey::new();
    let terminal_ref = terminal.clone();
    // Terminal clone is required because GTK key controllers outlive setup and must operate on the live VTE widget.
    controller.connect_key_pressed(move |_, key, _, modifiers| {
        if input::handle_terminal_clipboard_shortcuts(&terminal_ref, key, modifiers) {
            return gtk::glib::Propagation::Stop;
        }

        if modifiers == gdk::ModifierType::empty() && key == gdk::Key::Insert {
            terminal_ref.paste_clipboard();
            return gtk::glib::Propagation::Stop;
        }

        gtk::glib::Propagation::Proceed
    });
    terminal.add_controller(controller);
}

fn copy_terminal_selection(terminal: &Terminal) {
    terminal.copy_primary();
    terminal.copy_clipboard_format(Format::Text);
}
