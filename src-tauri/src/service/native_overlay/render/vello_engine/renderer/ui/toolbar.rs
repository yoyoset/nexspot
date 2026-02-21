use super::icons::draw_button_icon;
use crate::service::native_overlay::render::toolbar::types::{ButtonState, ToolType};
use crate::service::native_overlay::render::toolbar::Toolbar;
use crate::service::native_overlay::state::{DrawingTool, OverlayState};
use vello::kurbo::{Affine, Rect, RoundedRect, Stroke};
use vello::peniko::{Brush, Color, Fill};
use vello::Scene;

pub fn draw_toolbar_ui(scene: &mut Scene, state: &OverlayState, toolbar: &Toolbar) {
    log::info!(
        "[Vello Debug] Draw Toolbar UI. Visible: {}, Rect: {:?}",
        toolbar.visible,
        toolbar.rect
    );
    if !toolbar.visible {
        return;
    }

    if toolbar.is_loading {
        log::info!("[Vello Debug] Drawing Loading State");
        // Draw background even when loading to cover the GDI "ghost" bar
        let bg_rect = Rect::new(
            toolbar.rect.left as f64,
            toolbar.rect.top as f64,
            toolbar.rect.right as f64,
            toolbar.rect.bottom as f64,
        );
        let bg_brush = Brush::Solid(Color::from_rgba8(20, 20, 20, 255));
        let border_brush = Brush::Solid(Color::from_rgba8(0, 160, 255, 200));
        let border_stroke = Stroke::new(1.5);
        let rounded_bg = RoundedRect::from_rect(bg_rect, 12.0);
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &bg_brush,
            None,
            &rounded_bg,
        );
        scene.stroke(
            &border_stroke,
            Affine::IDENTITY,
            &border_brush,
            None,
            &rounded_bg,
        );
        return;
    }

    // 1. Draw Toolbar Background
    let bg_rect = Rect::new(
        toolbar.rect.left as f64,
        toolbar.rect.top as f64,
        toolbar.rect.right as f64,
        toolbar.rect.bottom as f64,
    );
    let bg_brush = Brush::Solid(Color::from_rgba8(20, 20, 20, 255));
    let border_brush = Brush::Solid(Color::from_rgba8(0, 160, 255, 200));
    let border_stroke = Stroke::new(1.5);

    let rounded_bg = RoundedRect::from_rect(bg_rect, 12.0);
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &bg_brush,
        None,
        &rounded_bg,
    );
    scene.stroke(
        &border_stroke,
        Affine::IDENTITY,
        &border_brush,
        None,
        &rounded_bg,
    );

    // 1.5 Draw Property Bar if visible
    // 1.5 Draw Property Bar if visible
    if toolbar.property_bar_visible {
        if let Some(tool_type) = &toolbar.current_tool {
            let tool = crate::service::native_overlay::render::toolbar::tool_type_to_drawing_tool(
                tool_type,
            );
            super::property_bar::draw_property_bar(
                scene,
                &toolbar.property_bar_rect,
                tool,
                state.current_color,
                state.current_font_size,
                state.current_stroke,
                state.current_is_filled,
                state.current_opacity,
                state.current_glow,
                state.current_shadow,
                state.enable_advanced_effects,
            );
        }
    }

    // 2. Draw Buttons
    for (i, btn) in toolbar.buttons.iter().enumerate() {
        log::info!(
            "[Vello Debug] Drawing Button {}: {:?} at {:?}",
            i,
            btn.tool,
            btn.rect
        );

        let is_active = match (&state.current_tool, &btn.tool) {
            (DrawingTool::Rect, ToolType::Rect) => true,
            (DrawingTool::Ellipse, ToolType::Ellipse) => true,
            (DrawingTool::Arrow, ToolType::Arrow) => true,
            (DrawingTool::Line, ToolType::Line) => true,
            (DrawingTool::Brush, ToolType::Brush) => true,
            (DrawingTool::Mosaic, ToolType::Mosaic) => true,
            (DrawingTool::Text, ToolType::Text) => true,
            (DrawingTool::Number, ToolType::Number) => true,
            _ => false,
        };

        if btn.state != ButtonState::Normal || is_active {
            let btn_color = if btn.state == ButtonState::Pressed {
                Color::from_rgba8(85, 85, 85, 255)
            } else if is_active {
                Color::from_rgba8(68, 68, 68, 255)
            } else {
                Color::from_rgba8(58, 58, 58, 255)
            };
            let btn_rect = Rect::new(
                btn.rect.left as f64,
                btn.rect.top as f64,
                btn.rect.right as f64,
                btn.rect.bottom as f64,
            );
            let rounded_btn = RoundedRect::from_rect(btn_rect, 8.0);
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(btn_color),
                None,
                &rounded_btn,
            );
        }

        draw_button_icon(scene, btn, is_active);
    }
}
