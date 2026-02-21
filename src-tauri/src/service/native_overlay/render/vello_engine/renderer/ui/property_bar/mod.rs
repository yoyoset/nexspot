pub mod colors;
pub mod common;
pub mod effects;
pub mod font_size;
pub mod stroke;

use crate::service::native_overlay::state::DrawingTool;
use vello::Scene;

pub fn draw_property_bar(
    scene: &mut Scene,
    rect: &windows::Win32::Foundation::RECT,
    tool: DrawingTool,
    current_color: u32,
    current_font_size: f32,
    current_stroke: f32,
    current_is_filled: bool,
    current_opacity: f32,
    current_glow: f32,
    _current_shadow: bool,
    enable_advanced_effects: bool,
) {
    // 1. Background
    common::draw_background(scene, rect);

    let mut offset_x = rect.left as f64 + 8.0;
    let top = rect.top as f64;
    let bottom = rect.bottom as f64;

    // 2. Font Size (Text only)
    if tool == DrawingTool::Text {
        font_size::draw_font_size_selectors(scene, &mut offset_x, rect, current_font_size);
        common::draw_divider(scene, offset_x - 4.0, top, bottom);
        offset_x += 4.0;
    }

    // 3. Stroke & Fill
    if tool != DrawingTool::Text && tool != DrawingTool::None {
        stroke::draw_stroke_selectors(
            scene,
            &mut offset_x,
            rect,
            tool,
            current_stroke,
            current_is_filled,
        );

        if tool != DrawingTool::Mosaic {
            common::draw_divider(scene, offset_x, top, bottom);
            offset_x += 4.0;
        }
    }

    // 4. Color Palette
    if tool != DrawingTool::Mosaic {
        colors::draw_color_palette(scene, &mut offset_x, rect, current_color);
    }

    // 5. Advanced Effects
    if enable_advanced_effects && tool != DrawingTool::Mosaic {
        common::draw_divider(scene, offset_x, top, bottom);
        offset_x += 4.0;
        effects::draw_advanced_effects(scene, &mut offset_x, rect, current_opacity, current_glow);
    }
}
