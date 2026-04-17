use crate::service::native_overlay::render::toolbar::Toolbar;
use crate::service::native_overlay::state::{HitZone, OverlayState};

pub fn handle(state: &mut OverlayState, toolbar: &Toolbar, x: i32, y: i32) {
    let mut found_zone = HitZone::None;
    if let Some(idx) = state.selected_object_index {
        if let Some(obj) = state.objects.get(idx) {
            found_zone = obj.hit_test(x, y);
        }
    }
    if matches!(found_zone, HitZone::None) {
        for obj in state.objects.iter().rev() {
            let zone = obj.hit_test(x, y);
            if !matches!(zone, HitZone::None) {
                found_zone = zone;
                break;
            }
        }
    }
    if !matches!(found_zone, HitZone::None) {
        state.hover_zone = found_zone;
    } else if let Some(sel) = state.selection {
        let mut zone = HitZone::detect(&sel, x, y);

        // Filter zone based on capture mode
        let can_resize = match state.capture_mode {
            crate::service::native_overlay::state::CaptureMode::Standard => true,
            crate::service::native_overlay::state::CaptureMode::Snapshot { allow_resize } => {
                allow_resize
            }
            crate::service::native_overlay::state::CaptureMode::FixedWindow => false,
        };

        if !can_resize && !matches!(zone, HitZone::None | HitZone::Body | HitZone::Stroke) {
            // Treat resize handles as Body (Move) if resizing is disabled
            zone = HitZone::Body;
        }

        // If Brush/Mosaic is active, we don't want the default selection hover (which shows arrow/resize cursors)
        // because we want to show the brush circle preview instead.
        if (state.current_tool == crate::service::native_overlay::state::DrawingTool::Brush
            || state.current_tool == crate::service::native_overlay::state::DrawingTool::Mosaic)
            && matches!(zone, HitZone::Body)
        {
            state.hover_zone = HitZone::None;
        } else {
            state.hover_zone = zone;
        }
    } else {
        state.hover_zone = HitZone::None;
    }

    if toolbar.hit_test(x, y) {
        state.hover_zone = HitZone::None;
    }
}
