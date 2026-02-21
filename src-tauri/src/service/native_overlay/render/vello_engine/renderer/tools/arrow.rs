use crate::service::native_overlay::render::vello_engine::renderer::utils::argb_to_vello;
use crate::service::native_overlay::state::DrawingObject;
use vello::kurbo::{Affine, BezPath, Circle};
use vello::peniko::{Brush, Fill};
use vello::Scene;

pub fn render_arrow(scene: &mut Scene, obj: &DrawingObject) {
    if obj.points.len() < 2 {
        return;
    }

    let color = argb_to_vello(obj.color);
    let brush = Brush::Solid(color);
    let start = obj.points[0];
    let end = obj.points[1];

    let dx = (end.0 - start.0) as f64;
    let dy = (end.1 - start.1) as f64;
    let len = (dx * dx + dy * dy).sqrt();

    let stroke_width = obj.stroke_width as f64;
    let head_len = (stroke_width * 8.0 + 32.0).min(len * 0.9);
    let head_width = obj.head_width.unwrap_or(head_len as f32 * 1.0) as f64;
    let wing_dist = head_len;
    let neck_dist = head_len * 0.88;
    let neck_width = stroke_width * 1.8 + 6.0;

    if len > 1.0 {
        let ux = dx / len;
        let uy = dy / len;
        let px = -uy;
        let py = ux;

        // 1. Tail Circle
        let tail_radius = (stroke_width * 1.5).max(4.0);
        let circle = Circle::new((start.0 as f64, start.1 as f64), tail_radius);
        scene.fill(Fill::NonZero, Affine::IDENTITY, &brush, None, &circle);

        // 2. Arrow Body
        let mut path = BezPath::new();
        path.move_to((end.0 as f64, end.1 as f64));
        path.line_to((
            end.0 as f64 - ux * wing_dist + px * head_width / 2.0,
            end.1 as f64 - uy * wing_dist + py * head_width / 2.0,
        ));
        path.line_to((
            end.0 as f64 - ux * neck_dist + px * neck_width / 2.0,
            end.1 as f64 - uy * neck_dist + py * neck_width / 2.0,
        ));
        path.line_to((start.0 as f64, start.1 as f64));
        path.line_to((
            end.0 as f64 - ux * neck_dist - px * neck_width / 2.0,
            end.1 as f64 - uy * neck_dist - py * neck_width / 2.0,
        ));
        path.line_to((
            end.0 as f64 - ux * wing_dist - px * head_width / 2.0,
            end.1 as f64 - uy * wing_dist - py * head_width / 2.0,
        ));
        path.close_path();

        scene.fill(Fill::NonZero, Affine::IDENTITY, &brush, None, &path);
    }
}
