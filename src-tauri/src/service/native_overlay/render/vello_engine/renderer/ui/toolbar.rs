use super::icons::draw_button_icon;
use crate::service::native_overlay::render::toolbar::types::{ButtonState, ToolType};
use crate::service::native_overlay::render::toolbar::Toolbar;
use crate::service::native_overlay::state::OverlayState;
use vello::kurbo::{Affine, Rect, RoundedRect, Stroke};
use vello::peniko::{Brush, Color, Fill};
use vello::Scene;

pub fn draw_toolbar_ui(scene: &mut Scene, state: &OverlayState, toolbar: &Toolbar) {
    if !toolbar.visible {
        return;
    }

    // --- 1. Draw Main Toolbar ---
    draw_bar(
        scene,
        state,
        &toolbar.main_buttons,
        &toolbar.rect,
        toolbar.is_loading,
        toolbar.property_bar_visible,
        &toolbar.property_bar_rect,
        toolbar.current_tool.as_ref(),
    );
}

fn draw_bar(
    scene: &mut Scene,
    state: &OverlayState,
    buttons: &[crate::service::native_overlay::render::toolbar::types::ToolbarButton],
    rect: &windows::Win32::Foundation::RECT,
    is_loading: bool,
    property_bar_visible: bool,
    property_bar_rect: &windows::Win32::Foundation::RECT,
    current_tool_type: Option<&ToolType>,
) {
    if rect.left == rect.right {
        return;
    }

    // Draw Background (Zinc-950)
    let bg_rect = Rect::new(
        rect.left as f64,
        rect.top as f64,
        rect.right as f64,
        rect.bottom as f64,
    );
    let bg_brush = Brush::Solid(Color::from_rgba8(24, 24, 27, 255));
    let border_brush = Brush::Solid(Color::from_rgba8(39, 39, 42, 255));
    let border_stroke = Stroke::new(1.0);
    let rounded_bg = RoundedRect::from_rect(bg_rect, 3.0);

    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &bg_brush,
        None,
        &rounded_bg,
    );
    
    // Top Highlight Line (8% White)
    // Top Highlight Line
    let highlight_brush = Brush::Solid(Color::from_rgba8(255, 255, 255, 20));
    let highlight_rect = Rect::new(
        bg_rect.x0 + 3.0,
        bg_rect.y0 + 1.0,
        bg_rect.x1 - 3.0,
        bg_rect.y0 + 2.0,
    );
    scene.fill(Fill::NonZero, Affine::IDENTITY, &highlight_brush, None, &highlight_rect);

    // Bottom Shadow Line (20% Black)
    // Bottom Shadow Line
    let shadow_brush = Brush::Solid(Color::from_rgba8(0, 0, 0, 50));
    let shadow_rect = Rect::new(
        bg_rect.x0 + 3.0,
        bg_rect.y1 - 2.0,
        bg_rect.x1 - 3.0,
        bg_rect.y1 - 1.0,
    );
    scene.fill(Fill::NonZero, Affine::IDENTITY, &shadow_brush, None, &shadow_rect);

    // Outer Border (Zinc-800)
    scene.stroke(
        &border_stroke,
        Affine::IDENTITY,
        &border_brush,
        None,
        &rounded_bg,
    );

    if is_loading {
        return;
    }

    // Draw Property Bar
    if property_bar_visible {
        if let Some(tool_type) = current_tool_type {
            let tool = crate::service::native_overlay::render::toolbar::tool_type_to_drawing_tool(
                tool_type,
            );
            super::property_bar::draw_property_bar(
                scene,
                property_bar_rect,
                tool,
                state.current_color,
                state.current_font_size,
                state.current_stroke,
                state.current_is_filled,
                state.current_opacity,
                state.current_glow,
                state.current_shadow,
                state.current_style,
                state.enable_advanced_effects,
            );
        }
    }

    // Draw Buttons
    for btn in buttons {
        let is_active = match (current_tool_type, &btn.tool) {
            (Some(ToolType::Rect), ToolType::Rect) => true,
            (Some(ToolType::Ellipse), ToolType::Ellipse) => true,
            (Some(ToolType::Arrow), ToolType::Arrow) => true,
            (Some(ToolType::Line), ToolType::Line) => true,
            (Some(ToolType::Brush), ToolType::Brush) => true,
            (Some(ToolType::Mosaic), ToolType::Mosaic) => true,
            (Some(ToolType::Text), ToolType::Text) => true,
            (Some(ToolType::Number), ToolType::Number) => true,
            _ => false,
        };

        if btn.state != ButtonState::Normal || is_active {
            let btn_color = if btn.state == ButtonState::Pressed || is_active {
                Color::from_rgba8(63, 63, 70, 255) // Zinc-700
            } else {
                Color::from_rgba8(39, 39, 42, 255) // Zinc-800
            };
            let btn_rect = Rect::new(
                btn.rect.left as f64,
                btn.rect.top as f64,
                btn.rect.right as f64,
                btn.rect.bottom as f64,
            );
            let rounded_btn = RoundedRect::from_rect(btn_rect, 2.0);
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(btn_color),
                None,
                &rounded_btn,
            );

            // Active Marker (Blue-500)
            if is_active {
                let marker_rect = Rect::new(
                    btn_rect.x0 + 2.0,
                    btn_rect.y0 + 10.0,
                    btn_rect.x0 + 4.0,
                    btn_rect.y1 - 10.0,
                );
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(Color::from_rgba8(59, 130, 246, 255)),
                    None,
                    &marker_rect,
                );
            }
        }

        draw_button_icon(scene, btn, is_active);
    }
}
