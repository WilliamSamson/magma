use gtk::{
    gdk,
    pango::FontDescription,
};
use vte4::{prelude::*, CursorBlinkMode, CursorShape, Terminal};

use super::profile::{profile, ProfileId};
use super::super::{
    settings::Settings,
    theme,
};

pub(crate) fn build_terminal(profile_id: ProfileId, settings: &Settings) -> Terminal {
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
    terminal.add_css_class("magma-terminal");
    apply_terminal_settings(&terminal, profile_id, settings);
    terminal
}

pub(crate) fn apply_terminal_settings(terminal: &Terminal, profile_id: ProfileId, settings: &Settings) {
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
    terminal.set_scrollback_lines(settings.scrollback_lines as i64);
    terminal.set_enable_sixel(settings.image_rendering);
    terminal.set_enable_shaping(settings.ligatures);
    apply_terminal_theme(terminal, settings);
}

fn rgba(red: f32, green: f32, blue: f32) -> gdk::RGBA {
    gdk::RGBA::new(red, green, blue, 1.0)
}

fn apply_terminal_theme(terminal: &Terminal, settings: &Settings) {
    let terminal_palette = theme::terminal_palette(settings.theme_mode);
    let palette = terminal_palette.ansi.map(color_rgba);
    let palette_refs = palette.iter().collect::<Vec<_>>();
    terminal.set_colors(
        Some(&color_rgba(terminal_palette.foreground)),
        Some(&color_rgba(terminal_palette.background)),
        &palette_refs,
    );
}

fn color_rgba(color: u32) -> gdk::RGBA {
    let red = ((color >> 16) & 0xFF) as f32 / 255.0;
    let green = ((color >> 8) & 0xFF) as f32 / 255.0;
    let blue = (color & 0xFF) as f32 / 255.0;
    rgba(red, green, blue)
}

pub(crate) fn terminal_font_description(settings: &Settings) -> FontDescription {
    FontDescription::from_string(&format!(
        "{} {}",
        settings.font_family, settings.font_size
    ))
}

pub(crate) fn scaled_spacing(base: i32, settings: &Settings) -> i32 {
    let scale = settings.app_font_size as f32 / 11.0;
    (base as f32 * scale).round() as i32
}
