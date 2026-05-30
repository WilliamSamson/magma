pub(crate) mod canvas;
mod dock;
pub(crate) mod font;

use std::{num::NonZeroU32, rc::Rc};

use ratatui::{
    buffer::Buffer,
    style::{Color, Modifier},
};
use softbuffer::{Context, Surface};
use winit::{dpi::PhysicalSize, window::Window};

use self::{canvas::Canvas, font::FontAtlas};
use crate::ui::{
    theme,
    titlebar::{self, TitleBarBrand},
};

const FONT_SIZE: u32 = 14;
const CONTENT_PADDING_X: i32 = 20;
const CONTENT_TOP_GAP: i32 = 18;
const CONTENT_BOTTOM_GAP: i32 = 20;
const CONTENT_RADIUS: u32 = 18;
pub const DOCK_BG: u32 = 0x00222222;
pub const DOCK_BORDER: u32 = 0x00444444;
// JetBrains Mono/Fira Code is not bundled in this repo, so Phase 1 embeds the available local monospace fallback.
const FONT_BYTES: &[u8] = include_bytes!("/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf");

struct SurfaceState {
    _context: Context<Rc<Window>>,
    surface: Surface<Rc<Window>, Rc<Window>>,
}

pub(crate) struct Renderer {
    surface: SurfaceState,
    pixels: Vec<u32>,
    font: FontAtlas,
    size: PhysicalSize<u32>,
    cell_width: u32,
    cell_height: u32,
    icon_rgba: Vec<u8>,
    icon_width: u32,
    icon_height: u32,
    dock: dock::Dock,
}

impl Renderer {
    pub(crate) fn new(
        window: Rc<Window>,
        icon_rgba: Vec<u8>,
        icon_w: u32,
        icon_h: u32,
    ) -> Result<Self, String> {
        let context = Context::new(window.clone()).map_err(|error| error.to_string())?;
        let surface = Surface::new(&context, window.clone()).map_err(|error| error.to_string())?;
        let mut font = FontAtlas::new(FONT_BYTES)?;
        let metrics = font.metrics(FONT_SIZE);
        let size = window.inner_size();
        let mut renderer = Self {
            surface: SurfaceState {
                _context: context,
                surface,
            },
            pixels: Vec::new(),
            font,
            size,
            cell_width: metrics.cell_width,
            cell_height: metrics.cell_height,
            icon_rgba,
            icon_width: icon_w,
            icon_height: icon_h,
            dock: dock::Dock::new(),
        };
        renderer.resize(size.width, size.height)?;
        Ok(renderer)
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) -> Result<(), String> {
        self.size = PhysicalSize::new(width, height);
        self.pixels.resize(
            (width as usize).saturating_mul(height as usize),
            theme::bg_primary(),
        );
        let surface_width = NonZeroU32::new(width.max(1)).unwrap_or(NonZeroU32::MIN);
        let surface_height = NonZeroU32::new(height.max(1)).unwrap_or(NonZeroU32::MIN);
        self.surface
            .surface
            .resize(surface_width, surface_height)
            .map_err(|error| error.to_string())
    }

    /// Returns the grid size available for the ratatui buffer
    /// (subtracts the title bar height from the vertical space).
    pub(crate) fn grid_size(&self) -> (u16, u16) {
        let horizontal_padding = (CONTENT_PADDING_X.max(0) as u32).saturating_mul(2);
        let columns =
            ((self.size.width.saturating_sub(horizontal_padding)) / self.cell_width.max(1)).max(1);
        let usable_height = self
            .size
            .height
            .saturating_sub(titlebar::HEIGHT + CONTENT_TOP_GAP as u32 + CONTENT_BOTTOM_GAP as u32);
        let rows = (usable_height / self.cell_height.max(1)).max(1);
        (
            columns.min(u16::MAX as u32) as u16,
            rows.min(u16::MAX as u32) as u16,
        )
    }

    pub(crate) fn render(
        &mut self,
        buffer: &Buffer,
        show_dock: bool,
        hovered_action: Option<titlebar::TitleBarAction>,
        is_active_window: bool,
    ) -> Result<(), String> {
        if self.size.width == 0 || self.size.height == 0 {
            return Ok(());
        }

        // Fill the entire window with the primary background.
        self.pixels.fill(theme::bg_primary());

        let mut canvas = Canvas::new(
            &mut self.pixels,
            self.size.width as usize,
            self.size.height as usize,
            &mut self.font,
        );
        draw_window_edge(&mut canvas, self.size);

        // Draw the custom title bar at the top.
        let brand = TitleBarBrand {
            title: "Magma",
            icon_rgba: &self.icon_rgba,
            icon_width: self.icon_width,
            icon_height: self.icon_height,
        };
        titlebar::draw(
            &mut canvas,
            self.size.width,
            &brand,
            hovered_action,
            is_active_window,
        );

        let content_frame = content_frame(self.size);
        draw_workspace_shell(&mut canvas, content_frame, show_dock);

        // Draw the ratatui buffer below the title bar.
        draw_terminal_buffer(
            buffer,
            &mut canvas,
            self.cell_width as i32,
            self.cell_height as i32,
            content_frame.x,
            content_frame.y,
        );
        if show_dock {
            self.dock.update_animation(0.016);
            self.dock
                .draw(&mut canvas, self.size.width, self.size.height);
        }

        let mut surface_buffer = self
            .surface
            .surface
            .buffer_mut()
            .map_err(|error| error.to_string())?;
        surface_buffer.copy_from_slice(&self.pixels);
        surface_buffer.present().map_err(|error| error.to_string())
    }

