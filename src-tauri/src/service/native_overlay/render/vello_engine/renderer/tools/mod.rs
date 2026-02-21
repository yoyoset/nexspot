use crate::service::native_overlay::state::DrawingObject;
use vello::Scene;

pub trait VelloToolRenderer: Send + Sync {
    fn render(&self, scene: &mut Scene, obj: &DrawingObject);
}

pub mod arrow;
pub mod effects;
pub mod freehand;
pub mod number;
pub mod shapes;
pub mod text;
