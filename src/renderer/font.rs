use std::collections::HashMap;

use fontdue::{Font, FontSettings, Metrics};

pub(crate) struct FontMetrics {
    pub(crate) cell_width: u32,
    pub(crate) cell_height: u32,
    pub(crate) baseline: i32,
}

pub(crate) struct GlyphBitmap {
    pub(crate) metrics: Metrics,
    pub(crate) bitmap: Vec<u8>,
}

pub(crate) struct FontAtlas {
    font: Font,
    glyphs: HashMap<(char, u32), GlyphBitmap>,
}

impl FontAtlas {
    pub(crate) fn new(bytes: &[u8]) -> Result<Self, String> {
        let font = Font::from_bytes(bytes, FontSettings::default()).map_err(|error| error.to_string())?;
        Ok(Self {
            font,
            glyphs: HashMap::new(),
        })
    }

    pub(crate) fn glyph(&mut self, character: char, font_size: u32) -> Option<&GlyphBitmap> {
        let key = (character, font_size);
        if !self.glyphs.contains_key(&key) {
            let (metrics, bitmap) = self.font.rasterize(character, font_size as f32);
            self.glyphs.insert(key, GlyphBitmap { metrics, bitmap });
        }
        self.glyphs.get(&key)
    }

    pub(crate) fn metrics(&mut self, font_size: u32) -> FontMetrics {
        let line_metrics = self.font.horizontal_line_metrics(font_size as f32);
        let advance_width = self
            .glyph('W', font_size)
            .map(|glyph| glyph.metrics.advance_width.ceil() as u32)
            .unwrap_or_else(|| (font_size / 2).max(1));

        FontMetrics {
            cell_width: advance_width + 2,
            cell_height: line_metrics
                .map(|metrics| metrics.new_line_size.ceil() as u32)
                .unwrap_or(font_size + 4)
                .max(1),
            baseline: line_metrics
                .map(|metrics| metrics.ascent.ceil() as i32)
                .unwrap_or(font_size as i32),
        }
    }
}
