use super::DrawingObject;
use crate::service::native_overlay::state::types::{DrawingTool, HitZone};

impl DrawingObject {
    pub fn hit_test(&self, x: i32, y: i32) -> HitZone {
        let tolerance = 10.0;

        // 1. Special handling for linear tools (Line, Arrow)
        if matches!(self.tool, DrawingTool::Line | DrawingTool::Arrow) && self.points.len() == 2 {
            let p1 = self.points[0];
            let p2 = self.points[1];

            if self.tool == DrawingTool::Arrow {
                // --- ARROW PRECISION HIT TEST ---
                let dx = (p2.0 - p1.0) as f32;
                let dy = (p2.1 - p1.1) as f32;
                let len = (dx * dx + dy * dy).sqrt();

                if len > 0.1 {
                    let ux = dx / len;
                    let uy = dy / len;
                    let px = -uy;
                    let py = ux;

                    let stroke_width = self.stroke_width.max(1.0);
                    let head_len = (stroke_width * 8.0 + 32.0).min(len * 0.9);
                    let head_width = self.head_width.unwrap_or(head_len * 0.82);

                    let wing_dist = head_len;
                    let neck_dist = head_len * 0.75;
                    let neck_width = stroke_width * 2.5 + 8.0;

                    // 1.1 Check 4 Specific Handles
                    if (((x - p1.0).pow(2) + (y - p1.1).pow(2)) as f32).sqrt() < tolerance {
                        return HitZone::Tail;
                    }
                    if (((x - p2.0).pow(2) + (y - p2.1).pow(2)) as f32).sqrt() < tolerance {
                        return HitZone::Tip;
                    }

                    let p_wing_r = (
                        p2.0 as f32 - ux * wing_dist + px * head_width / 2.0,
                        p2.1 as f32 - uy * wing_dist + py * head_width / 2.0,
                    );
                    let p_wing_l = (
                        p2.0 as f32 - ux * wing_dist - px * head_width / 2.0,
                        p2.1 as f32 - uy * wing_dist - py * head_width / 2.0,
                    );

                    if ((x as f32 - p_wing_r.0).powi(2) + (y as f32 - p_wing_r.1).powi(2)).sqrt()
                        < tolerance
                    {
                        return HitZone::WingRight;
                    }
                    if ((x as f32 - p_wing_l.0).powi(2) + (y as f32 - p_wing_l.1).powi(2)).sqrt()
                        < tolerance
                    {
                        return HitZone::WingLeft;
                    }

                    let p_neck_l = (
                        p2.0 as f32 - ux * neck_dist - px * neck_width / 2.0,
                        p2.1 as f32 - uy * neck_dist - py * neck_width / 2.0,
                    );
                    let p_neck_r = (
                        p2.0 as f32 - ux * neck_dist + px * neck_width / 2.0,
                        p2.1 as f32 - uy * neck_dist + py * neck_width / 2.0,
                    );

                    if ((x as f32 - p_neck_r.0).powi(2) + (y as f32 - p_neck_r.1).powi(2)).sqrt()
                        < tolerance
                    {
                        return HitZone::NeckRight;
                    }
                    if ((x as f32 - p_neck_l.0).powi(2) + (y as f32 - p_neck_l.1).powi(2)).sqrt()
                        < tolerance
                    {
                        return HitZone::NeckLeft;
                    }

                    // 1.2 Check Body (Point-in-Polygon)
                    let pts = [
                        (p1.0 as f32, p1.1 as f32),
                        (p_neck_l.0, p_neck_l.1),
                        (p_wing_l.0, p_wing_l.1),
                        (p2.0 as f32, p2.1 as f32),
                        (p_wing_r.0, p_wing_r.1),
                        (p_neck_r.0, p_neck_r.1),
                    ];

                    let mut inside = false;
                    let mut j = pts.len() - 1;
                    for i in 0..pts.len() {
                        if ((pts[i].1 > y as f32) != (pts[j].1 > y as f32))
                            && ((x as f32)
                                < (pts[j].0 - pts[i].0) * (y as f32 - pts[i].1)
                                    / (pts[j].1 - pts[i].1)
                                    + pts[i].0)
                        {
                            inside = !inside;
                        }
                        j = i;
                    }

                    if inside {
                        return HitZone::Body;
                    }

                    let tail_radius = (stroke_width * 1.5).max(4.0);
                    if (((x - p1.0).pow(2) + (y - p1.1).pow(2)) as f32).sqrt() < tail_radius + 2.0 {
                        return HitZone::Body;
                    }
                }
                return HitZone::None;
            }
        }

        // 2. Handling for Rect, Ellipse, Line (2-point objects)
        if matches!(
            self.tool,
            DrawingTool::Rect | DrawingTool::Ellipse | DrawingTool::Line
        ) && self.points.len() == 2
        {
            let p1 = self.points[0];
            let p2 = self.points[1];
            let bounds = self.get_bounds();

            // 2.1 Handle/Endpoint Check (Priority - Always use Bounding Box for handles)
            let zone = HitZone::detect(&bounds, x, y);
            if self.tool == DrawingTool::Line {
                if (((x - p1.0).pow(2) + (y - p1.1).pow(2)) as f32).sqrt() < tolerance {
                    return HitZone::Tail;
                }
                if (((x - p2.0).pow(2) + (y - p2.1).pow(2)) as f32).sqrt() < tolerance {
                    return HitZone::Tip;
                }
            } else {
                // For Rect/Ellipse: Prioritize 8-way resize handles
                if !matches!(zone, HitZone::None | HitZone::Body | HitZone::Stroke) {
                    return zone;
                }
            }

            // 2.2 Math-based Hit Testing
            if self.tool == DrawingTool::Line {
                let dx = (p2.0 - p1.0) as f32;
                let dy = (p2.1 - p1.1) as f32;
                let l2 = dx * dx + dy * dy;
                if l2 > 0.0 {
                    let t =
                        (((x - p1.0) as f32 * dx + (y - p1.1) as f32 * dy) / l2).clamp(0.0, 1.0);
                    let proj_x = p1.0 as f32 + t * dx;
                    let proj_y = p1.1 as f32 + t * dy;
                    let dist = ((x as f32 - proj_x).powi(2) + (y as f32 - proj_y).powi(2)).sqrt();
                    if dist < tolerance {
                        return HitZone::Stroke;
                    }
                }
            } else if self.tool == DrawingTool::Rect {
                let margin = tolerance as i32;
                let is_near_left = (x - bounds.left).abs() < margin;
                let is_near_right = (x - bounds.right).abs() < margin;
                let is_near_top = (y - bounds.top).abs() < margin;
                let is_near_bottom = (y - bounds.bottom).abs() < margin;

                let is_in_x_range = x >= bounds.left - margin && x <= bounds.right + margin;
                let is_in_y_range = y >= bounds.top - margin && y <= bounds.bottom + margin;

                if ((is_near_left || is_near_right) && is_in_y_range)
                    || ((is_near_top || is_near_bottom) && is_in_x_range)
                {
                    return HitZone::Stroke;
                }

                if self.is_filled
                    && x >= bounds.left
                    && x <= bounds.right
                    && y >= bounds.top
                    && y <= bounds.bottom
                {
                    return HitZone::Body;
                }
            } else if self.tool == DrawingTool::Ellipse {
                // ELLIPSE MATH: (x-cx)^2/a^2 + (y-cy)^2/b^2 = 1
                let cx = (bounds.left + bounds.right) as f32 / 2.0;
                let cy = (bounds.top + bounds.bottom) as f32 / 2.0;
                let a = (bounds.right - bounds.left) as f32 / 2.0;
                let b = (bounds.bottom - bounds.top) as f32 / 2.0;

                if a > 0.0 && b > 0.0 {
                    let dx = x as f32 - cx;
                    let dy = y as f32 - cy;

                    // Normalized distance from center (1.0 is on the boundary)
                    let norm_dist_sq = (dx * dx) / (a * a) + (dy * dy) / (b * b);
                    let norm_dist = norm_dist_sq.sqrt();

                    // Stroke Check: Is it near the curve?
                    // Approximate distance in pixels: |norm_dist - 1.0| * min(a,b)
                    let pixel_dist = (norm_dist - 1.0).abs() * a.min(b);
                    if pixel_dist < tolerance {
                        return HitZone::Stroke;
                    }

                    // Body Check: Is it inside? (Only if filled)
                    if self.is_filled && norm_dist <= 1.0 {
                        return HitZone::Body;
                    }
                }
            }
            return HitZone::None;
        }

        if matches!(self.tool, DrawingTool::Number) && !self.points.is_empty() {
            let p = self.points[0];
            let dx = (x - p.0) as f32;
            let dy = (y - p.1) as f32;
            let radius = 18.0; // Consistent with get_bounds
            if (dx * dx + dy * dy).sqrt() <= radius {
                return HitZone::Body;
            }
            return HitZone::None;
        }

        if matches!(self.tool, DrawingTool::Text) {
            let bounds = self.get_bounds();
            let zone = HitZone::detect(&bounds, x, y);
            if !matches!(zone, HitZone::None) {
                return zone;
            }
            return HitZone::None;
        }

        if matches!(self.tool, DrawingTool::Brush | DrawingTool::Mosaic) {
            if self.tool == DrawingTool::Mosaic {
                // Mosaic is non-selectable to avoid caching/performance issues
                return HitZone::None;
            }

            let stroke_radius = (self.stroke_width / 2.0).max(2.0) + tolerance;
            for i in 1..self.points.len() {
                let p1 = self.points[i - 1];
                let p2 = self.points[i];

                let dx = (p2.0 - p1.0) as f32;
                let dy = (p2.1 - p1.1) as f32;
                let l2 = dx * dx + dy * dy;

                if l2 == 0.0 {
                    let dist = (((x - p1.0).pow(2) + (y - p1.1).pow(2)) as f32).sqrt();
                    if dist <= stroke_radius {
                        return HitZone::Body;
                    }
                } else {
                    let t =
                        (((x - p1.0) as f32 * dx + (y - p1.1) as f32 * dy) / l2).clamp(0.0, 1.0);
                    let proj_x = p1.0 as f32 + t * dx;
                    let proj_y = p1.1 as f32 + t * dy;
                    let dist = ((x as f32 - proj_x).powi(2) + (y as f32 - proj_y).powi(2)).sqrt();
                    if dist <= stroke_radius {
                        return HitZone::Body;
                    }
                }
            }
            return HitZone::None;
        }

        let bounds = self.get_bounds();
        HitZone::detect(&bounds, x, y)
    }
}
