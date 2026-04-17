use crate::service::native_overlay::render::toolbar;
use crate::service::native_overlay::state;
use crate::service::native_overlay::interaction::InteractionHandler;
use std::sync::{Arc, RwLock};

pub fn handle_mouse_down(
    state_arc: &Arc<RwLock<state::OverlayState>>,
    toolbar: &mut toolbar::Toolbar,
    x: i32,
    y: i32,
) -> bool {
    if toolbar.handle_mouse_down(x, y) {
        return true;
    }

    let (enable_advanced, engine) = {
        let state = state_arc.read().unwrap();
        (state.enable_advanced_effects, state.capture_engine)
    };
    if toolbar.handle_property_down(x, y, enable_advanced, engine) {
        return true;
    }

    if toolbar.property_bar_visible
        && x >= toolbar.property_bar_rect.left
        && x < toolbar.property_bar_rect.right
        && y >= toolbar.property_bar_rect.top
        && y < toolbar.property_bar_rect.bottom
    {
        return true;
    }

    let mut state = match state_arc.write() {
        Ok(s) => s,
        Err(_) => return false,
    };
    
    for obj in &mut state.objects {
        obj.is_editing_text = false;
    }
    
    InteractionHandler::handle_mouse_down(&mut state, toolbar, x, y);
    true
}

pub fn handle_mouse_move(
    state_arc: &Arc<RwLock<state::OverlayState>>,
    toolbar: &mut toolbar::Toolbar,
    x: i32,
    y: i32,
) -> bool {
    let mut needs_render = false;

    if toolbar.handle_mouse_move(x, y) {
        needs_render = true;
    }

    // Property dragging (Slider)
    let (enable_advanced, engine) = {
        let state = state_arc.read().unwrap();
        (
            state.enable_advanced_effects,
            state.capture_engine,
        )
    };
    if let Some(change) = toolbar.handle_property_move(x, y, enable_advanced, engine) {
        if let Ok(mut state) = state_arc.write() {
            state.apply_property_change(change);
            needs_render = true;
        }
    }

    {
        if let Ok(mut state) = state_arc.write() {
            InteractionHandler::handle_mouse_move(&mut state, toolbar, x, y);
        }
    }

    {
        if let Ok(mut state) = state_arc.write() {
            // OPTIMIZATION: Check if we are still within the current monitor_rect
            // only enumerate if we cross boundaries.
            let in_current = x >= state.monitor_rect.left
                && x < state.monitor_rect.right
                && y >= state.monitor_rect.top
                && y < state.monitor_rect.bottom;

            if !in_current || state.monitors.is_empty() {
                if let Ok(monitors) = crate::service::win32::monitor::enumerate_monitors() {
                    if let Some(m) = monitors.iter().find(|m| {
                        x >= m.rect.left && x < m.rect.right && y >= m.rect.top && y < m.rect.bottom
                    }) {
                        state.monitor_rect = m.rect;
                        state.monitor_id = m.hmonitor.to_string();
                    }
                    state.monitors = monitors;
                }
            }

            if let Some(sel) = state.selection {
                toolbar.update_layout(
                    sel,
                    state.monitor_rect.left,
                    state.monitor_rect.top,
                    state.monitor_rect.right - state.monitor_rect.left,
                    state.monitor_rect.bottom - state.monitor_rect.top,
                    state.enable_advanced_effects,
                    state.capture_engine,
                );
            } else {
                toolbar.hide();
            }
        }
    }

    let interaction_needs_render = {
        let state = state_arc.read().unwrap();
        !matches!(state.interaction_mode, state::InteractionMode::None)
    };

    needs_render || interaction_needs_render
}

pub fn handle_mouse_up(
    state_arc: &Arc<RwLock<state::OverlayState>>,
    toolbar: &mut toolbar::Toolbar,
    x: i32,
    y: i32,
) -> (Option<toolbar::ToolType>, bool) {
    if let Some(cmd) = toolbar.handle_mouse_up(x, y) {
        return (Some(cmd), true);
    }

    toolbar.reset_dragging();

    let (enable_advanced, current_is_filled, engine) = {
        let state = state_arc.read().unwrap();
        (state.enable_advanced_effects, state.current_is_filled, state.capture_engine)
    };

    if let Some(change) = toolbar.handle_property_click(x, y, enable_advanced, current_is_filled, engine) {
        if let Ok(mut state) = state_arc.write() {
            state.apply_property_change(change);
            return (None, true);
        }
    }

    if let Ok(mut state) = state_arc.write() {
        InteractionHandler::handle_mouse_up(&mut state, toolbar);

        if let Some(sel) = state.selection {
            if matches!(state.interaction_mode, state::InteractionMode::None) {
                // Ensure context is updated for mouse up as well
                if let Ok(monitors) = crate::service::win32::monitor::enumerate_monitors() {
                    if let Some(m) = monitors.iter().find(|m| {
                        x >= m.rect.left && x < m.rect.right && y >= m.rect.top && y < m.rect.bottom
                    }) {
                        state.monitor_rect = m.rect;
                        state.monitor_id = m.hmonitor.to_string();
                    }
                }

                toolbar.update_layout(
                    sel,
                    state.monitor_rect.left,
                    state.monitor_rect.top,
                    state.monitor_rect.right - state.monitor_rect.left,
                    state.monitor_rect.bottom - state.monitor_rect.top,
                    state.enable_advanced_effects,
                    state.capture_engine,
                );
            } else {
                toolbar.hide();
            }
        } else {
            toolbar.hide();
        }
    }

    (None, true)
}

pub fn handle_double_click(
    state_arc: &Arc<RwLock<state::OverlayState>>,
    x: i32,
    y: i32,
) -> bool {
    let mut state = match state_arc.write() {
        Ok(s) => s,
        Err(_) => return false,
    };

    let mut hit_idx = None;
    for (idx, obj) in state.objects.iter().enumerate().rev() {
        if !matches!(obj.hit_test(x, y), state::HitZone::None) {
            hit_idx = Some(idx);
            break;
        }
    }

    if let Some(idx) = hit_idx {
        if state.objects[idx].tool == state::DrawingTool::Text {
            state.objects[idx].is_editing_text = true;
            state.selected_object_index = Some(idx);
            return false;
        } else {
            return true;
        }
    }

    if let Some(sel) = state.selection {
        if x >= sel.left && x <= sel.right && y >= sel.top && y <= sel.bottom {
            return true;
        }
    }
    false
}
