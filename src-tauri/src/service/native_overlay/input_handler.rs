use crate::service::native_overlay::interaction::InteractionHandler;
use crate::service::native_overlay::render::toolbar;
use crate::service::native_overlay::state;
use std::sync::{Arc, Mutex};

pub struct InputHandler;

impl InputHandler {
    pub fn handle_mouse_down(
        state_arc: &Arc<Mutex<state::OverlayState>>,
        toolbar: &mut toolbar::Toolbar,
        x: i32,
        y: i32,
    ) -> bool {
        // Toolbar Precedence
        if toolbar.handle_mouse_down(x, y) {
            return true;
        }

        // Property Bar Precedence (Prevent starting selection/drawing when clicking colors)
        if toolbar.property_bar_visible
            && x >= toolbar.property_bar_rect.left
            && x < toolbar.property_bar_rect.right
            && y >= toolbar.property_bar_rect.top
            && y < toolbar.property_bar_rect.bottom
        {
            return true;
        }

        let mut state = match state_arc.lock() {
            Ok(s) => s,
            Err(_) => return false,
        };
        // End any existing text editing
        // End any existing text editing
        for obj in &mut state.objects {
            obj.is_editing_text = false;
        }
        InteractionHandler::handle_mouse_down(&mut state, toolbar, x, y);
        false
    }

    pub fn handle_mouse_move(
        state_arc: &Arc<Mutex<state::OverlayState>>,
        toolbar: &mut toolbar::Toolbar,
        x: i32,
        y: i32,
    ) -> bool {
        let mut needs_render = false;

        // 1. Handle Toolbar Hover
        if toolbar.handle_mouse_move(x, y) {
            needs_render = true;
        }

        // Handle interaction logic first
        {
            if let Ok(mut state) = state_arc.lock() {
                InteractionHandler::handle_mouse_move(&mut state, toolbar, x, y);
            }
        }

        let (_mode, _zone) = {
            let state = match state_arc.lock() {
                Ok(s) => s,
                Err(_) => return needs_render,
            };
            if let Some(sel) = state.selection {
                toolbar.update_layout(
                    sel,
                    state.x,
                    state.y,
                    state.width,
                    state.height,
                    state.enable_advanced_effects,
                );
            } else {
                toolbar.hide();
            }

            (state.interaction_mode, state.hover_zone)
        };

        // 2. Cursor Logic Removed - Handled by WM_SETCURSOR now
        needs_render
    }

