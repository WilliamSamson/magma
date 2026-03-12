use crate::renderer::font::FontAtlas;

pub(crate) struct Canvas<'a> {
    pixels: &'a mut [u32],
    width: usize,
    height: usize,
    font: &'a mut FontAtlas,
}

impl<'a> Canvas<'a> {
    pub(crate) fn new(
        pixels: &'a mut [u32],
        width: usize,
        height: usize,
        font: &'a mut FontAtlas,
    ) -> Self {
        Self {
            pixels,
            width,
            height,
            font,
        }
    }

    pub(crate) fn draw_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: u32) {
        let x0 = x.max(0) as usize;
        let y0 = y.max(0) as usize;
        let x1 = (x + w as i32).min(self.width as i32).max(0) as usize;
        let y1 = (y + h as i32).min(self.height as i32).max(0) as usize;

        for py in y0..y1 {
            let row = py * self.width;
            for px in x0..x1 {
                self.pixels[row + px] = color;
            }
        }
    }

    pub(crate) fn draw_rounded_rect(
        &mut self,
        x: i32,
        y: i32,
        w: u32,
        h: u32,
        radius: u32,
        color: u32,
    ) {
        let radius = radius.min(w / 2).min(h / 2) as i32;
        if radius <= 0 {
            self.draw_rect(x, y, w, h, color);
            return;
        }

        self.draw_rect(x + radius, y, w.saturating_sub((radius as u32) * 2), h, color);
        self.draw_rect(x, y + radius, radius as u32, h.saturating_sub((radius as u32) * 2), color);
        self.draw_rect(
            x + w as i32 - radius,
            y + radius,
            radius as u32,
            h.saturating_sub((radius as u32) * 2),
            color,
        );

        let r2 = radius * radius;
        for dy in 0..radius {
            for dx in 0..radius {
                let ox = radius - dx - 1;
                let oy = radius - dy - 1;
                if ox * ox + oy * oy > r2 {
                    continue;
                }
                self.set_pixel(x + dx, y + dy, color);
                self.set_pixel(x + w as i32 - radius + dx, y + dy, color);
                self.set_pixel(x + dx, y + h as i32 - radius + dy, color);
                self.set_pixel(x + w as i32 - radius + dx, y + h as i32 - radius + dy, color);
            }
        }
    }

    pub(crate) fn draw_text(&mut self, x: i32, y: i32, text: &str, color: u32, font_size: u32) {
        let Canvas {
            pixels,
            width,
            height,
            font,
        } = self;
        let metrics = font.metrics(font_size);
        let mut cursor_x = x;
        let baseline = y + metrics.baseline;

        for character in text.chars() {
            if character == ' ' {
                cursor_x += metrics.cell_width as i32;
                continue;
            }

            let Some(glyph) = font.glyph(character, font_size) else {
                cursor_x += metrics.cell_width as i32;
                continue;
            };
            let draw_x = cursor_x + glyph.metrics.xmin;
            let draw_y = baseline - glyph.metrics.height as i32 - glyph.metrics.ymin;
            draw_bitmap(
                pixels,
                *width,
                *height,
                Bitmap {
                    x: draw_x,
                    y: draw_y,
                    width: glyph.metrics.width,
                    height: glyph.metrics.height,
                    bitmap: &glyph.bitmap,
                    color,
                },
            );
            cursor_x += glyph.metrics.advance_width.ceil() as i32;
        }
    }

    pub(crate) fn text_width(&mut self, text: &str, font_size: u32) -> u32 {
        let metrics = self.font.metrics(font_size);
        text.chars().count() as u32 * metrics.cell_width
    }

    pub(crate) fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: u32) {
        let mut x = x1;
        let mut y = y1;
        let dx = (x2 - x1).abs();
        let dy = -(y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut error = dx + dy;

        loop {
            self.set_pixel(x, y, color);
            if x == x2 && y == y2 {
                break;
            }

            let twice_error = error * 2;
            if twice_error >= dy {
                error += dy;
                x += sx;
            }
            if twice_error <= dx {
                error += dx;
                y += sy;
            }
        }
    }

    /// Set a pixel (public for title bar icon rendering).
    pub(crate) fn set_pixel_pub(&mut self, x: i32, y: i32, color: u32) {
        self.set_pixel(x, y, color);
    }

    /// Blend a pixel with alpha (public for title bar icon rendering).
    pub(crate) fn blend_pixel(&mut self, x: i32, y: i32, color: u32, alpha: u8) {
        if let Some(index) = self.pixel_index(x, y) {
            self.pixels[index] = blend(self.pixels[index], color, alpha);
        }
    }

    fn pixel_index(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 {
            return None;
        }

        let x = x as usize;
        let y = y as usize;
        if x >= self.width || y >= self.height {
            return None;
        }

        Some(y * self.width + x)
    }

    fn set_pixel(&mut self, x: i32, y: i32, color: u32) {
        if let Some(index) = self.pixel_index(x, y) {
            self.pixels[index] = color;
        }
    }
}

fn blend(background: u32, foreground: u32, alpha: u8) -> u32 {
    let alpha = alpha as u32;
    let inverse = 255 - alpha;
    let bg_r = (background >> 16) & 0xFF;
    let bg_g = (background >> 8) & 0xFF;
    let bg_b = background & 0xFF;
    let fg_r = (foreground >> 16) & 0xFF;
    let fg_g = (foreground >> 8) & 0xFF;
    let fg_b = foreground & 0xFF;
    let red = (fg_r * alpha + bg_r * inverse) / 255;
    let green = (fg_g * alpha + bg_g * inverse) / 255;
    let blue = (fg_b * alpha + bg_b * inverse) / 255;

    (red << 16) | (green << 8) | blue
}

struct Bitmap<'a> {
    x: i32,
    y: i32,
    width: usize,
    height: usize,
    bitmap: &'a [u8],
    color: u32,
}

fn draw_bitmap(pixels: &mut [u32], surface_width: usize, surface_height: usize, glyph: Bitmap<'_>) {
    for glyph_y in 0..glyph.height {
        for glyph_x in 0..glyph.width {
            let alpha = glyph.bitmap[glyph_y * glyph.width + glyph_x];
            if alpha == 0 {
                continue;
            }

            let px = glyph.x + glyph_x as i32;
            let py = glyph.y + glyph_y as i32;
            if px < 0 || py < 0 {
                continue;
            }

            let px = px as usize;
            let py = py as usize;
            if px >= surface_width || py >= surface_height {
                continue;
            }

            let index = py * surface_width + px;
            pixels[index] = blend(pixels[index], glyph.color, alpha);
        }
    }
}
