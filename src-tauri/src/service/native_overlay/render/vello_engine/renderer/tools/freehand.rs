use crate::service::native_overlay::render::vello_engine::renderer::utils::argb_to_vello;
use crate::service::native_overlay::state::DrawingObject;
use vello::kurbo::{Affine, BezPath, Stroke};
use vello::peniko::Brush;
use vello::Scene;

pub fn render_brush(scene: &mut Scene, obj: &DrawingObject) {
    if obj.points.len() < 2 {
        return;
    }

    let color = argb_to_vello(obj.color);
    let brush = Brush::Solid(color);
    let mut stroke = Stroke::new(obj.stroke_width as f64);
    if obj.is_dashed {
        stroke = stroke.with_dashes(10.0, [10.0, 5.0]);
    }

    let mut path = BezPath::new();
    path.move_to((obj.points[0].0 as f64, obj.points[0].1 as f64));
    for p in &obj.points[1..] {
        path.line_to((p.0 as f64, p.1 as f64));
    }
    scene.stroke(&stroke, Affine::IDENTITY, &brush, None, &path);
}
