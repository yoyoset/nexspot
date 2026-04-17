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

pub fn update_snap_lines(state: &mut OverlayState) {
    // Clear existing cache without losing capacity
    state.gdi.snap_x_cache.clear();
    state.gdi.snap_y_cache.clear();

    if !state.window_rects.is_empty() {
        let ox = state.capture_x;
        let oy = state.capture_y;
        for r in &state.window_rects {
            state.gdi.snap_x_cache.push(r.left - ox);
            state.gdi.snap_x_cache.push(r.right - ox);
            state.gdi.snap_y_cache.push(r.top - oy);
            state.gdi.snap_y_cache.push(r.bottom - oy);
        }
        
        // Boundaries
        state.gdi.snap_x_cache.push(0);
        state.gdi.snap_x_cache.push(state.width);
        state.gdi.snap_y_cache.push(0);
        state.gdi.snap_y_cache.push(state.height);
    }
    
    state.snapping_dirty = false;
}
