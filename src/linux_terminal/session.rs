use gtk::{
    gio, pango::FontDescription, prelude::*, Box as GtkBox, EventControllerFocus, Orientation,
};
use vte4::{prelude::*, CursorBlinkMode, CursorShape, Terminal};

use super::{
    input,
    profile::{profile, ProfileId},
    settings::Settings,
    shell,
    terminal,
};

pub(super) struct SessionView {
    root: GtkBox,
    terminal: Terminal,
}

impl SessionView {
    pub(super) fn new(profile_id: ProfileId, cwd: Option<&str>, settings: &Settings) -> Self {
        let root = GtkBox::new(Orientation::Vertical, 8);
        root.set_hexpand(true);
        root.set_vexpand(true);

        let terminal = terminal::build_terminal(profile_id, settings);
        let runtime = shell::spawn_shell(&terminal, cwd, &settings.shell);

        root.append(&terminal);
        let _ = input::append_input_row(&root, &terminal, runtime.status_path());

        Self {
            root,
            terminal,
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

    pub(super) fn apply_profile(&self, profile_id: ProfileId) {
        let config = profile(profile_id);
        self.terminal.set_font_scale(config.font_scale);
    }

    pub(super) fn apply_settings(&self, settings: &Settings, profile_id: ProfileId) {
        let font_desc = format!("{} {}", settings.font_family, settings.font_size);
        self.terminal.set_font(Some(&FontDescription::from_string(&font_desc)));
        self.terminal.set_font_scale(profile(profile_id).font_scale);
        self.terminal.set_scrollback_lines(settings.scrollback_lines as i64);

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
