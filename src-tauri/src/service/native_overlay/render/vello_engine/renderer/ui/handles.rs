use crate::service::native_overlay::render::vello_engine::renderer::utils::argb_to_vello_alpha;
use crate::service::native_overlay::state::{DrawingTool, HitZone, InteractionMode, OverlayState};
use vello::kurbo::{Affine, Rect, Stroke};
use vello::peniko::{Brush, Color, Fill};
use vello::Scene;

pub fn draw_object_handles(scene: &mut Scene, state: &OverlayState) {
    if let Some(idx) = state.selected_object_index {
        if let Some(obj) = state.objects.get(idx) {
            match obj.tool {
                DrawingTool::Mosaic => {}
                DrawingTool::Brush => {
                    // Just a dashed bounding box for freehand
                    let rect = obj.get_bounds();
                    let border_stroke = Stroke::new(1.5).with_dashes(0.0, [4.0, 4.0]);
                    let border_brush = Brush::Solid(Color::from_rgba8(0, 171, 181, 128)); // Tiffany Blue 50%

                    let vello_rect = Rect::new(
                        rect.left as f64 - 2.0,
                        rect.top as f64 - 2.0,
                        rect.right as f64 + 2.0,
                        rect.bottom as f64 + 2.0,
                    );
                    scene.stroke(
                        &border_stroke,
                        Affine::IDENTITY,
                        &border_brush,
                        None,
                        &vello_rect,
                    );
                }
                DrawingTool::Line if obj.points.len() == 2 => {
                    let p1 = obj.points[0];
                    let p2 = obj.points[1];
                    draw_point(scene, p1.0 as f64, p1.1 as f64, HitZone::Tail, state, obj);
                    draw_point(scene, p2.0 as f64, p2.1 as f64, HitZone::Tip, state, obj);
                }
                DrawingTool::Arrow if obj.points.len() == 2 => {
                    draw_arrow_handles(scene, obj, state);
                }
                DrawingTool::Number => {
                    // Number is fixed size, just draw a bounding box without points to show selection
                    let rect = obj.get_bounds();
                    let border_stroke = Stroke::new(1.0).with_dashes(0.0, [4.0, 4.0]);
                    let border_brush = Brush::Solid(Color::from_rgba8(0, 171, 181, 128)); // Tiffany Blue 50%

                    let vello_rect = Rect::new(
                        rect.left as f64 - 2.0,
                        rect.top as f64 - 2.0,
                        rect.right as f64 + 2.0,
                        rect.bottom as f64 + 2.0,
                    );
                    scene.stroke(
                        &border_stroke,
                        Affine::IDENTITY,
                        &border_brush,
                        None,
                        &vello_rect,
                    );
                }
                _ => {
                    let rect = obj.get_bounds();
                    draw_handles_box(scene, &rect, state, obj);
                }
            }
        }
    }
}

fn draw_point(
    scene: &mut Scene,
    cx: f64,
    cy: f64,
    zone: HitZone,
    state: &OverlayState,
    obj: &crate::service::native_overlay::state::DrawingObject,
) {
    let handle_size = 10.0;
    let radius = handle_size / 2.0;

    let is_hover = state.hover_zone == zone
        || matches!(state.interaction_mode, InteractionMode::Resizing(z) if z == zone)
        || matches!(state.interaction_mode, InteractionMode::TransformingObject(z) if z == zone);

    let tiffany_color = Color::from_rgba8(0, 171, 181, 255);
    let white_color = Color::WHITE;

    let fill_color = if is_hover { tiffany_color } else { white_color };
    let fill_brush = Brush::Solid(fill_color);
    let border_brush = Brush::Solid(tiffany_color);
    let border_stroke = Stroke::new(1.5);

    let _h_rect = Rect::new(cx - radius, cy - radius, cx + radius, cy + radius);
    let circle = vello::kurbo::Circle::new((cx, cy), radius);

    // Apply Style-specific rendering
    use crate::service::config::types::AestheticStyle;
    match obj.style {
        AestheticStyle::Default | AestheticStyle::PaperCut | AestheticStyle::Sketch | AestheticStyle::Glass => {
            scene.fill(Fill::NonZero, Affine::IDENTITY, &fill_brush, None, &circle);
            scene.stroke(
                &border_stroke,
                Affine::IDENTITY,
                &border_brush,
                None,
                &circle,
            );
        }
        AestheticStyle::Neon => {
            // Add glow to handles
            let glow_stroke = Stroke::new(4.0);
            let glow_color = argb_to_vello_alpha(obj.color, 0.6);
            let glow_brush = Brush::Solid(glow_color);
            scene.stroke(&glow_stroke, Affine::IDENTITY, &glow_brush, None, &circle);

            scene.fill(Fill::NonZero, Affine::IDENTITY, &fill_brush, None, &circle);
            scene.stroke(
                &border_stroke,
                Affine::IDENTITY,
                &border_brush,
                None,
                &circle,
            );
        }
    }
}

