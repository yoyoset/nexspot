use crate::service::native_overlay::snapping::{update_snap_lines, snap_coordinate};
use crate::service::native_overlay::state::{self, OverlayState};

pub fn handle_mouse_down(
    state: &mut OverlayState,
    toolbar: &mut crate::service::native_overlay::render::toolbar::Toolbar,
    x: i32,
    y: i32,
) {
    // 0. Check Property Bar Interaction (Drag Start)
    if toolbar.handle_property_down(x, y, state.enable_advanced_effects, state.capture_engine) {
        return;
    }
    let can_resize = match state.capture_mode {
        state::CaptureMode::Standard => true,
        state::CaptureMode::Snapshot { allow_resize } => allow_resize,
        state::CaptureMode::FixedWindow => false,
    };

    let mut detected_zone = if let Some(sel) = state.selection {
        state::HitZone::detect(&sel, x, y)
    } else {
        state::HitZone::None
    };

    if !can_resize
        && !matches!(
            detected_zone,
            state::HitZone::None | state::HitZone::Body | state::HitZone::Stroke
        )
    {
        // Treat resize handles as Body (Move) if resizing is disabled
        detected_zone = state::HitZone::Body;
    }

    // 1. Hit Test for Objects
    let mut object_hit = None;
    if let Some(idx) = state.selected_object_index {
        if let Some(obj) = state.objects.get(idx) {
            let zone = obj.hit_test(x, y);
            if !matches!(zone, state::HitZone::None) {
                object_hit = Some((idx, zone));
            }
        }
    }

    if object_hit.is_none() {
        for (idx, obj) in state.objects.iter().enumerate().rev() {
            let zone = obj.hit_test(x, y);
            if !matches!(zone, state::HitZone::None) {
                object_hit = Some((idx, zone));
                break;
            }
        }
    }

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
    if matches!(detected_zone, state::HitZone::None) && !is_ctrl && object_hit.is_none() {
        if state.snapping_dirty {
            update_snap_lines(state);
        }
        start_x = snap_coordinate(start_x, &state.gdi.snap_x_cache, 25);
        start_y = snap_coordinate(start_y, &state.gdi.snap_y_cache, 25);
    }

    state.start_x = start_x;
    state.start_y = start_y;
    state.drag_start_selection = state.selection;
    state.hover_zone = detected_zone;

    // If selection exists and we hit nothing related to it, deactivate
    if let Some(_sel) = state.selection {
        if matches!(detected_zone, state::HitZone::None) && object_hit.is_none() {
            state.is_selection_active = false;
        }
    }

    // 2. Determine Interaction based on priority

    // Priority 1: Hitting a drawing object (WeChat Style: Select even if tool is active)
    if let Some((idx, zone)) = object_hit {
        let already_selected = state.selected_object_index == Some(idx);
        state.selected_object_index = Some(idx);

        if already_selected {
            // Already selected: allow move (body) or resize (handles)
            state.interaction_mode = state::InteractionMode::TransformingObject(zone);
        } else {
            // First click: Just select it
            state.interaction_mode =
                state::InteractionMode::TransformingObject(state::HitZone::Body);
        }
        state.selection_pointer = Some((x, y));
        return;
    }

    state.selection_pointer = None;

    if state.current_tool != state::DrawingTool::None {
        // --- SCENARIO A: Drawing Tool Active ---

        // Priority 1: Global Selection Handles (Resizing)
        // We prioritize this so the user can still adjust the crop area while tools are active.
        if !matches!(detected_zone, state::HitZone::None)
            && !matches!(detected_zone, state::HitZone::Body)
        {
            state.is_selection_active = true;
            state.interaction_mode = state::InteractionMode::Resizing(detected_zone);
            return;
        }

        // Priority 2: Always start a new drawing (We already checked object hits above)
        state.interaction_mode = state::InteractionMode::Drawing;
        state.selected_object_index = None; // Deselect others when starting a new draw

        let mut text = None;
        if state.current_tool == state::DrawingTool::Number {
            let count = state
                .objects
                .iter()
                .filter(|o| matches!(o.tool, state::DrawingTool::Number))
                .count();
            text = Some((count + 1).to_string());
        }

        state.current_drawing = Some(state::DrawingObject {
            tool: state.current_tool,
            points: vec![(x, y)],
            text,
            color: state.current_color,
            stroke_width: state.current_stroke,
            font_size: state.current_font_size,
            is_filled: state.current_is_filled,
            is_dashed: false,
            is_editing_text: state.current_tool == state::DrawingTool::Text,
            font_family: state.font_family.clone(),
            head_width: None,
            opacity: state.current_opacity,
            has_shadow: state.current_shadow,
            glow: state.current_glow,
            style: state.current_style,
            mosaic_blocks: std::collections::HashMap::new(),
            mosaic_pending_points: std::collections::VecDeque::new(),
            mosaic_last_pos: None,
        });
    } else {
        // --- SCENARIO B: No Tool Active (Pointer/Selection Mode) ---

        // Priority 1: Global area selection (Object hit already handled above)
        match state.hover_zone {
            state::HitZone::None => {
                state.interaction_mode = state::InteractionMode::Selecting;
                state.selection = None;
                state.selected_object_index = None;
            }
            state::HitZone::Body => {
                // Interior: If NO tool is active, allow DRAGGING the selection.
                // If a tool IS active, we treat it as None to protect double-click/jitter,
                // but effectively we allow the drawing mode to take over if SCENARIO A is hit.
                if state.current_tool == state::DrawingTool::None {
                    state.selected_object_index = None;
                    state.is_selection_active = true;
                    state.interaction_mode = state::InteractionMode::Moving;
                } else {
                    state.selected_object_index = None;
                    state.interaction_mode = state::InteractionMode::None;
                }
            }
            state::HitZone::Stroke => {
                // Border: Move the selection
                state.selected_object_index = None;
                state.is_selection_active = true;
                state.interaction_mode = state::InteractionMode::Moving;
            }
            z => {
                state.is_selection_active = true;
                state.interaction_mode = state::InteractionMode::Resizing(z);
            }
        }
    }
}
