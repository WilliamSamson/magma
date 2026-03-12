use crate::renderer::{canvas::Canvas, DOCK_BG, DOCK_BORDER};
use crate::ui::theme;

const PILL_HEIGHT: u32 = 44;
const PILL_BOTTOM: i32 = 24;
const PILL_PADDING_X: i32 = 6;
const PILL_PADDING_Y: i32 = 6;
const TAB_WIDTH: u32 = 90;
const TAB_HEIGHT: u32 = 32;
const TAB_RADIUS: u32 = 16;
const PILL_RADIUS: u32 = 22;

const SHADOW_OFFSET_Y: i32 = 4;
const SHADOW_COLOR: u32 = 0x00050505;
const INNER_SHELL: u32 = 0x001A1A1A;
const TOP_EDGE: u32 = 0x002E2E2E;

pub struct Dock {
    pub options: [DockOption; 2],
    pub active_index: usize,
    pub hovered_index: Option<usize>,
    pub float_offset: f32,
    pub float_time: f32,
}

pub struct DockOption {
    pub label: &'static str,
    pub bounds: Rect,
}

#[derive(Clone, Copy)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl Dock {
    pub fn new() -> Self {
        Self {
            options: [
                DockOption {
                    label: "Terminal",
                    bounds: Rect { x: 0, y: 0, w: 0, h: 0 },
                },
                DockOption {
                    label: "Log",
                    bounds: Rect { x: 0, y: 0, w: 0, h: 0 },
                },
            ],
            active_index: 0, // Terminal is active by default
            hovered_index: None,
            float_offset: 0.0,
            float_time: 0.0,
        }
    }

    pub fn update_animation(&mut self, delta_time: f32) {
        self.float_time += delta_time;
        // Subtle floating effect
        self.float_offset = (self.float_time * 1.2).sin() * 2.5;
    }

    pub fn update_hover(&mut self, x: i32, y: i32) {
        self.hovered_index = self.options.iter().position(|opt| opt.bounds.contains(x, y));
    }

    pub fn hit_test(&self, x: i32, y: i32) -> Option<usize> {
        self.options.iter().position(|opt| opt.bounds.contains(x, y))
    }

    pub fn set_active(&mut self, index: usize) {
        if index < self.options.len() {
            self.active_index = index;
        }
    }

    pub fn draw(&mut self, canvas: &mut Canvas<'_>, window_width: u32, window_height: u32) {
        let pill_w = (PILL_PADDING_X * 2) as u32 + (self.options.len() as u32 * TAB_WIDTH);
        let pill_x = (window_width as i32 - pill_w as i32) / 2;
        let pill_y = window_height as i32 - PILL_BOTTOM - PILL_HEIGHT as i32 + self.float_offset as i32;

        draw_pill_background(canvas, pill_x, pill_y, pill_w, PILL_HEIGHT);

        for (index, opt) in self.options.iter_mut().enumerate() {
            let tab_x = pill_x + PILL_PADDING_X + (index as u32 * TAB_WIDTH) as i32;
            let tab_y = pill_y + PILL_PADDING_Y;
            opt.bounds = Rect { x: tab_x, y: tab_y, w: TAB_WIDTH, h: TAB_HEIGHT };

            let is_active = index == self.active_index;
            let is_hovered = self.hovered_index == Some(index);

            if is_active {
                // Active Tab Background (Accent color)
                canvas.draw_rounded_rect(tab_x, tab_y, TAB_WIDTH, TAB_HEIGHT, TAB_RADIUS, theme::ACCENT);
                // Inner bevel for active tab
                canvas.draw_rounded_rect(
                    tab_x + 1,
                    tab_y + 1,
                    TAB_WIDTH - 2,
                    TAB_HEIGHT - 2,
                    TAB_RADIUS.saturating_sub(1),
                    theme::ACCENT_MUTED,
                );
            } else if is_hovered {
                // Hovered Inactive Tab Background
                canvas.draw_rounded_rect(tab_x, tab_y, TAB_WIDTH, TAB_HEIGHT, TAB_RADIUS, 0x002A2A2A);
            }

            // Text Rendering
            let text_color = if is_active {
                theme::BG_PRIMARY // Dark text on light accent background
            } else if is_hovered {
                theme::TEXT_PRIMARY // Brighter text on hover
            } else {
                theme::TEXT_SECONDARY // Dim text for inactive
            };

            let text_w = canvas.text_width(opt.label, 13);
            let text_x = tab_x + (TAB_WIDTH as i32 - text_w as i32) / 2;
            let text_y = tab_y + (TAB_HEIGHT as i32 - 13) / 2 - 1; // Center vertically

            canvas.draw_text(text_x, text_y, opt.label, text_color, 13);
            
            // Draw subtle bold effect for active text
            if is_active {
                canvas.draw_text(text_x + 1, text_y, opt.label, text_color, 13);
            }
        }
    }
}

impl Rect {
    fn contains(self, x: i32, y: i32) -> bool {
        x >= self.x && y >= self.y && x < self.x + self.w as i32 && y < self.y + self.h as i32
    }
}

fn draw_pill_background(canvas: &mut Canvas<'_>, x: i32, y: i32, w: u32, h: u32) {
    // Diffused Shadow
    canvas.draw_rounded_rect(x - 2, y + SHADOW_OFFSET_Y, w + 4, h + 4, PILL_RADIUS + 2, 0x00030303);
    canvas.draw_rounded_rect(x, y + SHADOW_OFFSET_Y, w, h, PILL_RADIUS, SHADOW_COLOR);
    
    // Outer Border
    canvas.draw_rounded_rect(x, y, w, h, PILL_RADIUS, DOCK_BORDER);
    
    // Main Background
    canvas.draw_rounded_rect(x + 1, y + 1, w - 2, h - 2, PILL_RADIUS - 1, DOCK_BG);
    canvas.draw_rounded_rect(x + 2, y + 2, w - 4, h - 4, PILL_RADIUS - 2, INNER_SHELL);
    
    // Top highlight/glass edge
    canvas.draw_rect(x + PILL_RADIUS as i32, y + 2, w - (PILL_RADIUS * 2), 1, TOP_EDGE);
}
