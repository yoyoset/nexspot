use super::types::{DrawingTool, HitZone};
use windows::Win32::Foundation::RECT;

#[derive(Debug, Clone)]
pub struct DrawingObject {
    pub tool: DrawingTool,
    pub points: Vec<(i32, i32)>,
    pub text: Option<String>,
    pub color: u32,
    pub stroke_width: f32,
    pub font_size: f32,
    pub is_filled: bool,
    pub is_dashed: bool,
    pub is_editing_text: bool,
    pub font_family: String,
    pub head_width: Option<f32>, // Optional custom head width for arrows
    pub opacity: f32,
    pub has_shadow: bool,
    pub glow: f32, // 0.0 to 1.0 (Simulated Outer Glow)
}

impl DrawingObject {
    pub fn get_bounds(&self) -> RECT {
        if self.points.is_empty() {
            return RECT::default();
        }

        if matches!(self.tool, DrawingTool::Text) {
            let p = self.points[0];
            let font_size = self.font_size;
            let text_content = self.text.as_deref().unwrap_or("");
            // Rough estimation of text width: average char width is about 0.6 * font_size for Segoe UI
            let width = (text_content.len().max(1) as f32 * font_size * 0.6) as i32;
            let height = font_size as i32;

            return RECT {
                left: p.0,
                top: p.1,
                right: p.0 + width,
                bottom: p.1 + height,
            };
        }

        let mut min_x = self.points[0].0;
        let mut min_y = self.points[0].1;
        let mut max_x = self.points[0].0;
        let mut max_y = self.points[0].1;

        for p in &self.points {
            min_x = min_x.min(p.0);
            min_y = min_y.min(p.1);
            max_x = max_x.max(p.0);
            max_y = max_y.max(p.1);
        }

        RECT {
            left: min_x,
            top: min_y,
            right: max_x,
            bottom: max_y,
        }
    }

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
                    let head_width = self.head_width.unwrap_or(head_len * 1.0);

                    let wing_dist = head_len;
                    let neck_dist = head_len * 0.88;
                    let neck_width = stroke_width * 1.8 + 6.0;

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

                    // 1.2 Check Body (Point-in-Polygon)
                    let pts = [
                        (p1.0 as f32, p1.1 as f32),
                        (
                            p2.0 as f32 - ux * neck_dist - px * neck_width / 2.0,
                            p2.1 as f32 - uy * neck_dist - py * neck_width / 2.0,
                        ),
                        (p_wing_l.0, p_wing_l.1),
                        (p2.0 as f32, p2.1 as f32),
                        (p_wing_r.0, p_wing_r.1),
                        (
                            p2.0 as f32 - ux * neck_dist + px * neck_width / 2.0,
                            p2.1 as f32 - uy * neck_dist + py * neck_width / 2.0,
                        ),
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
            } else {
                let dx = (p2.0 - p1.0) as f32;
                let dy = (p2.1 - p1.1) as f32;
                let l2 = dx * dx + dy * dy;

                if l2 == 0.0 {
                    let dist = (((x - p1.0).pow(2) + (y - p1.1).pow(2)) as f32).sqrt();
                    if dist < tolerance {
                        return HitZone::Body;
                    }
                } else {
                    let t =
                        (((x - p1.0) as f32 * dx + (y - p1.1) as f32 * dy) / l2).clamp(0.0, 1.0);
                    let proj_x = p1.0 as f32 + t * dx;
                    let proj_y = p1.1 as f32 + t * dy;

                    let dist = ((x as f32 - proj_x).powi(2) + (y as f32 - proj_y).powi(2)).sqrt();
                    if dist < tolerance {
                        return HitZone::Body;
                    }
                }
            }

            return HitZone::None;
        }

        if matches!(self.tool, DrawingTool::Rect | DrawingTool::Ellipse) && self.points.len() == 2 {
            let bounds = self.get_bounds();
            let zone = HitZone::detect(&bounds, x, y);

            match zone {
                HitZone::None => return HitZone::None,
                HitZone::Body => {
                    if self.is_filled {
                        return HitZone::Body;
                    } else {
                        return HitZone::None;
                    }
                }
                _ => return zone,
            }
        }

        if matches!(self.tool, DrawingTool::Text | DrawingTool::Mosaic) {
            let bounds = self.get_bounds();
            let zone = HitZone::detect(&bounds, x, y);
            if !matches!(zone, HitZone::None) {
                return zone;
            }
            return HitZone::None;
        }

        let bounds = self.get_bounds();
        HitZone::detect(&bounds, x, y)
    }
}
