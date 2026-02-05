use super::super::traits::DrawingToolRenderer;
use crate::service::native_overlay::state::DrawingObject;
use crate::service::win32::gdi::{self, SafeHDC, RECT};

pub struct MosaicRenderer;
impl DrawingToolRenderer for MosaicRenderer {
    fn render(&self, hdc: &SafeHDC, obj: &DrawingObject) -> anyhow::Result<()> {
        if obj.points.len() > 1 {
            for p in &obj.points {
                let size = 20; // Mosaic Brush size
                let rect = RECT {
                    left: p.0 - size,
                    top: p.1 - size,
                    right: p.0 + size,
                    bottom: p.1 + size,
                };
                let _ = gdi::pixelate_rect(hdc, &rect, 10);
            }
        }
        Ok(())
    }
}
