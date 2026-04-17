use crate::service::native_overlay::render::toolbar::Toolbar;
use crate::service::native_overlay::state::{DrawingTool, OverlayState};
use vello::Scene;

pub mod tools;
pub mod ui;
pub mod utils;

pub fn render_state_to_scene(
    state: &OverlayState,
    toolbar_obj: Option<&crate::service::native_overlay::render::toolbar::Toolbar>,
    vello_ctx: &crate::service::native_overlay::render::vello_engine::VelloContext,
    scene: &mut Scene,
) {
    scene.reset();

    // 0. (Removed Global Translation)

    // 1. Render Background (v0.2.3 Industrial Scaling)
    // We always render the background in Vello to leverage GPU scaling/rotation robustness,
    // which handles vertical/portrait monitors far better than raw hardware blits.
    if let Some(bg) = &state.vello.background {
        let scale_x = state.width as f64 / bg.width as f64;
        let scale_y = state.height as f64 / bg.height as f64;
        let bg_transform = vello::kurbo::Affine::scale_non_uniform(scale_x, scale_y);
        scene.draw_image(bg, bg_transform);
    } else {
        // Fallback for missing background
        let fill_rect = vello::kurbo::Rect::new(
            state.monitor_rect.left as f64,
            state.monitor_rect.top as f64,
            state.monitor_rect.right as f64,
            state.monitor_rect.bottom as f64,
        );
        scene.fill(
            vello::peniko::Fill::NonZero,
            vello::kurbo::Affine::IDENTITY,
            vello::peniko::Color::from_rgba8(20, 20, 20, 200),
            None,
            &fill_rect,
        );
    }

    let clip_rect = vello::kurbo::Rect::new(-1000000.0, -1000000.0, 1000000.0, 1000000.0);
    let mut inner_scene = vello::Scene::new();

    let mut font_context = vello_ctx.font_context.lock().unwrap_or_else(|e| e.into_inner());
    let mut layout_context = vello_ctx.layout_context.lock().unwrap_or_else(|e| e.into_inner());

    // 3. Render Drawing Objects
    for obj in &state.objects {
        let enable_effects = state.enable_advanced_effects;

        if enable_effects && obj.glow > 0.0 {
            let mut glow_obj = obj.clone();
            let glow_opacity = (obj.glow * 0.4).clamp(0.05, 0.4);
            let glow_spread = obj.glow * 24.0;
            glow_obj.stroke_width += glow_spread;
            let final_glow_opacity = glow_opacity * obj.opacity;

            inner_scene.push_layer(
                vello::peniko::Fill::NonZero,
                vello::peniko::Mix::Normal,
                final_glow_opacity,
                vello::kurbo::Affine::IDENTITY,
                &clip_rect,
            );
            let bg = state.vello.background.as_ref();
            draw_object(&mut inner_scene, &glow_obj, bg, &mut font_context, &mut layout_context);
            inner_scene.pop_layer();
        }

        if enable_effects && obj.has_shadow {
            let mut shadow_obj = obj.clone();
            shadow_obj.color = 0x40000000;
            inner_scene.push_layer(
                vello::peniko::Fill::NonZero,
                vello::peniko::Mix::Normal,
                1.0,
                vello::kurbo::Affine::translate((4.0, 4.0)),
                &clip_rect,
            );
            let bg = state.vello.background.as_ref();
            draw_object(&mut inner_scene, &shadow_obj, bg, &mut font_context, &mut layout_context);
            inner_scene.pop_layer();
        }

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

        let bg = state.vello.background.as_ref();
        draw_object(&mut inner_scene, obj, bg, &mut font_context, &mut layout_context);

        if has_opacity {
            inner_scene.pop_layer();
        }
    }

    // ------------- 关键修改点 -------------
    // 仅在真实渲染环境 (toolbar_obj 有值) 时，绘制操作框与工具栏等 UI 预览
    if let Some(toolbar) = toolbar_obj {
        ui::selection::draw_selection_ui(&mut inner_scene, state, &state.selection);

        // 4. Render Current Interaction (Preview)
        if let Some(current) = &state.current_drawing {
            let enable_effects = state.enable_advanced_effects;

            if enable_effects && current.glow > 0.0 {
                let mut glow_obj = current.clone();
                let glow_opacity = (current.glow * 0.4).clamp(0.05, 0.4);
                let glow_spread = current.glow * 24.0;
                glow_obj.stroke_width += glow_spread;
                let final_glow_opacity = glow_opacity * current.opacity;

                inner_scene.push_layer(
                    vello::peniko::Fill::NonZero,
                    vello::peniko::Mix::Normal,
                    final_glow_opacity,
                    vello::kurbo::Affine::IDENTITY,
                    &clip_rect,
                );
                let bg = state.vello.background.as_ref();
                draw_object(&mut inner_scene, &glow_obj, bg, &mut font_context, &mut layout_context);
                inner_scene.pop_layer();
            }

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

            let bg = state.vello.background.as_ref();
            draw_object(&mut inner_scene, current, bg, &mut font_context, &mut layout_context);

            if has_opacity {
                inner_scene.pop_layer();
            }
        }

        ui::magnifier::draw_magnifier(&mut inner_scene, state);
        ui::handles::draw_object_handles(&mut inner_scene, state);
        ui::toolbar::draw_toolbar_ui(&mut inner_scene, state, toolbar);
        ui::tooltip::draw_tooltip_ui(&mut inner_scene, state, toolbar, &mut font_context, &mut layout_context);
        draw_tool_preview(&mut inner_scene, state, toolbar);
    }
    // ------------------------------------

    // 5. Append Inner Scene with Global Normalization (B-Scheme)
    // Map world coordinates to local window space
    let global_transform = vello::kurbo::Affine::translate((
        -(state.capture_x as f64),
        -(state.capture_y as f64),
    ));
    scene.append(&inner_scene, Some(global_transform));
}

fn draw_object(
    scene: &mut Scene,
    obj: &crate::service::native_overlay::state::DrawingObject,
    bg: Option<&vello::peniko::ImageData>,
    font_context: &mut parley::FontContext,
    layout_context: &mut parley::LayoutContext<[u8; 4]>,
) {
    if obj.points.is_empty() {
        return;
    }

    let mut ctx = tools::VelloRenderContext {
        scene,
        bg,
        font_context,
        layout_context,
    };

    let renderer: &dyn tools::VelloToolRenderer = match obj.tool {
        DrawingTool::Rect | DrawingTool::Ellipse | DrawingTool::Line => &tools::shapes::ShapesRenderer,
        DrawingTool::Arrow => &tools::arrow::ArrowRenderer,
        DrawingTool::Brush => &tools::freehand::BrushRenderer,
        DrawingTool::Mosaic => &tools::effects::MosaicRenderer,
        DrawingTool::Text => &tools::text::TextRenderer,
        DrawingTool::Number => &tools::number::NumberRenderer,
        DrawingTool::None => return,
    };

    renderer.render(&mut ctx, obj);
}

fn draw_tool_preview(scene: &mut Scene, state: &OverlayState, toolbar: &Toolbar) {
    if toolbar.hit_test(state.mouse_x, state.mouse_y) {
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
                let block_size = match state.current_stroke as i32 {
                    0..=3 => 6,
                    4..=7 => 10,
                    _ => 16,
                };
                block_size as f64 * 1.5
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
