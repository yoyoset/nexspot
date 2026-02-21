use crate::service::native_overlay::render::vello_engine::renderer::utils::argb_to_vello;
use crate::service::native_overlay::state::DrawingObject;
use vello::kurbo::{Affine, Circle};
use vello::peniko::{Brush, Fill};
use vello::Scene;

pub fn render_number(scene: &mut Scene, obj: &DrawingObject) {
    if obj.points.is_empty() {
        return;
    }

    let color = argb_to_vello(obj.color);
    let brush = Brush::Solid(color);
    let center = obj.points[0];

    let circle = Circle::new((center.0 as f64, center.1 as f64), 14.0);
    scene.fill(Fill::NonZero, Affine::IDENTITY, &brush, None, &circle);
    // Number label would go here (requires text/glyph support)
}
