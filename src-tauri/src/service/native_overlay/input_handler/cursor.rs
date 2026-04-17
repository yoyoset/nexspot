use crate::service::native_overlay::render::toolbar;
use crate::service::native_overlay::state;
use std::sync::{Arc, RwLock};
use windows::core::PCWSTR;

pub fn handle_cursor(
    state_arc: &Arc<RwLock<state::OverlayState>>,
    toolbar: &toolbar::Toolbar,
    _x: i32,
    _y: i32,
) -> bool {
    // This is a wrapper to match the InputHandler signature
    if let Ok(cursor) = get_current_cursor(state_arc, toolbar) {
        unsafe {
            windows::Win32::UI::WindowsAndMessaging::SetCursor(
                Some(windows::Win32::UI::WindowsAndMessaging::LoadCursorW(None, cursor).unwrap_or_default()),
            );
        }
        return true;
    }
    false
}

pub fn get_current_cursor(
    state_arc: &Arc<RwLock<state::OverlayState>>,
    toolbar: &toolbar::Toolbar,
) -> anyhow::Result<PCWSTR> {
    let state = match state_arc.read() {
        Ok(s) => s,
        Err(_) => return Err(anyhow::anyhow!("State lock poisoned")),
    };

    use windows::Win32::UI::WindowsAndMessaging::{
        IDC_ARROW, IDC_CROSS, IDC_SIZEALL, IDC_SIZENS, IDC_SIZEWE, IDC_SIZENWSE, IDC_SIZENESW,
    };

    // 1. Toolbar Precedence
    if toolbar.visible {
        if state.mouse_x >= toolbar.rect.left
            && state.mouse_x < toolbar.rect.right
            && state.mouse_y >= toolbar.rect.top
            && state.mouse_y < toolbar.rect.bottom
        {
            return Ok(IDC_ARROW);
        }
        if toolbar.property_bar_visible
            && state.mouse_x >= toolbar.property_bar_rect.left
            && state.mouse_x < toolbar.property_bar_rect.right
            && state.mouse_y >= toolbar.property_bar_rect.top
            && state.mouse_y < toolbar.property_bar_rect.bottom
        {
            return Ok(IDC_ARROW);
        }
    }

    // 2. Interaction Mode / Tool Logic
    Ok(match state.interaction_mode {
        state::InteractionMode::Selecting | state::InteractionMode::Drawing => {
            if state.interaction_mode == state::InteractionMode::Drawing
                && matches!(
                    state.current_tool,
                    state::DrawingTool::Brush | state::DrawingTool::Mosaic
                )
            {
                IDC_ARROW
            } else {
                IDC_CROSS
            }
        }
        state::InteractionMode::Moving => IDC_SIZEALL,
        state::InteractionMode::Resizing(z) | state::InteractionMode::TransformingObject(z) => {
            match z {
                state::HitZone::Top | state::HitZone::Bottom => IDC_SIZENS,
                state::HitZone::Left | state::HitZone::Right => IDC_SIZEWE,
                state::HitZone::TopLeft | state::HitZone::BottomRight => IDC_SIZENWSE,
                state::HitZone::TopRight | state::HitZone::BottomLeft => IDC_SIZENESW,
                state::HitZone::Body | state::HitZone::Stroke => IDC_SIZEALL,
                _ => IDC_ARROW,
            }
        }
        state::InteractionMode::None => {
            let mut object_cursor = None;

            for (_idx, obj) in state.objects.iter().enumerate().rev() {
                let zone = obj.hit_test(state.mouse_x, state.mouse_y);
                if !matches!(zone, state::HitZone::None) {
                    object_cursor = Some(match zone {
                        state::HitZone::Body | state::HitZone::Stroke => IDC_SIZEALL,
                        state::HitZone::Top | state::HitZone::Bottom => IDC_SIZENS,
                        state::HitZone::Left | state::HitZone::Right => IDC_SIZEWE,
                        state::HitZone::TopLeft | state::HitZone::BottomRight => IDC_SIZENWSE,
                        state::HitZone::TopRight | state::HitZone::BottomLeft => IDC_SIZENESW,
                        _ => IDC_ARROW,
                    });
                    break;
                }
            }

            if let Some(cursor) = object_cursor {
                cursor
            } else if state.current_tool != state::DrawingTool::None
                && matches!(state.hover_zone, state::HitZone::Body)
            {
                if matches!(
                    state.current_tool,
                    state::DrawingTool::Brush | state::DrawingTool::Mosaic
                ) {
                    IDC_ARROW
                } else {
                    IDC_CROSS
                }
            } else {
                match state.hover_zone {
                    state::HitZone::Stroke => IDC_SIZEALL,
                    state::HitZone::Top | state::HitZone::Bottom => IDC_SIZENS,
                    state::HitZone::Left | state::HitZone::Right => IDC_SIZEWE,
                    state::HitZone::TopLeft | state::HitZone::BottomRight => IDC_SIZENWSE,
                    state::HitZone::TopRight | state::HitZone::BottomLeft => IDC_SIZENESW,
                    state::HitZone::Body => IDC_ARROW,
                    _ => IDC_ARROW,
                }
            }
        }
    })
}
