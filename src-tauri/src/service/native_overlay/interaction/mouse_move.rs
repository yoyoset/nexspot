use crate::service::native_overlay::snapping::{collect_snap_lines, snap_coordinate};
use crate::service::native_overlay::state::{self, OverlayState};
use windows::Win32::Foundation::RECT;

pub fn handle_mouse_move(
    state: &mut OverlayState,
    toolbar: &mut crate::service::native_overlay::render::toolbar::Toolbar,
    x: i32,
    y: i32,
) {
    // 0. Handle Property Drag (Opacity)
    if toolbar.is_dragging_opacity {
        if let Some(change) = toolbar.handle_property_move(x, y, state.enable_advanced_effects) {
            state.apply_property_change(change);
        }
        return;
    }
    state.mouse_x = x;
    state.mouse_y = y;

    // Helper to prevent NaN/Inf in coordinates
    let sanitize = |val: i32| -> i32 {
        if val == i32::MIN || val == i32::MAX {
            val
        } else {
            val.clamp(-100000, 100000)
        }
    };

    // Check Ctrl Key (Disable Snap)
    let is_ctrl = unsafe {
        (windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(
            windows::Win32::UI::Input::KeyboardAndMouse::VK_CONTROL.0 as i32,
        ) as u16
            & 0x8000)
            != 0
    };

    // Prepare Snap Targets
    let (snap_x, snap_y) = if !is_ctrl {
        collect_snap_lines(state)
    } else {
        (Vec::new(), Vec::new())
    };
    let threshold = 25;

    match state.interaction_mode {
        state::InteractionMode::TransformingObject(zone) => {
            if let Some(idx) = state.selected_object_index {
                let dx = sanitize(x - state.start_x);
                let dy = sanitize(y - state.start_y);
                state.start_x = x;
                state.start_y = y;

                if let Some(obj) = state.objects.get_mut(idx) {
                    let is_arrow = obj.tool == state::DrawingTool::Arrow;

                    if is_arrow && obj.points.len() == 2 {
                        match zone {
                            state::HitZone::Body | state::HitZone::Stroke => {
                                for p in &mut obj.points {
                                    p.0 += dx;
                                    p.1 += dy;
                                }
                            }
                            state::HitZone::Tail => {
                                obj.points[0].0 += dx;
                                obj.points[0].1 += dy;
                            }
                            state::HitZone::Tip => {
                                obj.points[1].0 += dx;
                                obj.points[1].1 += dy;
                            }
                            state::HitZone::WingLeft | state::HitZone::WingRight => {
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

                                    // Project mouse onto the normal vector (px, py) relative to Tip
                                    let mdx = x as f32 - p2.0 as f32;
                                    let mdy = y as f32 - p2.1 as f32;
                                    let dist_h = (mdx * px + mdy * py).abs();

                                    // head_width = distance from axis * 2
                                    obj.head_width = Some(dist_h * 2.0);
                                }
                            }
                            _ => {}
                        }
                    } else {
                        let bounds = obj.get_bounds();
                        let is_2point = obj.points.len() == 2;

                        for p in &mut obj.points {
                            match zone {
                                state::HitZone::Body | state::HitZone::Stroke => {
                                    p.0 += dx;
                                    p.1 += dy;
                                }
                                _ if is_2point => {
                                    let is_top = p.1 == bounds.top;
                                    let is_bottom = p.1 == bounds.bottom;
                                    let is_left = p.0 == bounds.left;
                                    let is_right = p.0 == bounds.right;

                                    match zone {
                                        state::HitZone::TopLeft if is_left && is_top => {
                                            p.0 += dx;
                                            p.1 += dy;
                                        }
                                        state::HitZone::TopRight if is_right && is_top => {
                                            p.0 += dx;
                                            p.1 += dy;
                                        }
                                        state::HitZone::BottomLeft if is_left && is_bottom => {
                                            p.0 += dx;
                                            p.1 += dy;
                                        }
                                        state::HitZone::BottomRight if is_right && is_bottom => {
                                            p.0 += dx;
                                            p.1 += dy;
                                        }
                                        state::HitZone::Top if is_top => {
                                            p.1 += dy;
                                        }
                                        state::HitZone::Bottom if is_bottom => {
                                            p.1 += dy;
                                        }
                                        state::HitZone::Left if is_left => {
                                            p.0 += dx;
                                        }
                                        state::HitZone::Right if is_right => {
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
        state::InteractionMode::Selecting => {
            let mut sx = x;
            let mut sy = y;
            if !is_ctrl {
                sx = snap_coordinate(sx, &snap_x, threshold);
                sy = snap_coordinate(sy, &snap_y, threshold);
            }

            let left = state.start_x.min(sx);
            let top = state.start_y.min(sy);
            let right = state.start_x.max(sx);
            let bottom = state.start_y.max(sy);

            let mut result_rect = RECT {
                left,
                top,
                right,
                bottom,
            };

            let bounds = state.restrict_to_monitor.unwrap_or(RECT {
                left: state.x,
                top: state.y,
                right: state.x + state.width,
                bottom: state.y + state.height,
            });

            result_rect.left = result_rect.left.clamp(bounds.left, bounds.right);
            result_rect.right = result_rect.right.clamp(bounds.left, bounds.right);
            result_rect.top = result_rect.top.clamp(bounds.top, bounds.bottom);
            result_rect.bottom = result_rect.bottom.clamp(bounds.top, bounds.bottom);

            state.selection = Some(result_rect);
            state.is_selection_active = true;
        }
        state::InteractionMode::Moving => {
            if let Some(start_sel) = state.drag_start_selection {
                let dx = x - state.start_x;
                let dy = y - state.start_y;
                let mut l = start_sel.left + dx;
                let mut t = start_sel.top + dy;
                let w = start_sel.right - start_sel.left;
                let h = start_sel.bottom - start_sel.top;

                if !is_ctrl {
                    let sl = snap_coordinate(l, &snap_x, threshold);
                    let sr = snap_coordinate(l + w, &snap_x, threshold) - w;
                    if (sl - l).abs() < (sr - l).abs() && (sl - l).abs() < threshold {
                        l = sl;
                    } else if (sr - l).abs() < threshold {
                        l = sr;
                    }
                    let st = snap_coordinate(t, &snap_y, threshold);
                    let sb = snap_coordinate(t + h, &snap_y, threshold) - h;
                    if (st - t).abs() < (sb - t).abs() && (st - t).abs() < threshold {
                        t = st;
                    } else if (sb - t).abs() < threshold {
                        t = sb;
                    }
                }

                let bounds = state.restrict_to_monitor.unwrap_or(RECT {
                    left: state.x,
                    top: state.y,
                    right: state.x + state.width,
                    bottom: state.y + state.height,
                });

                if l < bounds.left {
                    l = bounds.left;
                }
                if t < bounds.top {
                    t = bounds.top;
                }
                if l + w > bounds.right {
                    l = bounds.right - w;
                }
                if t + h > bounds.bottom {
                    t = bounds.bottom - h;
                }

                state.selection = Some(RECT {
                    left: l,
                    top: t,
                    right: l + w,
                    bottom: t + h,
                });
            }
        }
        state::InteractionMode::Resizing(zone) => {
            if let Some(start_sel) = state.drag_start_selection {
                let dx = x - state.start_x;
                let dy = y - state.start_y;
                let mut r = start_sel;

                match zone {
                    state::HitZone::Left => r.left += dx,
                    state::HitZone::Right => r.right += dx,
                    state::HitZone::Top => r.top += dy,
                    state::HitZone::Bottom => r.bottom += dy,
                    state::HitZone::TopLeft => {
                        r.left += dx;
                        r.top += dy;
                    }
                    state::HitZone::TopRight => {
                        r.right += dx;
                        r.top += dy;
                    }
                    state::HitZone::BottomLeft => {
                        r.left += dx;
                        r.bottom += dy;
                    }
                    state::HitZone::BottomRight => {
                        r.right += dx;
                        r.bottom += dy;
                    }
                    _ => {}
                }

                if !is_ctrl {
                    match zone {
                        state::HitZone::Left
                        | state::HitZone::TopLeft
                        | state::HitZone::BottomLeft => {
                            r.left = snap_coordinate(r.left, &snap_x, threshold);
                        }
                        _ => {}
                    }
                    match zone {
                        state::HitZone::Right
                        | state::HitZone::TopRight
                        | state::HitZone::BottomRight => {
                            r.right = snap_coordinate(r.right, &snap_x, threshold);
                        }
                        _ => {}
                    }
                    match zone {
                        state::HitZone::Top
                        | state::HitZone::TopLeft
                        | state::HitZone::TopRight => {
                            r.top = snap_coordinate(r.top, &snap_y, threshold);
                        }
                        _ => {}
                    }
                    match zone {
                        state::HitZone::Bottom
                        | state::HitZone::BottomLeft
                        | state::HitZone::BottomRight => {
                            r.bottom = snap_coordinate(r.bottom, &snap_y, threshold);
                        }
                        _ => {}
                    }
                }

                let bounds = state.restrict_to_monitor.unwrap_or(RECT {
                    left: state.x,
                    top: state.y,
                    right: state.x + state.width,
                    bottom: state.y + state.height,
                });

                r.left = r.left.clamp(bounds.left, bounds.right - 5);
                r.right = r.right.clamp(bounds.left + 5, bounds.right);
                r.top = r.top.clamp(bounds.top, bounds.bottom - 5);
                r.bottom = r.bottom.clamp(bounds.top + 5, bounds.bottom);

                if r.right < r.left + 5 {
                    r.right = r.left + 5;
                }
                if r.bottom < r.top + 5 {
                    r.bottom = r.top + 5;
                }
                state.selection = Some(r);
            }
        }
        state::InteractionMode::Drawing => {
            if let Some(drawing) = &mut state.current_drawing {
                let rect = state.selection.unwrap_or(RECT {
                    left: 0,
                    top: 0,
                    right: state.width,
                    bottom: state.height,
                });

                let cx = x.clamp(rect.left, rect.right);
                let cy = y.clamp(rect.top, rect.bottom);

                match drawing.tool {
                    state::DrawingTool::Brush | state::DrawingTool::Mosaic => {
                        let last = drawing.points.last().unwrap_or(&(0, 0));
                        if (cx - last.0).abs() > 2 || (cy - last.1).abs() > 2 {
                            drawing.points.push((cx, cy));
                        }
                    }
                    _ => {
                        if drawing.points.len() == 1 {
                            drawing.points.push((cx, cy));
                        } else if drawing.points.len() == 2 {
                            drawing.points[1] = (cx, cy);
                        }
                    }
                }
            }
        }
        state::InteractionMode::None => {
            let mut found_zone = state::HitZone::None;
            if let Some(idx) = state.selected_object_index {
                if let Some(obj) = state.objects.get(idx) {
                    found_zone = obj.hit_test(x, y);
                }
            }
            if matches!(found_zone, state::HitZone::None) {
                for obj in state.objects.iter().rev() {
                    let zone = obj.hit_test(x, y);
                    if !matches!(zone, state::HitZone::None) {
                        found_zone = zone;
                        break;
                    }
                }
            }
            if !matches!(found_zone, state::HitZone::None) {
                state.hover_zone = found_zone;
            } else if let Some(sel) = state.selection {
                state.hover_zone = state::HitZone::detect(&sel, x, y);
            } else {
                state.hover_zone = state::HitZone::None;
            }
        }
    }
}
