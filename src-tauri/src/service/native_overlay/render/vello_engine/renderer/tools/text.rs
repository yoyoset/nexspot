use crate::service::native_overlay::state::DrawingObject;
use vello::Scene;

pub fn render_text(_scene: &mut Scene, obj: &DrawingObject) {
    if obj.points.is_empty() {
        return;
    }
    if obj.text.as_ref().map_or(true, |s| s.is_empty()) {
        return;
    }

    // TODO: For text in Vello, we need to load a font and render glyphs.
    // For now, we draw a placeholder outline or use a fallback mechanism.
}
