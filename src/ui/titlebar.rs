use crate::renderer::canvas::Canvas;
use crate::ui::theme;

/// Height of the custom title bar in pixels.
pub(crate) const HEIGHT: u32 = 40;

const BUTTON_W: u32 = 34;
const BUTTON_H: u32 = 24;
const BUTTON_TOP: i32 = 8;
const BUTTON_RIGHT_MARGIN: i32 = 12;
const ICON_SIZE: u32 = 16;
const BRAND_LEFT: i32 = 16;
const BRAND_TOP: i32 = 8;
const BRAND_HEIGHT: u32 = 24;
const TITLE_FONT_SIZE: u32 = 14;

// The lifetime ties the title bar view to renderer-owned title/icon buffers without copying them.
pub(crate) struct TitleBarBrand<'a> {
    pub(crate) title: &'a str,
    pub(crate) icon_rgba: &'a [u8],
    pub(crate) icon_width: u32,
    pub(crate) icon_height: u32,
}

/// What the user clicked on in the title bar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum TitleBarAction {
    Close,
    Maximize,
    Minimize,
    Drag,
    None,
}

/// Draw the custom title bar onto the pixel canvas.
pub(crate) fn draw(
    canvas: &mut Canvas<'_>,
    width: u32,
    brand: &TitleBarBrand<'_>,
    hovered_action: Option<TitleBarAction>,
    is_active_window: bool,
) {
    canvas.draw_rect(0, 0, width, HEIGHT, theme::BG_TITLEBAR);
    canvas.draw_rect(0, HEIGHT as i32 - 1, width, 1, theme::BORDER_STRONG);

    let [close_btn, minimize_btn, maximize_btn] = button_layout(width);
    let brand_width = brand_width(canvas, brand.title, width);
    canvas.draw_rounded_rect(
        BRAND_LEFT,
        BRAND_TOP,
        brand_width,
        BRAND_HEIGHT,
        8,
        theme::BORDER,
    );
    canvas.draw_rounded_rect(
        BRAND_LEFT + 1,
        BRAND_TOP + 1,
        brand_width.saturating_sub(2),
        BRAND_HEIGHT.saturating_sub(2),
        7,
        theme::SURFACE_BASE,
    );
    canvas.draw_rect(BRAND_LEFT + 35, BRAND_TOP + 4, 1, BRAND_HEIGHT.saturating_sub(8), theme::BORDER_STRONG);

    let active_color = if is_active_window { theme::TEXT_PRIMARY } else { theme::TEXT_SECONDARY };
    let dim_color = if is_active_window { theme::TEXT_SECONDARY } else { theme::TEXT_DIM };

    draw_control_button(canvas, minimize_btn, hovered_action, ControlGlyph::Minimize, active_color, dim_color);
    draw_control_button(canvas, maximize_btn, hovered_action, ControlGlyph::Maximize, active_color, dim_color);
    draw_control_button(canvas, close_btn, hovered_action, ControlGlyph::Close, active_color, dim_color);

    draw_icon(
        canvas,
        brand.icon_rgba,
        brand.icon_width,
        brand.icon_height,
        BRAND_LEFT + 10,
    );
    let title_color = if is_active_window { theme::TEXT_PRIMARY } else { theme::TEXT_SECONDARY };
    canvas.draw_text(BRAND_LEFT + 46, 12, brand.title, title_color, TITLE_FONT_SIZE);
}

/// Hit test: given a click at (x, y), return what was clicked.
pub(crate) fn hit_test(x: f64, y: f64, width: u32) -> TitleBarAction {
    let ix = x as i32;
    let iy = y as i32;

    if iy < 0 || iy >= HEIGHT as i32 {
        return TitleBarAction::None;
    }

    for button in button_layout(width) {
        if point_in_button(ix, iy, button) {
            return button.action;
        }
    }

    TitleBarAction::Drag
}

#[derive(Clone, Copy)]
struct ControlButton {
    x: i32,
    y: i32,
    action: TitleBarAction,
}

#[derive(Clone, Copy, PartialEq)]
enum ControlGlyph {
    Close,
    Minimize,
    Maximize,
}

