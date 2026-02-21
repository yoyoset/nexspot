use crate::service::native_overlay::render::vello_engine::renderer::utils::points_to_rect;
use crate::service::native_overlay::state::DrawingObject;
use vello::kurbo::Affine;
use vello::peniko::{Brush, Color, Fill};
use vello::Scene;

pub fn render_mosaic(scene: &mut Scene, obj: &DrawingObject) {
    if obj.points.len() < 2 {
        return;
    }

    let rect = points_to_rect(obj.points[0], obj.points[1]);
    // Simplified mosaic: a semi-transparent grid or pattern
    let mosaic_brush = Brush::Solid(Color::from_rgba8(128, 128, 128, 180));
    scene.fill(Fill::NonZero, Affine::IDENTITY, &mosaic_brush, None, &rect);
}
