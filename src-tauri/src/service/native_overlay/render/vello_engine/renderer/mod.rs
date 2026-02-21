use crate::service::native_overlay::render::toolbar::Toolbar;
use crate::service::native_overlay::state::{DrawingTool, OverlayState};
use vello::Scene;

pub mod tools;
pub mod ui;
pub mod utils;

pub fn render_state_to_scene(state: &OverlayState, toolbar_obj: &Toolbar, scene: &mut Scene) {
    scene.reset();

    // 0. Apply Global Translation
    // Since our state coordinates (mouse, selection, objects) are in GLOBAL physical space,
    // but the Vello surface is local to the window (which is at state.x, state.y),
    // we must offset everything by -state.x and -state.y.
    let global_transform = vello::kurbo::Affine::translate((-(state.x as f64), -(state.y as f64)));

    // 1. Render Background
    if let Some(bg) = &state.vello.background {
        // The background image is a capture of the specific monitor we are on.
        // Its (0,0) corresponds to the monitor's top-left, which is where our window starts.
        // So we draw it at (0,0) in the surface (which is state.x, state.y in global space).
        scene.draw_image(bg, vello::kurbo::Affine::IDENTITY);
    }

    // Use a bounding box safe from GPU texture limit (16k) but broad enough for multi-monitor setups.
    let clip_rect = vello::kurbo::Rect::new(-8000.0, -8000.0, 8000.0, 8000.0);

    // Wrap everything else in the global transform layer
    scene.push_layer(
        vello::peniko::Fill::NonZero,
        vello::peniko::Mix::Normal,
        1.0,
        global_transform,
        &clip_rect,
    );

    // 2. Render UI and Objects into an Intermediate Scene
    // Vello's `push_layer` does NOT apply the `transform` argument to the internal shapes (it only transforms the clip).
    // To ensure all our absolute coordinates accurately shift over multi-monitor coordinate offsets,
    // we must draw into an independent scene, then apply our global affine when appending it.
    let mut inner_scene = vello::Scene::new();

    if let Some(sel) = state.selection {
        ui::selection::draw_selection_ui(&mut inner_scene, state, &sel);
    }

    // 3. Render Drawing Objects
    for obj in &state.objects {
        let enable_effects = state.enable_advanced_effects;

        // 3a. Glow (Advanced Only)
        if enable_effects && obj.glow > 0.0 {
            let mut glow_obj = obj.clone();
            // Glow intensity: opacity base 0.1 to 0.4 based on glow slider
            // Glow spread: increase stroke width by up to 24px
            let glow_opacity = (obj.glow * 0.4).clamp(0.05, 0.4);
            let glow_spread = obj.glow * 24.0;

            glow_obj.stroke_width += glow_spread;
            // Apply glow opacity by mixing with existing opacity
            let final_glow_opacity = glow_opacity * obj.opacity;

            inner_scene.push_layer(
                vello::peniko::Fill::NonZero,
                vello::peniko::Mix::Normal,
                final_glow_opacity,
                vello::kurbo::Affine::IDENTITY,
                &clip_rect,
            );
            draw_object(&mut inner_scene, &glow_obj);
            inner_scene.pop_layer();
        }

        // 3b. Shadow (Advanced Only)
        if enable_effects && obj.has_shadow {
            let mut shadow_obj = obj.clone();
            shadow_obj.color = 0x40000000; // ~25% Black

            inner_scene.push_layer(
                vello::peniko::Fill::NonZero,
                vello::peniko::Mix::Normal,
                1.0,
                vello::kurbo::Affine::translate((4.0, 4.0)),
                &clip_rect,
            );
            draw_object(&mut inner_scene, &shadow_obj);
            inner_scene.pop_layer();
        }

        // 3c. Opacity
        let has_opacity = enable_effects && obj.opacity < 1.0;
        if has_opacity {
            inner_scene.push_layer(
                vello::peniko::Fill::NonZero,
                vello::peniko::Mix::Normal,
                obj.opacity,
                vello::kurbo::Affine::IDENTITY,
                &clip_rect,
            );
        }

        draw_object(&mut inner_scene, obj);

        if has_opacity {
            inner_scene.pop_layer();
        }
    }

    // 4. Render Current Interaction (Preview)
    if let Some(current) = &state.current_drawing {
        let enable_effects = state.enable_advanced_effects;

        // Preview Glow
        if enable_effects && current.glow > 0.0 {
            let mut glow_obj = current.clone();
            let glow_opacity = (current.glow * 0.4).clamp(0.05, 0.4);
            let glow_spread = current.glow * 24.0;
            glow_obj.stroke_width += glow_spread;
            let final_glow_opacity = glow_opacity * current.opacity;

            // Preview Glow
            inner_scene.push_layer(
                vello::peniko::Fill::NonZero,
                vello::peniko::Mix::Normal,
                final_glow_opacity,
                vello::kurbo::Affine::IDENTITY,
                &clip_rect,
            );
            draw_object(&mut inner_scene, &glow_obj);
            inner_scene.pop_layer();
        }

        // Preview Opacity
        let has_opacity = enable_effects && current.opacity < 1.0;
        if has_opacity {
            inner_scene.push_layer(
                vello::peniko::Fill::NonZero,
                vello::peniko::Mix::Normal,
                current.opacity,
                vello::kurbo::Affine::IDENTITY,
                &clip_rect,
            );
        }

        draw_object(&mut inner_scene, current);

        if has_opacity {
            inner_scene.pop_layer();
        }
    }

    // 5. Render Magnifier
    ui::magnifier::draw_magnifier(&mut inner_scene, state);

    // 6. Render Toolbar
    ui::toolbar::draw_toolbar_ui(&mut inner_scene, state, toolbar_obj);

    // 7. Draw Custom Brush/Mosaic Circle Preview
    draw_tool_preview(&mut inner_scene, state, toolbar_obj);

    // 8. Append the intermediate scene using the true global transform
    scene.append(&inner_scene, Some(global_transform));

    // Pop Global Translation Layer
    scene.pop_layer();
}

