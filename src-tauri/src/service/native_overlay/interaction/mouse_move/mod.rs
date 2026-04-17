mod drawing_handler;
mod handle_selection;
mod hover_handler;
mod move_selection;
mod resize_selection;
mod transform_object;

use crate::service::native_overlay::render::toolbar::Toolbar;
use crate::service::native_overlay::snapping::{update_snap_lines, snap_coordinate};
use crate::service::native_overlay::state::{InteractionMode, OverlayState};

pub fn handle_mouse_move(state: &mut OverlayState, toolbar: &mut Toolbar, x: i32, y: i32) {
    // 0. Handle Property Drag (Opacity/Glow)
    if toolbar.is_dragging_opacity || toolbar.is_dragging_glow {
        if let Some(change) = toolbar.handle_property_move(x, y, state.enable_advanced_effects, state.capture_engine) {
            match change {
                crate::service::native_overlay::state::PropertyChange::Opacity(new_val) => {
                    if (new_val - state.current_opacity).abs() > 0.005 {
                        state.apply_property_change(change);
                    }
                }
                crate::service::native_overlay::state::PropertyChange::Glow(new_val) => {
                    if (new_val - state.current_glow).abs() > 0.005 {
                        state.apply_property_change(change);
                    }
                }
                _ => state.apply_property_change(change),
            }
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

    // Prepare Snap Targets (Zero Allocation Cache)
    if !is_ctrl && state.snapping_dirty {
        update_snap_lines(state);
    }
    
    let threshold = 25;

    match state.interaction_mode {
        InteractionMode::TransformingObject(zone) => {
            let dx = sanitize(x - state.start_x);
            let dy = sanitize(y - state.start_y);
            transform_object::handle(state, zone, x, y, dx, dy);
        }
        InteractionMode::Selecting => {
            let mut sx = x;
            let mut sy = y;
            if !is_ctrl {
                sx = snap_coordinate(sx, &state.gdi.snap_x_cache, threshold);
                sy = snap_coordinate(sy, &state.gdi.snap_y_cache, threshold);
            }
            handle_selection::handle(state, sx, sy);
        }
        InteractionMode::Moving => {
            move_selection::handle(state, x, y, is_ctrl, threshold);
        }
        InteractionMode::Resizing(zone) => {
            resize_selection::handle(state, zone, x, y, is_ctrl, threshold);
        }
        InteractionMode::Drawing => {
            drawing_handler::handle(state, x, y);
        }
        InteractionMode::None => {
            hover_handler::handle(state, toolbar, x, y);
        }
    }
}
