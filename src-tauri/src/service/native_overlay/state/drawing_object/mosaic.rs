use super::DrawingObject;
use crate::service::native_overlay::state::types::DrawingTool;

impl DrawingObject {
    pub fn process_mosaic_pending_points<F>(&mut self, max_points: usize, mut sampler: F) -> usize
    where
        F: FnMut(f64, f64) -> u32,
    {
        if self.tool != DrawingTool::Mosaic {
            return 0;
        }

        let mut processed = 0;

        // S(2.0) -> 8, M(4.0) -> 16, L(8.0) -> 32
        let block_size = match self.stroke_width as i32 {
            0..=3 => 6,
            4..=7 => 10,
            _ => 16,
        };
        let block_size_f = block_size as f64;
        let brush_radius = block_size_f * 1.5;

        while processed < max_points {
            let p1 = match self.mosaic_pending_points.pop_front() {
                Some(p) => p,
                None => break,
            };

            let mut sub_points = vec![p1];
            if let Some(p0) = self.mosaic_last_pos {
                let dx = p1.0 - p0.0;
                let dy = p1.1 - p0.1;
                let dist = ((dx * dx + dy * dy) as f64).sqrt();
                let step_dist = 2.0;

                if dist > step_dist {
                    let steps = (dist / step_dist) as i32;
                    for step in 1..steps {
                        sub_points.push((p0.0 + (dx * step / steps), p0.1 + (dy * step / steps)));
                    }
                }
            }
            self.mosaic_last_pos = Some(p1);

            for p in sub_points {
                let px = p.0 as f64;
                let py = p.1 as f64;

                let start_gx = ((px - brush_radius) / block_size_f).floor() as i32;
                let end_gx = ((px + brush_radius) / block_size_f).floor() as i32;
                let start_gy = ((py - brush_radius) / block_size_f).floor() as i32;
                let end_gy = ((py + brush_radius) / block_size_f).floor() as i32;

                for gx in start_gx..=end_gx {
                    for gy in start_gy..=end_gy {
                        let bx = gx as f64 * block_size_f;
                        let by = gy as f64 * block_size_f;
                        let cx = bx + block_size_f / 2.0;
                        let cy = by + block_size_f / 2.0;

                        let dx = cx - px;
                        let dy = cy - py;

                        if dx * dx + dy * dy <= brush_radius * brush_radius {
                            if !self.mosaic_blocks.contains_key(&(gx, gy)) {
                                let color = sampler(cx, cy);
                                self.mosaic_blocks.insert((gx, gy), color);
                            }
                        }
                    }
                }
            }
            processed += 1;
        }

        processed
    }
}
