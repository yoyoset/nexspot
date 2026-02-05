use super::super::traits::DrawingToolRenderer;
use crate::service::native_overlay::state::DrawingObject;
use crate::service::win32::gdi::{self, SafeHDC};

pub struct ArrowRenderer;
impl DrawingToolRenderer for ArrowRenderer {
    fn render(&self, hdc: &SafeHDC, obj: &DrawingObject) -> anyhow::Result<()> {
        if obj.points.len() >= 2 {
            let start = obj.points[0];
            let end = obj.points[1];
            gdi::move_to(hdc, start.0, start.1)?;
            gdi::line_to(hdc, end.0, end.1)?;

            // Draw Arrow Head
            let angle = (end.1 as f32 - start.1 as f32).atan2(end.0 as f32 - start.0 as f32);
            let head_len = 15.0;
            let angle1 = angle + std::f32::consts::PI * 0.85;
            let angle2 = angle - std::f32::consts::PI * 0.85;

            let p1x = end.0 as f32 + head_len * angle1.cos();
            let p1y = end.1 as f32 + head_len * angle1.sin();
            let p2x = end.0 as f32 + head_len * angle2.cos();
            let p2y = end.1 as f32 + head_len * angle2.sin();

            gdi::move_to(hdc, end.0, end.1)?;
            gdi::line_to(hdc, p1x as i32, p1y as i32)?;
            gdi::move_to(hdc, end.0, end.1)?;
            gdi::line_to(hdc, p2x as i32, p2y as i32)?;
        }
        Ok(())
    }
}
