pub mod colors;
pub mod common;
pub mod constants;
pub mod effects;
pub mod font_size;
pub mod stroke;

use crate::service::native_overlay::state::DrawingTool;
use vello::Scene;
use constants::*;

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
    _current_style: crate::service::config::types::AestheticStyle,
    enable_advanced_effects: bool,
) {
    // 1. Create a sub-scene for the content to measure its width
    let mut content_scene = Scene::new();
    
    let mut offset_x = rect.left as f64 + PB_HPADDING;

    // 2. Group 1: Base Tool Params (Font Size or Stroke)
    let group1_start = offset_x;
    if tool == DrawingTool::Text {
        font_size::draw_font_size_selectors(&mut content_scene, &mut offset_x, rect, current_font_size);
    } else if tool != DrawingTool::None {
        stroke::draw_stroke_selectors(
            &mut content_scene,
            &mut offset_x,
            rect,
            tool,
            current_stroke,
            current_is_filled,
        );
    }
    
    // Draw Background for Group 1 (on content_scene)
    if offset_x > group1_start {
        draw_group_background(&mut content_scene, group1_start - 1.0, offset_x + 1.0, rect);
    }

    // 3. Group 2: Color Palette
    if tool != DrawingTool::Mosaic {
        let group2_start = if offset_x > group1_start + 1.0 { offset_x + GROUP_GAP } else { offset_x };
        offset_x = group2_start;
        colors::draw_color_palette(&mut content_scene, &mut offset_x, rect, current_color);
        draw_group_background(&mut content_scene, group2_start - 1.0, offset_x + 1.0, rect);
    }

    // 4. Group 3: Advanced Effects
    if enable_advanced_effects && tool != DrawingTool::Mosaic {
        let group3_start = offset_x + GROUP_GAP;
        offset_x = group3_start;
        effects::draw_advanced_effects(
            &mut content_scene,
            &mut offset_x,
            rect,
            current_opacity,
            current_glow,
        );
        draw_group_background(&mut content_scene, group3_start - 1.0, offset_x + 1.0, rect);
    }

    // 5. Draw Main Container Background (Dynamically sized)
    let final_width = (offset_x - rect.left as f64 + PB_HPADDING).max(40.0);
    let final_rect = windows::Win32::Foundation::RECT {
        left: rect.left,
        top: rect.top,
        right: rect.left + final_width.ceil() as i32,
        bottom: rect.bottom,
    };
    common::draw_background(scene, &final_rect);

    // 6. Append Content Scene
    scene.append(&content_scene, None);
}

fn draw_group_background(scene: &mut Scene, x0: f64, x1: f64, rect: &windows::Win32::Foundation::RECT) {
    use vello::kurbo::{Affine, Rect, RoundedRect};
    use vello::peniko::Color;

    let group_rect = Rect::new(
        x0,
        rect.top as f64 + 4.0,
        x1,
        rect.bottom as f64 - 4.0,
    );
    let rounded = RoundedRect::from_rect(group_rect, 1.0);
    
    scene.stroke(
        &vello::kurbo::Stroke::new(1.0),
        Affine::IDENTITY,
        Color::from_rgba8(39, 39, 42, 100), // Zinc-800 subtle
        None,
        &rounded,
    );
}