fn draw_handles_box(
    scene: &mut Scene,
    rect: &windows::Win32::Foundation::RECT,
    state: &OverlayState,
    obj: &crate::service::native_overlay::state::DrawingObject,
) {
    let left = rect.left as f64;
    let top = rect.top as f64;
    let right = rect.right as f64;
    let bottom = rect.bottom as f64;
    let mid_x = left + (right - left) / 2.0;
    let mid_y = top + (bottom - top) / 2.0;

    // Connectors
    let line_brush = Brush::Solid(Color::from_rgba8(0, 171, 181, 128));
    let line_stroke = Stroke::new(1.0);
    let box_rect = Rect::new(left, top, right, bottom);
    scene.stroke(&line_stroke, Affine::IDENTITY, &line_brush, None, &box_rect);

    // Points
    draw_point(scene, left, top, HitZone::TopLeft, state, obj);
    draw_point(scene, mid_x, top, HitZone::Top, state, obj);
    draw_point(scene, right, top, HitZone::TopRight, state, obj);
    draw_point(scene, right, mid_y, HitZone::Right, state, obj);
    draw_point(scene, right, bottom, HitZone::BottomRight, state, obj);
    draw_point(scene, mid_x, bottom, HitZone::Bottom, state, obj);
    draw_point(scene, left, bottom, HitZone::BottomLeft, state, obj);
    draw_point(scene, left, mid_y, HitZone::Left, state, obj);
}

fn draw_arrow_handles(
    scene: &mut Scene,
    obj: &crate::service::native_overlay::state::DrawingObject,
    state: &OverlayState,
) {
    let p1 = obj.points[0];
    let p2 = obj.points[1];
    let dx = (p2.0 - p1.0) as f64;
    let dy = (p2.1 - p1.1) as f64;
    let len = (dx * dx + dy * dy).sqrt();

    if len < 0.1 {
        return;
    }

    let ux = dx / len;
    let uy = dy / len;
    let px = -uy;
    let py = ux;

    let stroke_width = obj.stroke_width.max(1.0) as f64;
    let head_len = (stroke_width * 8.0 + 32.0).min(len * 0.9);
    let head_width = obj.head_width.unwrap_or(head_len as f32 * 0.82) as f64;
    let wing_dist = head_len;
    let neck_dist = head_len * 0.75;
    let neck_width = stroke_width * 2.5 + 8.0;

    let p_neck_l = (
        p2.0 as f64 - ux * neck_dist - px * neck_width / 2.0,
        p2.1 as f64 - uy * neck_dist - py * neck_width / 2.0,
    );
    let p_neck_r = (
        p2.0 as f64 - ux * neck_dist + px * neck_width / 2.0,
        p2.1 as f64 - uy * neck_dist + py * neck_width / 2.0,
    );
    let wr_x = p2.0 as f64 - ux * wing_dist + px * head_width / 2.0;
    let wr_y = p2.1 as f64 - uy * wing_dist + py * head_width / 2.0;
    let wl_x = p2.0 as f64 - ux * wing_dist - px * head_width / 2.0;
    let wl_y = p2.1 as f64 - uy * wing_dist - py * head_width / 2.0;

    // Connections
    let mut path = vello::kurbo::BezPath::new();
    path.move_to((p1.0 as f64, p1.1 as f64));
    path.line_to((p_neck_l.0, p_neck_l.1));
    path.line_to((wl_x, wl_y));
    path.line_to((p2.0 as f64, p2.1 as f64));
    path.line_to((wr_x, wr_y));
    path.line_to((p_neck_r.0, p_neck_r.1));
    path.close_path();

    let line_brush = Brush::Solid(Color::from_rgba8(0, 171, 181, 128));
    let line_stroke = Stroke::new(1.0);
    scene.stroke(&line_stroke, Affine::IDENTITY, &line_brush, None, &path);

    // Points
    draw_point(scene, p1.0 as f64, p1.1 as f64, HitZone::Tail, state, obj);
    draw_point(scene, p2.0 as f64, p2.1 as f64, HitZone::Tip, state, obj);
    draw_point(scene, wr_x, wr_y, HitZone::WingRight, state, obj);
    draw_point(scene, wl_x, wl_y, HitZone::WingLeft, state, obj);
    draw_point(
        scene,
        p_neck_r.0,
        p_neck_r.1,
        HitZone::NeckRight,
        state,
        obj,
    );
    draw_point(scene, p_neck_l.0, p_neck_l.1, HitZone::NeckLeft, state, obj);
}