fn draw_object(scene: &mut Scene, obj: &crate::service::native_overlay::state::DrawingObject) {
    if obj.points.is_empty() {
        return;
    }
    if obj.font_size.is_nan() || obj.font_size.is_infinite() || obj.font_size <= 0.0 {
        // Skip invalid font size, though unrelated tools might not use it.
    }

    match obj.tool {
        DrawingTool::Rect | DrawingTool::Ellipse | DrawingTool::Line => {
            if obj.points.len() < 2 {
                return;
            }
            tools::shapes::render_shape(scene, obj);
        }
        DrawingTool::Arrow => {
            if obj.points.len() < 2 {
                return;
            }
            tools::arrow::render_arrow(scene, obj);
        }
        DrawingTool::Brush => {
            // Brush can have 1 point (dot)
            tools::freehand::render_brush(scene, obj);
        }
        DrawingTool::Mosaic => {
            tools::effects::render_mosaic(scene, obj);
        }
        DrawingTool::Text => {
            if obj.text.as_ref().map_or(true, |t| t.is_empty()) {
                return;
            }
            tools::text::render_text(scene, obj);
        }
        DrawingTool::Number => {
            // Number might not depend on points for text, but location?
            if obj.points.is_empty() {
                return;
            }
            tools::number::render_number(scene, obj);
        }
        DrawingTool::None => {}
    }
}

fn draw_tool_preview(scene: &mut Scene, state: &OverlayState, toolbar: &Toolbar) {
    let is_over_toolbar = state.mouse_x >= toolbar.rect.left
        && state.mouse_x < toolbar.rect.right
        && state.mouse_y >= toolbar.rect.top
        && state.mouse_y < toolbar.rect.bottom;

    if is_over_toolbar {
        return;
    }

    if matches!(state.current_tool, DrawingTool::Brush | DrawingTool::Mosaic) {
        let is_in_selection = if let Some(sel) = state.selection {
            state.mouse_x >= sel.left
                && state.mouse_x <= sel.right
                && state.mouse_y >= sel.top
                && state.mouse_y <= sel.bottom
        } else {
            true
        };

        if is_in_selection {
            let radius = if state.current_tool == DrawingTool::Brush {
                (state.current_stroke as f64 / 2.0).max(2.0)
            } else {
                20.0
            };

            let circle =
                vello::kurbo::Circle::new((state.mouse_x as f64, state.mouse_y as f64), radius);
            let stroke = vello::kurbo::Stroke::new(1.0);
            let brush = vello::peniko::Brush::Solid(vello::peniko::Color::WHITE);
            scene.stroke(
                &stroke,
                vello::kurbo::Affine::IDENTITY,
                &brush,
                None,
                &circle,
            );
        }
    }
}
