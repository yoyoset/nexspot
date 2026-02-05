use super::super::traits::DrawingToolRenderer;
use crate::service::native_overlay::state::DrawingObject;
use crate::service::win32::gdi::{self, SafeHDC};

pub struct BrushRenderer;
impl DrawingToolRenderer for BrushRenderer {
    fn render(&self, hdc: &SafeHDC, obj: &DrawingObject) -> anyhow::Result<()> {
        if obj.points.len() > 1 {
            gdi::move_to(hdc, obj.points[0].0, obj.points[0].1)?;
            for p in obj.points.iter().skip(1) {
                gdi::line_to(hdc, p.0, p.1)?;
            }
        }
        Ok(())
    }
}
