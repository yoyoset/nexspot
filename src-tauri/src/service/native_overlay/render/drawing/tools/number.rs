use super::super::traits::DrawingToolRenderer;
use crate::service::native_overlay::state::DrawingObject;
use crate::service::win32::gdi::SafeHDC;
use crate::service::win32::gdiplus;

pub struct NumberRenderer;
impl DrawingToolRenderer for NumberRenderer {
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
        if obj.points.is_empty() {
            return Ok(());
        }
        let center = obj.points[0];
        let radius = 12.0 + obj.stroke_width;

        let argb = obj.color | 0xFF000000;
        let brush = cache.get_gdiplus_brush(argb)?;

        // 2. Draw Filled Circle
        gdiplus::fill_ellipse(
            graphics,
            &brush,
            center.0 as f32 - radius,
            center.1 as f32 - radius,
            radius * 2.0,
            radius * 2.0,
        )?;

        // 2.5 Draw subtle dark outline stroke for visibility on similar backgrounds
        let pen = cache.get_gdiplus_pen(0xFF444444, 1.0, None)?;
        gdiplus::draw_ellipse(
            graphics,
            &pen,
            center.0 as f32 - radius,
            center.1 as f32 - radius,
            radius * 2.0,
            radius * 2.0,
        )?;

        // 3. Draw Number Text
        if let Some(text) = &obj.text {
            let rgb = obj.color & 0x00FFFFFF;
            let text_color = if rgb == 0x00FFFFFF {
                0xFF000000
            } else {
                0xFFFFFFFF
            };
            let text_brush = cache.get_gdiplus_brush(text_color)?;

            gdiplus::draw_text_centered(
                graphics,
                text,
                (center.0 as f32, center.1 as f32),
                &obj.font_family,
                radius * 1.4, // Slightly larger than radius for good fit
                &text_brush,
                None,
            )?;
        }

        Ok(())
    }
}
