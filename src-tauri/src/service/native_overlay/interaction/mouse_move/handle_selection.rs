use crate::service::native_overlay::state::OverlayState;
use windows::Win32::Foundation::RECT;

pub fn handle(state: &mut OverlayState, sx: i32, sy: i32) {
    // --- DOUBLE-CLICK FIX: Minimum dragging threshold ---
    // If we already have a selection, don't start a NEW one until we've moved significantly.
    if state.drag_start_selection.is_some() {
        let dx = (sx - state.start_x).abs();
        let dy = (sy - state.start_y).abs();
        if dx < 5 && dy < 5 && !state.is_selection_active {
            return;
        }
    }

    let left = state.start_x.min(sx);
    let top = state.start_y.min(sy);
    let right = state.start_x.max(sx);
    let bottom = state.start_y.max(sy);

    let result_rect = RECT {
        left,
        top,
        right,
        bottom,
    };

    state.selection = Some(result_rect);
    state.is_selection_active = true;
}
