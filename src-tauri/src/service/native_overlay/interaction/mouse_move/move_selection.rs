use crate::service::native_overlay::snapping::snap_coordinate;
use crate::service::native_overlay::state::OverlayState;
use windows::Win32::Foundation::RECT;

pub fn handle(
    state: &mut OverlayState,
    x: i32,
    y: i32,
    is_ctrl: bool,
    threshold: i32,
) {
    if let Some(start_sel) = state.drag_start_selection {
        let dx = x - state.start_x;
        let dy = y - state.start_y;
        let mut l = start_sel.left + dx;
        let mut t = start_sel.top + dy;
        let w = start_sel.right - start_sel.left;
        let h = start_sel.bottom - start_sel.top;

        if !is_ctrl {
            let snap_x = &state.gdi.snap_x_cache;
            let snap_y = &state.gdi.snap_y_cache;
            
            let sl = snap_coordinate(l, snap_x, threshold);
            let sr = snap_coordinate(l + w, snap_x, threshold) - w;
            if (sl - l).abs() < (sr - l).abs() && (sl - l).abs() < threshold {
                l = sl;
            } else if (sr - l).abs() < threshold {
                l = sr;
            }
            let st = snap_coordinate(t, snap_y, threshold);
            let sb = snap_coordinate(t + h, snap_y, threshold) - h;
            if (st - t).abs() < (sb - t).abs() && (st - t).abs() < threshold {
                t = st;
            } else if (sb - t).abs() < threshold {
                t = sb;
            }
        }

        let bounds = state.restrict_to_monitor.unwrap_or(RECT {
            left: state.capture_x,
            top: state.capture_y,
            right: state.capture_x + state.width,
            bottom: state.capture_y + state.height,
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
