use crate::service::native_overlay::snapping::snap_coordinate;
use crate::service::native_overlay::state::{HitZone, OverlayState};
use windows::Win32::Foundation::RECT;

pub fn handle(
    state: &mut OverlayState,
    zone: HitZone,
    x: i32,
    y: i32,
    is_ctrl: bool,
    threshold: i32,
) {
    if let Some(start_sel) = state.drag_start_selection {
        let dx = x - state.start_x;
        let dy = y - state.start_y;
        let mut r = start_sel;

        match zone {
            HitZone::Left => r.left += dx,
            HitZone::Right => r.right += dx,
            HitZone::Top => r.top += dy,
            HitZone::Bottom => r.bottom += dy,
            HitZone::TopLeft => {
                r.left += dx;
                r.top += dy;
            }
            HitZone::TopRight => {
                r.right += dx;
                r.top += dy;
            }
            HitZone::BottomLeft => {
                r.left += dx;
                r.bottom += dy;
            }
            HitZone::BottomRight => {
                r.right += dx;
                r.bottom += dy;
            }
            _ => {}
        }

        if !is_ctrl {
            // Local borrow allows access to cache while we have &mut state for writing the result later.
            // But we must borrow ONLY what we need.
            let snap_x = &state.gdi.snap_x_cache;
            let snap_y = &state.gdi.snap_y_cache;

            match zone {
                HitZone::Left | HitZone::TopLeft | HitZone::BottomLeft => {
                    r.left = snap_coordinate(r.left, snap_x, threshold);
                }
                _ => {}
            }
            match zone {
                HitZone::Right | HitZone::TopRight | HitZone::BottomRight => {
                    r.right = snap_coordinate(r.right, snap_x, threshold);
                }
                _ => {}
            }
            match zone {
                HitZone::Top | HitZone::TopLeft | HitZone::TopRight => {
                    r.top = snap_coordinate(r.top, snap_y, threshold);
                }
                _ => {}
            }
            match zone {
                HitZone::Bottom | HitZone::BottomLeft | HitZone::BottomRight => {
                    r.bottom = snap_coordinate(r.bottom, snap_y, threshold);
                }
                _ => {}
            }
        }

        let bounds = state.restrict_to_monitor.unwrap_or(RECT {
            left: state.capture_x,
            top: state.capture_y,
            right: state.capture_x + state.width,
            bottom: state.capture_y + state.height,
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
