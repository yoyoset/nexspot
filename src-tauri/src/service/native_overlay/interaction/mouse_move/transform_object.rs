use crate::service::native_overlay::state::{DrawingTool, HitZone, OverlayState};

pub fn handle(state: &mut OverlayState, zone: HitZone, x: i32, y: i32, dx: i32, dy: i32) {
    if let Some(idx) = state.selected_object_index {
        state.start_x = x;
        state.start_y = y;

        if let Some(obj) = state.objects.get_mut(idx) {
            let is_arrow = obj.tool == DrawingTool::Arrow;

            if is_arrow && obj.points.len() == 2 {
                match zone {
                    HitZone::Body | HitZone::Stroke => {
                        for p in &mut obj.points {
                            p.0 += dx;
                            p.1 += dy;
                        }
                    }
                    HitZone::Tail => {
                        obj.points[0].0 += dx;
                        obj.points[0].1 += dy;
                    }
                    HitZone::Tip => {
                        obj.points[1].0 += dx;
                        obj.points[1].1 += dy;
                    }
                    HitZone::WingLeft | HitZone::WingRight => {
                        let p1 = obj.points[0];
                        let p2 = obj.points[1];
                        let adx = (p2.0 - p1.0) as f32;
                        let ady = (p2.1 - p1.1) as f32;
                        let len = (adx * adx + ady * ady).sqrt();
                        if len > 1.0 {
                            let ux = adx / len;
                            let uy = ady / len;
                            let px = -uy;
                            let py = ux;

                            let mdx = x as f32 - p2.0 as f32;
                            let mdy = y as f32 - p2.1 as f32;
                            let dist_h = (mdx * px + mdy * py).abs();

                            // head_width = distance from axis * 2
                            obj.head_width = Some(dist_h * 2.0);
                        }
                    }
                    HitZone::NeckLeft | HitZone::NeckRight => {
                        let p1 = obj.points[0];
                        let p2 = obj.points[1];
                        let adx = (p2.0 - p1.0) as f32;
                        let ady = (p2.1 - p1.1) as f32;
                        let len = (adx * adx + ady * ady).sqrt();
                        if len > 1.0 {
                            let ux = adx / len;
                            let uy = ady / len;
                            let px = -uy;
                            let py = ux;

                            let mdx = x as f32 - p2.0 as f32;
                            let mdy = y as f32 - p2.1 as f32;
                            let dist_h = (mdx * px + mdy * py).abs();

                            // neck_width = stroke_width * 2.5 + 8.0
                            // reverse: stroke_width = (neck_width - 8.0) / 2.5
                            let new_stroke = ((dist_h * 2.0 - 8.0) / 2.5).clamp(1.0, 50.0);
                            obj.stroke_width = new_stroke;
                        }
                    }
                    _ => {}
                }
            } else {
                let bounds = obj.get_raw_bounds();
                let is_2point = obj.points.len() == 2;
                let p0_orig = obj.points[0];
                let p1_orig = if is_2point { Some(obj.points[1]) } else { None };

                for p in &mut obj.points {
                    match zone {
                        HitZone::Body | HitZone::Stroke => {
                            p.0 += dx;
                            p.1 += dy;
                        }
                        HitZone::Tail if is_2point => {
                            // First point for 2rd logic
                            if p.0 == p0_orig.0 && p.1 == p0_orig.1 {
                                p.0 += dx;
                                p.1 += dy;
                            }
                        }
                        HitZone::Tip if is_2point => {
                            // Second point
                            if let Some(p1) = p1_orig {
                                if p.0 == p1.0 && p.1 == p1.1 {
                                    p.0 += dx;
                                    p.1 += dy;
                                }
                            }
                        }
                        _ if is_2point => {
                            let is_top = p.1 == bounds.top;
                            let is_bottom = p.1 == bounds.bottom;
                            let is_left = p.0 == bounds.left;
                            let is_right = p.0 == bounds.right;

                            match zone {
                                HitZone::TopLeft => {
                                    if is_left {
                                        p.0 += dx;
                                    }
                                    if is_top {
                                        p.1 += dy;
                                    }
                                }
                                HitZone::TopRight => {
                                    if is_right {
                                        p.0 += dx;
                                    }
                                    if is_top {
                                        p.1 += dy;
                                    }
                                }
                                HitZone::BottomLeft => {
                                    if is_left {
                                        p.0 += dx;
                                    }
                                    if is_bottom {
                                        p.1 += dy;
                                    }
                                }
                                HitZone::BottomRight => {
                                    if is_right {
                                        p.0 += dx;
                                    }
                                    if is_bottom {
                                        p.1 += dy;
                                    }
                                }
                                HitZone::Top if is_top => {
                                    p.1 += dy;
                                }
                                HitZone::Bottom if is_bottom => {
                                    p.1 += dy;
                                }
                                HitZone::Left if is_left => {
                                    p.0 += dx;
                                }
                                HitZone::Right if is_right => {
                                    p.0 += dx;
                                }
                                _ => {}
                            }
                        }
                        _ => {
                            p.0 += dx;
                            p.1 += dy;
                        }
                    }
                }
            }
        }
    }
}