    pub(crate) fn update_dock_hover(&mut self, x: i32, y: i32) {
        self.dock.update_hover(x, y);
    }

    pub(crate) fn dock_hit_test(&self, x: i32, y: i32) -> Option<usize> {
        self.dock.hit_test(x, y)
    }

    pub(crate) fn set_active_dock_item(&mut self, index: usize) {
        self.dock.set_active(index);
    }
}

fn draw_terminal_buffer(
    buffer: &Buffer,
    canvas: &mut Canvas<'_>,
    cell_width: i32,
    cell_height: i32,
    x_offset: i32,
    y_offset: i32,
) {
    for row in 0..buffer.area.height {
        for column in 0..buffer.area.width {
            let Some(cell) = buffer.cell((column, row)) else {
                continue;
            };
            let x = column as i32 * cell_width + x_offset;
            let y = row as i32 * cell_height + y_offset;
            let background = map_background_color(cell.bg);
            if background != theme::bg_primary() {
                canvas.draw_rect(x, y, cell_width as u32, cell_height as u32, background);
            }

            let symbol = cell.symbol();
            if symbol.trim().is_empty() {
                continue;
            }

            let foreground = map_foreground_color(cell.fg);
            if draw_border_symbol(canvas, symbol, x, y, cell_width, cell_height, foreground) {
                continue;
            }

            let font_size = FONT_SIZE;
            let text_x = x + 1;
            let text_y = y;
            canvas.draw_text(text_x, text_y, symbol, foreground, font_size);
            if cell.modifier.contains(Modifier::BOLD) {
                canvas.draw_text(text_x + 1, text_y, symbol, foreground, font_size);
            }
        }
    }
}

fn content_frame(size: PhysicalSize<u32>) -> ContentFrame {
    let x = CONTENT_PADDING_X;
    let y = titlebar::HEIGHT as i32 + CONTENT_TOP_GAP;
    let width = size
        .width
        .saturating_sub((CONTENT_PADDING_X.max(0) as u32).saturating_mul(2));
    let height = size
        .height
        .saturating_sub(titlebar::HEIGHT + CONTENT_TOP_GAP as u32 + CONTENT_BOTTOM_GAP as u32);

    ContentFrame {
        x,
        y,
        width,
        height,
    }
}

