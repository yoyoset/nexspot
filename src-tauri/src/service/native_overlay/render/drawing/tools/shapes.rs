use super::super::traits::DrawingToolRenderer;
use crate::service::native_overlay::state::DrawingObject;
use crate::service::win32::gdi::SafeHDC;
use crate::service::win32::gdiplus::{self, BrushWrapper, GraphicsWrapper, PenWrapper};

pub struct RectRenderer;
impl DrawingToolRenderer for RectRenderer {
    fn render(
        &self,
        hdc: &SafeHDC,
        _src_hdc: Option<&SafeHDC>,
        obj: &DrawingObject,
    ) -> anyhow::Result<()> {
        if obj.points.len() >= 2 {
            let start = obj.points[0];
            let end = obj.points[1];
            let left = start.0.min(end.0) as f32;
            let top = start.1.min(end.1) as f32;
            let width = (start.0 - end.0).abs() as f32;
            let height = (start.1 - end.1).abs() as f32;

            let graphics = GraphicsWrapper::new(hdc.0)?;
            let argb = obj.color | 0xFF000000;

            if obj.is_filled {
                let brush = BrushWrapper::new_solid(argb)?;
                gdiplus::fill_rectangle(&graphics, &brush, left, top, width, height)?;
            } else {
                let pen = PenWrapper::new(argb, obj.stroke_width)?;
                gdiplus::draw_rectangle(&graphics, &pen, left, top, width, height)?;
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
        _src_hdc: Option<&SafeHDC>,
        obj: &DrawingObject,
    ) -> anyhow::Result<()> {
        if obj.points.len() >= 2 {
            let start = obj.points[0];
            let end = obj.points[1];
            let left = start.0.min(end.0) as f32;
            let top = start.1.min(end.1) as f32;
            let width = (start.0 - end.0).abs() as f32;
            let height = (start.1 - end.1).abs() as f32;

            let graphics = GraphicsWrapper::new(hdc.0)?;
            let argb = obj.color | 0xFF000000;

            if obj.is_filled {
                let brush = BrushWrapper::new_solid(argb)?;
                gdiplus::fill_ellipse(&graphics, &brush, left, top, width, height)?;
            } else {
                let pen = PenWrapper::new(argb, obj.stroke_width)?;
                gdiplus::draw_ellipse(&graphics, &pen, left, top, width, height)?;
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
        _src_hdc: Option<&SafeHDC>,
        obj: &DrawingObject,
    ) -> anyhow::Result<()> {
        if obj.points.len() >= 2 {
            let start = obj.points[0];
            let end = obj.points[1];

            let graphics = GraphicsWrapper::new(hdc.0)?;
            let argb = obj.color | 0xFF000000;
            let pen = PenWrapper::new(argb, obj.stroke_width)?;

            gdiplus::draw_line(
                &graphics,
                &pen,
                start.0 as f32,
                start.1 as f32,
                end.0 as f32,
                end.1 as f32,
            )?;
        }
        Ok(())
    }
}
