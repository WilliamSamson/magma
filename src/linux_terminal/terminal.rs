use gtk::{
    gdk,
    pango::FontDescription,
};
use vte4::{prelude::*, CursorBlinkMode, CursorShape, Terminal};

use super::{
    profile::{profile, ProfileId},
    settings::Settings,
};

pub(super) fn build_terminal(profile_id: ProfileId, settings: &Settings) -> Terminal {
    let terminal = Terminal::builder()
        .hexpand(true)
        .vexpand(true)
        .can_focus(true)
        .focus_on_click(false)
        .focusable(true)
        .input_enabled(true)
        .scrollback_lines(settings.scrollback_lines)
        .allow_hyperlink(true)
        .enable_shaping(settings.ligatures)
        .enable_sixel(settings.image_rendering)
        .build();
    terminal.add_css_class("obsidian-terminal");

    let blink = if settings.cursor_blink {
        CursorBlinkMode::On
    } else {
        CursorBlinkMode::Off
    };
    terminal.set_cursor_blink_mode(blink);

    let shape = match settings.cursor_style.as_str() {
        "block" => CursorShape::Block,
        "underline" => CursorShape::Underline,
        _ => CursorShape::Ibeam,
    };
    terminal.set_cursor_shape(shape);

    terminal.set_font(Some(&terminal_font_description(settings)));
    terminal.set_font_scale(profile(profile_id).font_scale);

    let palette = [
        rgba(0.00, 0.00, 0.00),
        rgba(1.00, 0.20, 0.20),
        rgba(0.28, 0.66, 0.35),
        rgba(0.96, 0.80, 0.00),
        rgba(0.92, 0.92, 0.92),
        rgba(0.70, 0.70, 0.70),
        rgba(0.70, 0.70, 0.70),
        rgba(0.96, 0.96, 0.96),
    ];
    let palette_refs = palette.iter().collect::<Vec<_>>();
    terminal.set_colors(
        Some(&rgba(0.96, 0.96, 0.96)),
        Some(&rgba(0.00, 0.00, 0.00)),
        &palette_refs,
    );
    terminal
}

fn rgba(red: f32, green: f32, blue: f32) -> gdk::RGBA {
    gdk::RGBA::new(red, green, blue, 1.0)
}

pub(super) fn terminal_font_description(settings: &Settings) -> FontDescription {
    FontDescription::from_string(&format!(
        "{} {}",
        settings.font_family, settings.font_size
    ))
}
