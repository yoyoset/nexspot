use super::super::traits::DrawingToolRenderer;
use crate::service::native_overlay::state::DrawingObject;
use crate::service::win32::gdi::SafeHDC;
use crate::service::win32::gdiplus;

pub struct BrushRenderer;
impl DrawingToolRenderer for BrushRenderer {
    fn render(
        &self,
        hdc: &SafeHDC,
        graphics: Option<&crate::service::win32::gdiplus::GraphicsWrapper>,
        _src_hdc: Option<&SafeHDC>,
        cache: &mut crate::service::win32::gdi::GdiCache,
        obj: &DrawingObject,
    ) -> anyhow::Result<()> {
        if let Some(g) = graphics {
            self.render_gdiplus(g, cache, obj)
        } else {
            let g = crate::service::win32::gdiplus::GraphicsWrapper::new(hdc.0)?;
            self.render_gdiplus(&g, cache, obj)
        }
    }

    fn render_gdiplus(
        &self,
        graphics: &crate::service::win32::gdiplus::GraphicsWrapper,
        cache: &mut crate::service::win32::gdi::GdiCache,
        obj: &DrawingObject,
    ) -> anyhow::Result<()> {
        if obj.points.len() > 1 {
            let argb = obj.color | 0xFF000000;
            let pen = cache.get_gdiplus_pen(argb, obj.stroke_width, None)?;

            let points: Vec<(f32, f32)> = obj
                .points
                .iter()
                .map(|&(x, y)| (x as f32, y as f32))
                .collect();
            gdiplus::draw_curve(graphics, &pen, &points)?;
        }
        Ok(())
    }
}

