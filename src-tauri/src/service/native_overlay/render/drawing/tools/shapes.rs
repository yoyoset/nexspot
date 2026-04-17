use super::super::traits::DrawingToolRenderer;
use crate::service::native_overlay::state::DrawingObject;
use crate::service::win32::gdi::SafeHDC;
use crate::service::win32::gdiplus;

pub struct RectRenderer;
impl DrawingToolRenderer for RectRenderer {
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
            let start = obj.points[0];
            let end = obj.points[1];
            let left = start.0.min(end.0) as f32;
            let top = start.1.min(end.1) as f32;
            let width = (start.0 - end.0).abs() as f32;
            let height = (start.1 - end.1).abs() as f32;

            if width > 0.1 && height > 0.1 {
                let argb = obj.color | 0xFF000000;
                
                // Draw filled if alpha/opacity logic allows, but here we usually draw stroke + optional fill
                // For now, follow the industrial standard: always stroke.
                let pen = cache.get_gdiplus_pen(argb, obj.stroke_width, None)?;
                
                // If we want to support filled shapes later, we'd use brush here.
                // For now, let's just ensure we use standard pen.
                gdiplus::draw_rectangle(graphics, &pen, left, top, width, height)?;
            }
        }
        Ok(())
    }
}

pub struct EllipseRenderer;
impl DrawingToolRenderer for EllipseRenderer {
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
            let start = obj.points[0];
            let end = obj.points[1];
            let left = start.0.min(end.0) as f32;
            let top = start.1.min(end.1) as f32;
            let width = (start.0 - end.0).abs() as f32;
            let height = (start.1 - end.1).abs() as f32;

            if width > 0.1 && height > 0.1 {
                let argb = obj.color | 0xFF000000;
                let pen = cache.get_gdiplus_pen(argb, obj.stroke_width, None)?;
                gdiplus::draw_ellipse(graphics, &pen, left, top, width, height)?;
            }
        }
        Ok(())
    }
}

pub struct LineRenderer;
impl DrawingToolRenderer for LineRenderer {
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
            gdiplus::draw_line(
                graphics,
                &pen,
                obj.points[0].0 as f32,
                obj.points[0].1 as f32,
                obj.points[1].0 as f32,
                obj.points[1].1 as f32,
            )?;
        }
        Ok(())
    }
}

