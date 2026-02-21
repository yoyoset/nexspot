use super::super::traits::DrawingToolRenderer;
use crate::service::native_overlay::state::DrawingObject;
use crate::service::win32::gdi::SafeHDC;
use crate::service::win32::gdiplus::{self, BrushWrapper, GraphicsWrapper};

pub struct NumberRenderer;
impl DrawingToolRenderer for NumberRenderer {
    fn render(
        &self,
        hdc: &SafeHDC,
        _src_hdc: Option<&SafeHDC>,
        obj: &DrawingObject,
    ) -> anyhow::Result<()> {
        if obj.points.is_empty() {
            return Ok(());
        }
        let center = obj.points[0];
        let radius = 12.0 + obj.stroke_width;

        // 1. Setup GDI+
        let graphics = GraphicsWrapper::new(hdc.0)?;
        let argb = obj.color | 0xFF000000;
        let brush = BrushWrapper::new_solid(argb)?;

        // 2. Draw Filled Circle
        gdiplus::fill_ellipse(
            &graphics,
            &brush,
            center.0 as f32 - radius,
            center.1 as f32 - radius,
            radius * 2.0,
            radius * 2.0,
        )?;

        // 3. Draw Number Text
        if let Some(text) = &obj.text {
            let white_brush = BrushWrapper::new_solid(0xFFFFFFFF)?;
            gdiplus::draw_text_centered(
                &graphics,
                text,
                (center.0 as f32, center.1 as f32),
                &obj.font_family,
                radius * 1.4, // Slightly larger than radius for good fit
                &white_brush,
                None,
            )?;
        }

        Ok(())
    }
}
