use crate::service::native_overlay::snapping::{collect_snap_lines, snap_coordinate};
use crate::service::native_overlay::state::{self, OverlayState};
use windows::Win32::Foundation::RECT;

pub struct InteractionHandler;

impl InteractionHandler {
    pub fn handle_mouse_down(state: &mut OverlayState, x: i32, y: i32) {
        // Detect zone first with raw coordinates
        let detected_zone = if let Some(sel) = state.selection {
            state::HitZone::detect(&sel, x, y)
        } else {
            state::HitZone::None
        };

        let mut start_x = x;
        let mut start_y = y;

        // Check Ctrl Key
        let is_ctrl = unsafe {
            (windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(
                windows::Win32::UI::Input::KeyboardAndMouse::VK_CONTROL.0 as i32,
            ) as u16
                & 0x8000)
                != 0
        };

        // If Starting Selection (None zone), Snap the Start Point!
        if matches!(detected_zone, state::HitZone::None) && !is_ctrl {
            let (sx, sy) = collect_snap_lines(state);
            start_x = snap_coordinate(start_x, &sx, 25);
            start_y = snap_coordinate(start_y, &sy, 25);
        }

        state.start_x = start_x;
        state.start_y = start_y;
        state.drag_start_selection = state.selection;
        state.hover_zone = detected_zone;

        match state.hover_zone {
            state::HitZone::None => {
                state.interaction_mode = state::InteractionMode::Selecting;
                state.selection = None; // Start new
            }
            state::HitZone::Body => {
                // Check if we are using a tool
                if state.current_tool != state::DrawingTool::None {
                    state.interaction_mode = state::InteractionMode::Drawing;
                    // Create new drawing object
                    state.current_drawing = Some(state::DrawingObject {
                        tool: state.current_tool,
                        points: vec![(x, y)],
                        text: None,
                        color: state.current_color,
                        stroke_width: state.current_stroke,
                        is_filled: false,
                        is_dashed: false,
                    });
                } else {
                    state.interaction_mode = state::InteractionMode::Moving;
                }
            }
            z => {
                state.interaction_mode = state::InteractionMode::Resizing(z);
            }
        }
    }

    pub fn handle_mouse_move(state: &mut OverlayState, x: i32, y: i32) {
        state.mouse_x = x;
        state.mouse_y = y;

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
                state.selection = Some(RECT {
                    left,
                    top,
                    right,
                    bottom,
                });
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
                        // Snap Left Edge
                        let sl = snap_coordinate(l, &snap_x, threshold);
                        // Snap Right Edge
                        let sr = snap_coordinate(l + w, &snap_x, threshold) - w;

                        if (sl - l).abs() < (sr - l).abs() && (sl - l).abs() < threshold {
                            l = sl;
                        } else if (sr - l).abs() < threshold {
                            l = sr;
                        }

                        // Snap Top
                        let st = snap_coordinate(t, &snap_y, threshold);
                        // Snap Bottom
                        let sb = snap_coordinate(t + h, &snap_y, threshold) - h;

                        if (st - t).abs() < (sb - t).abs() && (st - t).abs() < threshold {
                            t = st;
                        } else if (sb - t).abs() < threshold {
                            t = sb;
                        }
                    }

                    // Bounds
                    if l < 0 {
                        l = 0;
                    }
                    if t < 0 {
                        t = 0;
                    }
                    if l + w > state.width {
                        l = state.width - w;
                    }
                    if t + h > state.height {
                        t = state.height - h;
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

                    // Apply Delta
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

                    // Apply Snap
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

                    // Min Size
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
                    // For Brush, append point
                    // For Rect/Arrow/Line/Ellipse, update 2nd point (end point)
                    match drawing.tool {
                        state::DrawingTool::Brush | state::DrawingTool::Mosaic => {
                            // Distance check?
                            let last = drawing.points.last().unwrap_or(&(0, 0));
                            if (x - last.0).abs() > 2 || (y - last.1).abs() > 2 {
                                drawing.points.push((x, y));
                            }
                        }
                        _ => {
                            // Replace last point or ensure we have 2 points
                            if drawing.points.len() == 1 {
                                drawing.points.push((x, y));
                            } else if drawing.points.len() == 2 {
                                drawing.points[1] = (x, y);
                            }
                        }
                    }
                }
            }
            state::InteractionMode::None => {
                if let Some(sel) = state.selection {
                    state.hover_zone = state::HitZone::detect(&sel, x, y);
                } else {
                    state.hover_zone = state::HitZone::None;
                }
            }
        }
    }

    pub fn handle_mouse_up(state: &mut OverlayState) {
        if let state::InteractionMode::Drawing = state.interaction_mode {
            if let Some(drawing) = state.current_drawing.take() {
                // If it's a valid drawing (not just a click), save it
                if drawing.points.len() >= 2 {
                    state.objects.push(drawing);
                } else if matches!(drawing.tool, state::DrawingTool::Number) {
                    // For Number, Click is enough
                    state.objects.push(drawing);
                }

                // If the tool is NOT continuous (like Rect/Arrow), should we reset tool?
                // Usually user wants to draw multiple arrows. Keep tool selected.
                // But for Text?
            }
        }

        state.interaction_mode = state::InteractionMode::None;
    }
}
