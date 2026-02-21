use super::super::traits::DrawingToolRenderer;
use crate::service::native_overlay::state::DrawingObject;
use crate::service::win32::gdi::{self, SafeHDC};
use crate::service::win32::gdiplus::{self, BrushWrapper, GraphicsWrapper};
use std::collections::HashSet;

pub struct MosaicRenderer;
impl DrawingToolRenderer for MosaicRenderer {
    fn render(
        &self,
        hdc: &SafeHDC,
        src_hdc: Option<&SafeHDC>,
        obj: &DrawingObject,
    ) -> anyhow::Result<()> {
        let src_hdc = match src_hdc {
            Some(s) => s,
            None => return Ok(()),
        };

        if obj.points.is_empty() {
            return Ok(());
        }

        let block_size = 12;
        let brush_radius = 20;

        let graphics = GraphicsWrapper::new(hdc.0)?;
        let mut drawn_blocks = HashSet::new();

        for i in 0..obj.points.len() {
            let p1 = obj.points[i];
            let mut sub_points = vec![p1];
            if i > 0 {
                let p0 = obj.points[i - 1];
                let dx = (p1.0 - p0.0) as f32;
                let dy = (p1.1 - p0.1) as f32;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist > (block_size as f32) / 2.0 {
                    let steps = (dist / ((block_size as f32) / 2.0)) as i32;
                    for step in 1..steps {
                        sub_points.push((
                            p0.0 + (dx * step as f32 / steps as f32) as i32,
                            p0.1 + (dy * step as f32 / steps as f32) as i32,
                        ));
                    }
                }
            }

            for p in sub_points {
                let start_gx = (p.0 - brush_radius) / block_size;
                let end_gx = (p.0 + brush_radius) / block_size;
                let start_gy = (p.1 - brush_radius) / block_size;
                let end_gy = (p.1 + brush_radius) / block_size;

                for gx in start_gx..=end_gx {
                    for gy in start_gy..=end_gy {
                        if drawn_blocks.contains(&(gx, gy)) {
                            continue;
                        }

                        let cx = gx * block_size + block_size / 2;
                        let cy = gy * block_size + block_size / 2;
                        let dx = cx - p.0;
                        let dy = cy - p.1;
                        if dx * dx + dy * dy <= brush_radius * brush_radius {
                            let color = gdi::get_pixel(src_hdc, cx, cy);

                            // Convert GDI COLORREF to GDI+ ARGB
                            let r = (color & 0x000000FF) as u32;
                            let g = (color & 0x0000FF00) >> 8;
                            let b = (color & 0x00FF0000) >> 16;
                            let argb = 0xFF000000 | (r << 16) | (g << 8) | b;

                            let brush = BrushWrapper::new_solid(argb)?;
                            gdiplus::fill_rectangle(
                                &graphics,
                                &brush,
                                (gx * block_size) as f32,
                                (gy * block_size) as f32,
                                block_size as f32,
                                block_size as f32,
                            )?;

                            drawn_blocks.insert((gx, gy));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
