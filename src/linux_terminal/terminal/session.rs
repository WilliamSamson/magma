use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use gtk::{
    gdk, gio, prelude::*, Align, Box as GtkBox, EventControllerFocus, EventControllerKey, Label, Orientation,
};
use vte4::{prelude::*, Format, Terminal};

use super::{
    profile::{profile, ProfileId},
    scaled_spacing,
    shell,
    widget as terminal,
};
use super::super::{
    input,
    persist::SessionSnapshot,
    settings::Settings,
};

pub(crate) struct SessionView {
    root: GtkBox,
    terminal: Terminal,
    snapshot: SessionSnapshot,
}

impl SessionView {
    pub(crate) fn new(
        profile_id: ProfileId,
        snapshot: &SessionSnapshot,
        settings: Rc<RefCell<Settings>>, // settings Rc wrapper is cloned to share settings with SessionView.
    ) -> Self {
        let settings_ref = settings.borrow();
        let root = GtkBox::new(Orientation::Vertical, 0);
        root.add_css_class("magma-terminal-container");
        root.set_hexpand(true);
        root.set_vexpand(true);

        // Snapshot is cloned because SessionView needs to own its specific session state.
        let snapshot = snapshot.clone().normalized();
        let terminal = terminal::build_terminal(profile_id, &settings_ref);
        let _runtime = shell::spawn_shell(&terminal, &snapshot, &settings_ref.shell);
        drop(settings_ref);

        // Create a compact path breadcrumb bar at the top of the terminal layout
        let path_bar = GtkBox::new(Orientation::Horizontal, 4);
        path_bar.add_css_class("magma-terminal-path-bar");

        let path_icon = gtk::Image::from_icon_name("folder-symbolic");
        path_icon.add_css_class("magma-terminal-path-icon");
        path_bar.append(&path_icon);

        let path_label = Label::new(Some(&format_path_display(&terminal)));
        path_label.add_css_class("magma-terminal-path-label");
        path_label.set_halign(Align::Start);
        path_bar.append(&path_label);

        let terminal_for_update = terminal.clone(); // terminal clone is needed inside URI update callback to fetch path.
        let path_label_ref = path_label.clone(); // path_label clone is needed to modify label text inside URI update callback.
        terminal.connect_current_directory_uri_changed(move |_| {
            path_label_ref.set_text(&format_path_display(&terminal_for_update));
        });

        root.append(&path_bar);
        root.append(&terminal);
        wire_terminal_clipboard(&terminal);

        let terminal_ref = terminal.clone(); // terminal_ref clone is needed to grab focus on the VTE terminal widget on idle initialization.
        gtk::glib::idle_add_local_once(move || {
            let _ = terminal_ref.grab_focus();
        });

        Self {
            root,
            terminal,
            snapshot,
        }
    }

    pub(crate) fn root(&self) -> &GtkBox {
        &self.root
    }

    pub(crate) fn terminal(&self) -> &Terminal {
        &self.terminal
    }

    pub(crate) fn focus_terminal(&self) {
        self.terminal.grab_focus();
    }

    pub(crate) fn connect_focus_enter(&self, on_focus: impl Fn() + 'static) {
        let controller = EventControllerFocus::new();
        controller.connect_enter(move |_| on_focus());
        self.terminal.add_controller(controller);
    }

    pub(crate) fn current_cwd(&self) -> Option<String> {
        self.terminal
            .current_directory_uri()
            .as_deref()
            .and_then(|uri| gio::File::for_uri(uri).path())
            .map(|path| path.display().to_string())
    }

    pub(crate) fn to_snapshot(&self) -> SessionSnapshot {
        let mut snapshot = self.snapshot.clone();
        snapshot.cwd = self.current_cwd().or_else(|| snapshot.cwd.clone());
        snapshot
    }

    pub(crate) fn apply_profile(&self, profile_id: ProfileId) {
        let config = profile(profile_id);
        self.terminal.set_font_scale(config.font_scale);
    }

    pub(crate) fn apply_settings(&self, settings: &Settings, profile_id: ProfileId) {
        terminal::apply_terminal_settings(&self.terminal, profile_id, settings);
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

fn format_path_display(terminal: &Terminal) -> String {
    let uri = terminal.current_directory_uri();
    let path_str = uri.as_deref()
        .and_then(|u| gio::File::for_uri(u).path())
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| {
            std::env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "/".to_string())
        });

    let home = std::env::var("HOME").unwrap_or_default();
    if !home.is_empty() && path_str.starts_with(&home) {
        format!("~{}", &path_str[home.len()..])
    } else {
        path_str
    }
}
