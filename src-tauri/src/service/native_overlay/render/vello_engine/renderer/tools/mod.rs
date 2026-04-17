use crate::service::native_overlay::state::DrawingObject;
use vello::Scene;
use vello::peniko::ImageData; // Added this use statement for ImageData

pub struct VelloRenderContext<'a> {
    pub scene: &'a mut Scene,
    pub bg: Option<&'a ImageData>,
    pub font_context: &'a mut parley::FontContext,
    pub layout_context: &'a mut parley::LayoutContext<[u8; 4]>,
}

pub trait VelloToolRenderer: Send + Sync {
    fn render(&self, ctx: &mut VelloRenderContext, obj: &DrawingObject);
}

pub mod arrow;
pub mod effects;
pub mod freehand;
pub mod number;
pub mod shapes;
pub mod text;
