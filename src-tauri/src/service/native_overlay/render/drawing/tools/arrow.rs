use super::super::traits::DrawingToolRenderer;
use crate::service::native_overlay::state::DrawingObject;
use crate::service::win32::gdi::SafeHDC;
use crate::service::win32::gdiplus::{self, BrushWrapper, GraphicsWrapper};

pub struct ArrowRenderer;
impl DrawingToolRenderer for ArrowRenderer {
    fn render(
        &self,
        hdc: &SafeHDC,
        _src_hdc: Option<&SafeHDC>,
        obj: &DrawingObject,
    ) -> anyhow::Result<()> {
        if obj.points.len() < 2 {
            return Ok(());
        }
        let start = obj.points[0];
        let end = obj.points[1];

        let dx = (end.0 - start.0) as f32;
        let dy = (end.1 - start.1) as f32;
        let len = (dx * dx + dy * dy).sqrt();

        // 1. Parameters - Premium WeChat Style
        let stroke_width = obj.stroke_width.max(1.0);

        // Head Style: Large, Long, and Sharp
        let head_len = (stroke_width * 8.0 + 32.0).min(len * 0.9);
        // NARROWER WINGS: Use 1.0 or 1.1 instead of 1.5
        let head_width = obj.head_width.unwrap_or(head_len * 1.0);

        let wing_dist = head_len;
        let neck_dist = head_len * 0.88;
        let neck_width = stroke_width * 1.8 + 6.0;

        if len < head_len * 0.1 {
            return Ok(());
        }

        let ux = dx / len;
        let uy = dy / len;
        let px = -uy;
        let py = ux;

        // 2. Setup GDI+
        let graphics = GraphicsWrapper::new(hdc.0)?;
        let argb = obj.color | 0xFF000000;

        let brush = BrushWrapper::new_solid(argb)?;

        // 3. Draw Tail Circle (The "Base Dot")
        let tail_radius = (stroke_width * 1.5).max(4.0);
        gdiplus::fill_ellipse(
            &graphics,
            &brush,
            start.0 as f32 - tail_radius,
            start.1 as f32 - tail_radius,
            tail_radius * 2.0,
            tail_radius * 2.0,
        )?;

        // 4. Draw Arrow Polygon (Kite Shape Body)
        // 1. Tip
        let p_tip = (end.0 as f32, end.1 as f32);
        // 2. Right Wing
        let p_rwx = end.0 as f32 - ux * wing_dist + px * head_width / 2.0;
        let p_rwy = end.1 as f32 - uy * wing_dist + py * head_width / 2.0;
        // 3. Right Neck
        let p_rnx = end.0 as f32 - ux * neck_dist + px * neck_width / 2.0;
        let p_rny = end.1 as f32 - uy * neck_dist + py * neck_width / 2.0;
        // 4. Tail Point
        let p_tail = (start.0 as f32, start.1 as f32);
        // 5. Left Neck
        let p_lnx = end.0 as f32 - ux * neck_dist - px * neck_width / 2.0;
        let p_lny = end.1 as f32 - uy * neck_dist - py * neck_width / 2.0;
        // 6. Left Wing
        let p_lwx = end.0 as f32 - ux * wing_dist - px * head_width / 2.0;
        let p_lwy = end.1 as f32 - uy * wing_dist - py * head_width / 2.0;

        let points = [
            p_tail,
            (p_lnx, p_lny),
            (p_lwx, p_lwy),
            p_tip,
            (p_rwx, p_rwy),
            (p_rnx, p_rny),
        ];

        gdiplus::draw_polygon(&graphics, &brush, &points)?;

        Ok(())
    }
}
