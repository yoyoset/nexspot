use super::super::traits::DrawingToolRenderer;
use crate::service::native_overlay::state::DrawingObject;
use crate::service::win32::gdi::SafeHDC;
use crate::service::win32::gdiplus::{self, GraphicsWrapper, PenWrapper};

pub struct BrushRenderer;
impl DrawingToolRenderer for BrushRenderer {
    fn render(
        &self,
        hdc: &SafeHDC,
        _src_hdc: Option<&SafeHDC>,
        obj: &DrawingObject,
    ) -> anyhow::Result<()> {
        if obj.points.len() > 1 {
            let graphics = GraphicsWrapper::new(hdc.0)?;
            let argb = obj.color | 0xFF000000;
            let pen = PenWrapper::new(argb, obj.stroke_width)?;

            let points: Vec<(f32, f32)> = obj
                .points
                .iter()
                .map(|&(x, y)| (x as f32, y as f32))
                .collect();
            gdiplus::draw_lines(&graphics, &pen, &points)?;
        }
        Ok(())
    }
}
