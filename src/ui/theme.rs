use std::sync::RwLock;

// Instead of hardcoded constants, we query the currently active palette
// managed by the settings. This bridges the GTK theme state to the custom
// winit/ratatui renderer.
use crate::linux_terminal::theme::{self as gtk_theme, Palette};

static ACTIVE_PALETTE: RwLock<Option<Palette>> = RwLock::new(None);

pub(crate) fn set_palette(palette: Palette) {
    if let Ok(mut w) = ACTIVE_PALETTE.write() {
        *w = Some(palette);
    }
}

fn get_color(selector: impl FnOnce(&Palette) -> u32) -> u32 {
    if let Ok(r) = ACTIVE_PALETTE.read() {
        if let Some(palette) = &*r {
            return selector(palette);
        }
    }
    // Fallback to dark theme default if not yet initialized
    selector(&gtk_theme::palette(gtk_theme::ThemeMode::Dark))
}

// ── Basic UI Colors ──

pub(crate) fn bg_primary() -> u32 {
    get_color(|p| p.bg_primary)
}
pub(crate) fn bg_secondary() -> u32 {
    get_color(|p| p.surface_base)
} // Mapping desktop's secondary to surface
pub(crate) fn bg_sidebar() -> u32 {
    get_color(|p| p.bg_titlebar)
} // Sidebar uses titlebar color in new standard
pub(crate) fn bg_titlebar() -> u32 {
    get_color(|p| p.bg_titlebar)
}
pub(crate) fn surface_base() -> u32 {
    get_color(|p| p.surface_base)
}
pub(crate) fn surface_raised() -> u32 {
    get_color(|p| p.surface_base)
} // Unified
pub(crate) fn surface_elevated() -> u32 {
    get_color(|p| p.surface_base)
}

pub(crate) fn accent() -> u32 {
    get_color(|p| p.accent)
}
pub(crate) fn accent_muted() -> u32 {
    get_color(|p| p.accent) & 0xFFFFFF | 0x33000000
} // Crude alpha down
pub(crate) fn success() -> u32 {
    0x00388C50
} // Can leave these semantic colors static or get from terminal palette
pub(crate) fn warning() -> u32 {
    0x00C89A1E
}

pub(crate) fn text_primary() -> u32 {
    get_color(|p| p.text_primary)
}
pub(crate) fn text_secondary() -> u32 {
    get_color(|p| p.text_secondary)
}
pub(crate) fn text_dim() -> u32 {
    get_color(|p| p.text_dim)
}
pub(crate) fn window_edge() -> u32 {
    get_color(|p| p.window_edge)
}

pub(crate) fn border() -> u32 {
    get_color(|p| p.border_strong)
} // Unified
pub(crate) fn border_strong() -> u32 {
    get_color(|p| p.border_strong)
}

// ── Terminal Semantics ──

pub(crate) fn term_black() -> u32 {
    bg_primary()
}
pub(crate) fn term_red() -> u32 {
    accent()
}
pub(crate) fn term_green() -> u32 {
    success()
}
pub(crate) fn term_yellow() -> u32 {
    warning()
}
pub(crate) fn term_blue() -> u32 {
    text_primary()
}
pub(crate) fn term_magenta() -> u32 {
    text_secondary()
}
pub(crate) fn term_cyan() -> u32 {
    text_secondary()
}
pub(crate) fn term_white() -> u32 {
    text_primary()
}
pub(crate) fn term_gray() -> u32 {
    text_secondary()
}
pub(crate) fn term_dark_gray() -> u32 {
    text_dim()
}

/// Convert a theme hex color to ratatui `Color::Rgb`.
pub(crate) fn to_ratatui(color: u32) -> ratatui::style::Color {
    let r = ((color >> 16) & 0xFF) as u8;
    let g = ((color >> 8) & 0xFF) as u8;
    let b = (color & 0xFF) as u8;
    ratatui::style::Color::Rgb(r, g, b)
}