fn draw_icon(
    canvas: &mut Canvas<'_>,
    rgba: &[u8],
    src_w: u32,
    src_h: u32,
    x_offset: i32,
) {
    if rgba.len() < (src_w * src_h * 4) as usize {
        return;
    }
    let dst_w = ICON_SIZE;
    let dst_h = ICON_SIZE;
    let y_offset = (HEIGHT - dst_h) / 2;

    for dy in 0..dst_h {
        for dx in 0..dst_w {
            let sx = (dx * src_w / dst_w) as usize;
            let sy = (dy * src_h / dst_h) as usize;
            let idx = (sy * src_w as usize + sx) * 4;
            let r = rgba[idx] as u32;
            let g = rgba[idx + 1] as u32;
            let b = rgba[idx + 2] as u32;
            let a = rgba[idx + 3];
            if a < 10 {
                continue;
            }
            let color = (r << 16) | (g << 8) | b;
            let px = x_offset + dx as i32;
            let py = y_offset as i32 + dy as i32;
            if a > 240 {
                canvas.set_pixel_pub(px, py, color);
            } else {
                canvas.blend_pixel(px, py, color, a);
            }
        }
    }
}

fn button_layout(width: u32) -> [ControlButton; 3] {
    let y = BUTTON_TOP;
    let close_x = width as i32 - BUTTON_RIGHT_MARGIN - BUTTON_W as i32;
    let maximize_x = close_x - BUTTON_W as i32;
    let minimize_x = maximize_x - BUTTON_W as i32;

    [
        ControlButton {
            x: close_x,
            y,
            action: TitleBarAction::Close,
        },
        ControlButton {
            x: minimize_x,
            y,
            action: TitleBarAction::Minimize,
        },
        ControlButton {
            x: maximize_x,
            y,
            action: TitleBarAction::Maximize,
        },
    ]
}

fn point_in_button(x: i32, y: i32, button: ControlButton) -> bool {
    x >= button.x 
        && y >= button.y
        && x < button.x + BUTTON_W as i32
        && y < button.y + BUTTON_H as i32
}

fn draw_control_button(
    canvas: &mut Canvas<'_>,
    button: ControlButton,
    hovered_action: Option<TitleBarAction>,
    glyph: ControlGlyph,
    active_color: u32,
    dim_color: u32,
) {
    let is_hovered = hovered_action == Some(button.action);
    if is_hovered {
        let plate_color = if glyph == ControlGlyph::Close {
            theme::ACCENT
        } else {
            theme::SURFACE_RAISED
        };
        canvas.draw_rounded_rect(button.x, button.y, BUTTON_W, BUTTON_H, 8, plate_color);
    }

    let icon_color = if is_hovered && glyph == ControlGlyph::Close {
        theme::BG_PRIMARY
    } else if is_hovered {
        active_color
    } else {
        dim_color
    };

    let cx = button.x + BUTTON_W as i32 / 2;
    let cy = button.y + BUTTON_H as i32 / 2;

    match glyph {
        ControlGlyph::Close => {
            canvas.draw_line(cx - 4, cy - 4, cx + 4, cy + 4, icon_color);
            canvas.draw_line(cx - 3, cy - 4, cx + 5, cy + 4, icon_color);
            canvas.draw_line(cx + 4, cy - 4, cx - 4, cy + 4, icon_color);
            canvas.draw_line(cx + 3, cy - 4, cx - 5, cy + 4, icon_color);
        }
        ControlGlyph::Minimize => {
            canvas.draw_rect(cx - 4, cy + 1, 9, 1, icon_color);
        }
        ControlGlyph::Maximize => {
            canvas.draw_rect(cx - 4, cy - 4, 9, 1, icon_color);
            canvas.draw_rect(cx - 4, cy - 4, 1, 9, icon_color);
            canvas.draw_rect(cx + 4, cy - 4, 1, 9, icon_color);
            canvas.draw_rect(cx - 4, cy + 4, 9, 1, icon_color);
        }
    }
}

fn brand_width(canvas: &mut Canvas<'_>, title: &str, window_width: u32) -> u32 {
    let title_width = canvas.text_width(title, TITLE_FONT_SIZE);
    let control_reserve = (BUTTON_W * 3 + BUTTON_RIGHT_MARGIN as u32 + 48) as i32;
    let desired = title_width.saturating_add(72);
    let max_width = (window_width as i32 - control_reserve).max(124) as u32;
    desired.clamp(124, max_width)
}
