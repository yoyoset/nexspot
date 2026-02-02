use crate::service::native_overlay::state::OverlayState;

pub fn snap_coordinate(val: i32, targets: &[i32], threshold: i32) -> i32 {
    let mut best = val;
    let mut min_dist = threshold;
    for &target in targets {
        let dist = (val - target).abs();
        if dist < min_dist {
            min_dist = dist;
            best = target;
        }
    }
    best
}

pub fn collect_snap_lines(state: &OverlayState) -> (Vec<i32>, Vec<i32>) {
    let mut snap_x = Vec::new();
    let mut snap_y = Vec::new();
    if !state.window_rects.is_empty() {
        let ox = state.x;
        let oy = state.y;
        for r in &state.window_rects {
            snap_x.push(r.left - ox);
            snap_x.push(r.right - ox);
            snap_y.push(r.top - oy);
            snap_y.push(r.bottom - oy);
        }
        snap_x.push(0);
        snap_x.push(state.width);
        snap_y.push(0);
        snap_y.push(state.height);
    }
    (snap_x, snap_y)
}