#[derive(Clone, Copy)]
struct ContentFrame {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

fn draw_workspace_shell(canvas: &mut Canvas<'_>, frame: ContentFrame, show_dock: bool) {
    if frame.width == 0 || frame.height == 0 {
        return;
    }

    canvas.draw_rounded_rect(
        frame.x,
        frame.y,
        frame.width,
        frame.height,
        CONTENT_RADIUS,
        theme::border_strong(),
    );
    canvas.draw_rounded_rect(
        frame.x + 1,
        frame.y + 1,
        frame.width.saturating_sub(2),
        frame.height.saturating_sub(2),
        CONTENT_RADIUS.saturating_sub(1),
        theme::surface_elevated(),
    );
    canvas.draw_rounded_rect(
        frame.x + 3,
        frame.y + 3,
        frame.width.saturating_sub(6),
        frame.height.saturating_sub(6),
        CONTENT_RADIUS.saturating_sub(3),
        theme::surface_base(),
    );
    canvas.draw_rounded_rect(
        frame.x + 8,
        frame.y + 8,
        frame.width.saturating_sub(16),
        frame.height.saturating_sub(16),
        CONTENT_RADIUS.saturating_sub(7),
        theme::bg_secondary(),
    );
    canvas.draw_rect(
        frame.x + 24,
        frame.y + 14,
        frame.width.saturating_sub(48),
        1,
        theme::border(),
    );

    if show_dock && frame.height > 110 {
        let dock_clear_height = 88;
        let strip_y = frame.y + frame.height as i32 - dock_clear_height as i32 - 8;
        canvas.draw_rect(
            frame.x + 10,
            strip_y,
            frame.width.saturating_sub(20),
            dock_clear_height,
            theme::surface_base(),
        );
        canvas.draw_rect(
            frame.x + 18,
            strip_y + 8,
            frame.width.saturating_sub(36),
            1,
            theme::border_strong(),
        );
    }
}

fn draw_window_edge(canvas: &mut Canvas<'_>, size: PhysicalSize<u32>) {
    if size.width == 0 || size.height == 0 {
        return;
    }

    canvas.draw_rect(0, 0, size.width, 1, theme::window_edge());
    canvas.draw_rect(
        0,
        size.height as i32 - 1,
        size.width,
        1,
        theme::window_edge(),
    );
    canvas.draw_rect(0, 0, 1, size.height, theme::window_edge());
    canvas.draw_rect(
        size.width as i32 - 1,
        0,
        1,
        size.height,
        theme::window_edge(),
    );
}

fn draw_border_symbol(
    canvas: &mut Canvas<'_>,
    symbol: &str,
    x: i32,
    y: i32,
    cell_width: i32,
    cell_height: i32,
    color: u32,
) -> bool {
    let left = x;
    let right = x + cell_width - 1;
    let top = y;
    let bottom = y + cell_height - 1;
    let mid_x = x + cell_width / 2;
    let mid_y = y + cell_height / 2;

    match symbol {
        "─" => canvas.draw_line(left, mid_y, right, mid_y, color),
        "│" => canvas.draw_line(mid_x, top, mid_x, bottom, color),
        "┌" => {
            canvas.draw_line(mid_x, mid_y, right, mid_y, color);
            canvas.draw_line(mid_x, mid_y, mid_x, bottom, color);
        }
        "┐" => {
            canvas.draw_line(left, mid_y, mid_x, mid_y, color);
            canvas.draw_line(mid_x, mid_y, mid_x, bottom, color);
        }
        "└" => {
            canvas.draw_line(mid_x, top, mid_x, mid_y, color);
            canvas.draw_line(mid_x, mid_y, right, mid_y, color);
        }
        "┘" => {
            canvas.draw_line(left, mid_y, mid_x, mid_y, color);
            canvas.draw_line(mid_x, top, mid_x, mid_y, color);
        }
        "├" => {
            canvas.draw_line(mid_x, top, mid_x, bottom, color);
            canvas.draw_line(mid_x, mid_y, right, mid_y, color);
        }
        "┤" => {
            canvas.draw_line(mid_x, top, mid_x, bottom, color);
            canvas.draw_line(left, mid_y, mid_x, mid_y, color);
        }
        "┬" => {
            canvas.draw_line(left, mid_y, right, mid_y, color);
            canvas.draw_line(mid_x, mid_y, mid_x, bottom, color);
        }
        "┴" => {
            canvas.draw_line(left, mid_y, right, mid_y, color);
            canvas.draw_line(mid_x, top, mid_x, mid_y, color);
        }
        "┼" => {
            canvas.draw_line(left, mid_y, right, mid_y, color);
            canvas.draw_line(mid_x, top, mid_x, bottom, color);
        }
        _ => return false,
    }
    true
}

fn map_foreground_color(color: Color) -> u32 {
    match color {
        Color::Reset => theme::text_primary(),
        other => map_palette_color(other),
    }
}

fn map_background_color(color: Color) -> u32 {
    match color {
        Color::Reset => theme::bg_primary(),
        other => map_palette_color(other),
    }
}

fn map_palette_color(color: Color) -> u32 {
    match color {
        Color::Reset | Color::Black => theme::term_black(),
        Color::Red => theme::term_red(),
        Color::LightRed => theme::term_red(),
        Color::Green => theme::term_green(),
        Color::LightGreen => theme::term_green(),
        Color::Yellow => theme::term_yellow(),
        Color::LightYellow => theme::term_yellow(),
        Color::Blue => theme::term_blue(),
        Color::LightBlue => theme::term_blue(),
        Color::Magenta => theme::term_magenta(),
        Color::LightMagenta => theme::term_magenta(),
        Color::Cyan => theme::term_cyan(),
        Color::LightCyan => theme::term_cyan(),
        Color::White => theme::term_white(),
        Color::Gray => theme::term_gray(),
        Color::DarkGray => theme::term_dark_gray(),
        Color::Rgb(red, green, blue) => ((red as u32) << 16) | ((green as u32) << 8) | blue as u32,
        Color::Indexed(value) => indexed_color(value),
    }
}

fn indexed_color(value: u8) -> u32 {
    match value {
        0 => theme::term_black(),
        1 => theme::term_red(),
        2 => theme::term_green(),
        3 => theme::term_yellow(),
        4 => theme::term_blue(),
        5 => theme::term_magenta(),
        6 => theme::term_cyan(),
        7 => theme::term_white(),
        8 => theme::term_dark_gray(),
        9 => theme::term_red(),
        10 => theme::term_green(),
        11 => theme::term_yellow(),
        12 => theme::term_blue(),
        13 => theme::term_magenta(),
        14 => theme::term_cyan(),
        15 => theme::term_white(),
        // 256-color: 16-231 = 6x6x6 color cube, 232-255 = grayscale.
        16..=231 => {
            let idx = value - 16;
            let r = (idx / 36) % 6;
            let g = (idx / 6) % 6;
            let b = idx % 6;
            let to_8bit = |v: u8| if v == 0 { 0u32 } else { 55 + 40 * v as u32 };
            (to_8bit(r) << 16) | (to_8bit(g) << 8) | to_8bit(b)
        }
        232..=255 => {
            let gray = 8 + 10 * (value - 232) as u32;
            (gray << 16) | (gray << 8) | gray
        }
    }
}
