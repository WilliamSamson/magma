use gtk::{gdk, pango::FontDescription};
use vte4::{prelude::*, CursorBlinkMode, Terminal};

const SCROLLBACK_LINES: i64 = 20_000;

pub(super) fn build_terminal() -> Terminal {
    let terminal = Terminal::builder()
        .hexpand(true)
        .vexpand(true)
        .can_focus(false)
        .focus_on_click(false)
        .focusable(false)
        .scrollback_lines(SCROLLBACK_LINES as u32)
        .build();
    terminal.add_css_class("obsidian-terminal");
    terminal.set_cursor_blink_mode(CursorBlinkMode::On);
    terminal.set_font(Some(&FontDescription::from_string("DejaVu Sans Mono 10")));
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