    pub fn get_current_cursor(
        state_arc: &Arc<Mutex<state::OverlayState>>,
        toolbar: &toolbar::Toolbar,
    ) -> anyhow::Result<windows::core::PCWSTR> {
        let state = match state_arc.lock() {
            Ok(s) => s,
            Err(_) => return Err(anyhow::anyhow!("State mutex poisoned")),
        };

        // 1. Toolbar Precedence (implicitly uses Arrow via Default if not handled)
        if toolbar.visible {
            // If mouse is within toolbar rect, use Arrow
            if state.mouse_x >= toolbar.rect.left
                && state.mouse_x < toolbar.rect.right
                && state.mouse_y >= toolbar.rect.top
                && state.mouse_y < toolbar.rect.bottom
            {
                return Ok(windows::Win32::UI::WindowsAndMessaging::IDC_ARROW);
            }
            if toolbar.property_bar_visible
                && state.mouse_x >= toolbar.property_bar_rect.left
                && state.mouse_x < toolbar.property_bar_rect.right
                && state.mouse_y >= toolbar.property_bar_rect.top
                && state.mouse_y < toolbar.property_bar_rect.bottom
            {
                return Ok(windows::Win32::UI::WindowsAndMessaging::IDC_ARROW);
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
                    windows::Win32::UI::WindowsAndMessaging::IDC_ARROW
                } else {
                    windows::Win32::UI::WindowsAndMessaging::IDC_CROSS
                }
            }
            state::InteractionMode::Moving => windows::Win32::UI::WindowsAndMessaging::IDC_SIZEALL,
            state::InteractionMode::Resizing(z) | state::InteractionMode::TransformingObject(z) => {
                match z {
                    state::HitZone::Top | state::HitZone::Bottom => {
                        windows::Win32::UI::WindowsAndMessaging::IDC_SIZENS
                    }
                    state::HitZone::Left | state::HitZone::Right => {
                        windows::Win32::UI::WindowsAndMessaging::IDC_SIZEWE
                    }
                    state::HitZone::TopLeft | state::HitZone::BottomRight => {
                        windows::Win32::UI::WindowsAndMessaging::IDC_SIZENWSE
                    }
                    state::HitZone::TopRight | state::HitZone::BottomLeft => {
                        windows::Win32::UI::WindowsAndMessaging::IDC_SIZENESW
                    }
                    state::HitZone::Body | state::HitZone::Stroke => {
                        windows::Win32::UI::WindowsAndMessaging::IDC_SIZEALL
                    }
                    _ => windows::Win32::UI::WindowsAndMessaging::IDC_ARROW,
                }
            }
            state::InteractionMode::None => {
                // Check if hovering an OBJECT first (Priority)
                let mut object_cursor = None;

                for (_idx, obj) in state.objects.iter().enumerate().rev() {
                    let zone = obj.hit_test(state.mouse_x, state.mouse_y);
                    if !matches!(zone, state::HitZone::None) {
                        // If hovering an object stroke/handle
                        object_cursor = Some(match zone {
                            state::HitZone::Body | state::HitZone::Stroke => {
                                windows::Win32::UI::WindowsAndMessaging::IDC_SIZEALL
                            }
                            state::HitZone::Top | state::HitZone::Bottom => {
                                windows::Win32::UI::WindowsAndMessaging::IDC_SIZENS
                            }
                            state::HitZone::Left | state::HitZone::Right => {
                                windows::Win32::UI::WindowsAndMessaging::IDC_SIZEWE
                            }
                            state::HitZone::TopLeft | state::HitZone::BottomRight => {
                                windows::Win32::UI::WindowsAndMessaging::IDC_SIZENWSE
                            }
                            state::HitZone::TopRight | state::HitZone::BottomLeft => {
                                windows::Win32::UI::WindowsAndMessaging::IDC_SIZENESW
                            }
                            _ => windows::Win32::UI::WindowsAndMessaging::IDC_ARROW,
                        });
                        break;
                    }
                }

                if let Some(cursor) = object_cursor {
                    cursor
                } else if state.current_tool != state::DrawingTool::None
                    && matches!(state.hover_zone, state::HitZone::Body)
                {
                    // Only use CROSS inside the drawing area (Body) for specific tools
                    if matches!(
                        state.current_tool,
                        state::DrawingTool::Brush | state::DrawingTool::Mosaic
                    ) {
                        // Use ARROW so the user only sees the Circle drawn by GDI
                        windows::Win32::UI::WindowsAndMessaging::IDC_ARROW
                    } else {
                        windows::Win32::UI::WindowsAndMessaging::IDC_CROSS
                    }
                } else {
                    match state.hover_zone {
                        state::HitZone::Stroke => {
                            windows::Win32::UI::WindowsAndMessaging::IDC_SIZEALL
                        }
                        state::HitZone::Top | state::HitZone::Bottom => {
                            windows::Win32::UI::WindowsAndMessaging::IDC_SIZENS
                        }
                        state::HitZone::Left | state::HitZone::Right => {
                            windows::Win32::UI::WindowsAndMessaging::IDC_SIZEWE
                        }
                        state::HitZone::TopLeft | state::HitZone::BottomRight => {
                            windows::Win32::UI::WindowsAndMessaging::IDC_SIZENWSE
                        }
                        state::HitZone::TopRight | state::HitZone::BottomLeft => {
                            windows::Win32::UI::WindowsAndMessaging::IDC_SIZENESW
                        }
                        state::HitZone::Body => windows::Win32::UI::WindowsAndMessaging::IDC_ARROW,
                        _ => windows::Win32::UI::WindowsAndMessaging::IDC_ARROW,
                    }
                }
            }
        })
    }

    pub fn handle_mouse_up(
        state_arc: &Arc<Mutex<state::OverlayState>>,
        toolbar: &mut toolbar::Toolbar,
        x: i32,
        y: i32,
    ) -> (Option<toolbar::ToolType>, bool) {
        // 1. Check Toolbar Trigger
        if let Some(cmd) = toolbar.handle_mouse_up(x, y) {
            return (Some(cmd), true);
        }

        // 1.5 Check Property Bar Trigger (Color/FontSize Selection)
        let enable_advanced = state_arc
            .lock()
            .map(|s| s.enable_advanced_effects)
            .unwrap_or(false);
        if let Some(change) = toolbar.handle_property_click(x, y, enable_advanced) {
            if let Ok(mut state) = state_arc.lock() {
                state.apply_property_change(change);
                return (None, true); // re-render
            }
        }

        // 2. Regular Interaction
        if let Ok(mut state) = state_arc.lock() {
            InteractionHandler::handle_mouse_up(&mut state, toolbar);

            if let Some(sel) = state.selection {
                if matches!(state.interaction_mode, state::InteractionMode::None) {
                    toolbar.update_layout(
                        sel,
                        state.x,
                        state.y,
                        state.width,
                        state.height,
                        state.enable_advanced_effects,
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
        state_arc: &Arc<Mutex<state::OverlayState>>,
        x: i32,
        y: i32,
    ) -> bool {
        let mut state = match state_arc.lock() {
            Ok(s) => s,
            Err(_) => return false,
        };

        // 1. Check if we hit an existing object for editing
        let mut hit_idx = None;
        for (idx, obj) in state.objects.iter().enumerate().rev() {
            if !matches!(obj.hit_test(x, y), state::HitZone::None) {
                hit_idx = Some(idx);
                break;
            }
        }

        if let Some(idx) = hit_idx {
            if state.objects[idx].tool == state::DrawingTool::Text {
                // Enter editing mode for this text
                state.objects[idx].is_editing_text = true;
                state.selected_object_index = Some(idx);
                return false; // Return false so we DON'T trigger global save/clipboard
            }
        }

        // 2. Fallback to selection copy logic
        if let Some(sel) = state.selection {
            if x > sel.left && x < sel.right && y > sel.top && y < sel.bottom {
                return true; // Trigger Copy (global save)
            }
        }
        false
    }

    pub fn handle_char(state_arc: &Arc<Mutex<state::OverlayState>>, char_code: u32) -> bool {
        let mut state = match state_arc.lock() {
            Ok(s) => s,
            Err(_) => return false,
        };
        if let Some(drawing) = state.objects.iter_mut().rev().find(|o| o.is_editing_text) {
            let c = char::from_u32(char_code).unwrap_or('\0');
            if c == '\u{8}' {
                // Backspace
                if let Some(text) = &mut drawing.text {
                    text.pop();
                }
            } else if !c.is_control() {
                if drawing.text.is_none() {
                    drawing.text = Some(String::new());
                }
                drawing.text.as_mut().unwrap().push(c);
            }
            return true;
        }
        false
    }
}
