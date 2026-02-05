use super::super::traits::DrawingToolRenderer;
use crate::service::native_overlay::state::DrawingObject;
use crate::service::win32::gdi::{self, SafeHDC};

pub struct RectRenderer;
impl DrawingToolRenderer for RectRenderer {
    fn render(&self, hdc: &SafeHDC, obj: &DrawingObject) -> anyhow::Result<()> {
        if obj.points.len() >= 2 {
            let start = obj.points[0];
            let end = obj.points[1];
            let left = start.0.min(end.0);
            let top = start.1.min(end.1);
            let right = start.0.max(end.0);
            let bottom = start.1.max(end.1);
            gdi::rectangle(hdc, left, top, right, bottom)?;
        }
        Ok(())
    }
}

pub struct EllipseRenderer;
impl DrawingToolRenderer for EllipseRenderer {
    fn render(&self, hdc: &SafeHDC, obj: &DrawingObject) -> anyhow::Result<()> {
        if obj.points.len() >= 2 {
            let start = obj.points[0];
            let end = obj.points[1];
            gdi::ellipse(hdc, start.0, start.1, end.0, end.1)?;
        }
        Ok(())
    }
}

pub struct LineRenderer;
impl DrawingToolRenderer for LineRenderer {
    fn render(&self, hdc: &SafeHDC, obj: &DrawingObject) -> anyhow::Result<()> {
        if obj.points.len() >= 2 {
            gdi::move_to(hdc, obj.points[0].0, obj.points[0].1)?;
            gdi::line_to(hdc, obj.points[1].0, obj.points[1].1)?;
        }
        Ok(())
    }
}
