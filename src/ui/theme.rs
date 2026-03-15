// Magma desktop palette.
// Constraint: black, white, red, with restrained green/yellow accents only.

pub(crate) const BG_PRIMARY: u32 = 0x00000000;
pub(crate) const BG_SECONDARY: u32 = 0x00040404;
pub(crate) const BG_SIDEBAR: u32 = 0x00070707;
pub(crate) const BG_TITLEBAR: u32 = 0x00000000;
pub(crate) const SURFACE_BASE: u32 = 0x000B0B0B;
pub(crate) const SURFACE_RAISED: u32 = 0x00111111;
pub(crate) const SURFACE_ELEVATED: u32 = 0x00181818;

pub(crate) const ACCENT: u32 = 0x00FF4D4D;
pub(crate) const ACCENT_MUTED: u32 = 0x007A2A2A;
pub(crate) const SUCCESS: u32 = 0x00388C50;
pub(crate) const WARNING: u32 = 0x00C89A1E;

pub(crate) const TEXT_PRIMARY: u32 = 0x00F3F3EF;
pub(crate) const TEXT_SECONDARY: u32 = 0x00BABAB3;
pub(crate) const TEXT_DIM: u32 = 0x006B6B66;
pub(crate) const WINDOW_EDGE: u32 = 0x00E8E8E8;

pub(crate) const BORDER: u32 = 0x00151515;
pub(crate) const BORDER_STRONG: u32 = 0x00242424;

pub(crate) const TERM_BLACK: u32 = BG_PRIMARY;
pub(crate) const TERM_RED: u32 = ACCENT;
pub(crate) const TERM_GREEN: u32 = SUCCESS;
pub(crate) const TERM_YELLOW: u32 = WARNING;
pub(crate) const TERM_BLUE: u32 = TEXT_PRIMARY;
pub(crate) const TERM_MAGENTA: u32 = TEXT_SECONDARY;
pub(crate) const TERM_CYAN: u32 = TEXT_SECONDARY;
pub(crate) const TERM_WHITE: u32 = TEXT_PRIMARY;
pub(crate) const TERM_GRAY: u32 = TEXT_SECONDARY;
pub(crate) const TERM_DARK_GRAY: u32 = TEXT_DIM;

/// Convert a theme hex color to ratatui `Color::Rgb`.
pub(crate) fn to_ratatui(color: u32) -> ratatui::style::Color {
    let r = ((color >> 16) & 0xFF) as u8;
    let g = ((color >> 8) & 0xFF) as u8;
    let b = (color & 0xFF) as u8;
    ratatui::style::Color::Rgb(r, g, b)
}
