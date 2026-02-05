use crate::service::native_overlay::interaction::InteractionHandler;
use crate::service::native_overlay::render::toolbar;
use crate::service::native_overlay::state;
use crate::service::win32;
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

        let mut state = state_arc.lock().unwrap();
        InteractionHandler::handle_mouse_down(&mut state, x, y);
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

        let (mode, zone) = {
            let mut state = state_arc.lock().unwrap();
            InteractionHandler::handle_mouse_move(&mut state, x, y);

            if let Some(sel) = state.selection {
                toolbar.update_layout(sel, state.width, state.height);
            } else {
                toolbar.hide();
            }

            (state.interaction_mode, state.hover_zone)
        };

        // 2. Cursor Logic
        let cursor = match mode {
            state::InteractionMode::Selecting => windows::Win32::UI::WindowsAndMessaging::IDC_CROSS,
            state::InteractionMode::Drawing => windows::Win32::UI::WindowsAndMessaging::IDC_CROSS,
            state::InteractionMode::Moving => windows::Win32::UI::WindowsAndMessaging::IDC_SIZEALL,
            state::InteractionMode::Resizing(z) => match z {
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
            },
            state::InteractionMode::None => match zone {
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
                state::HitZone::Body => windows::Win32::UI::WindowsAndMessaging::IDC_SIZEALL,
                _ => windows::Win32::UI::WindowsAndMessaging::IDC_ARROW,
            },
        };
        let _ = win32::window::set_system_cursor(cursor);

        needs_render
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

        // 2. Regular Interaction
        let mut state = state_arc.lock().unwrap();
        InteractionHandler::handle_mouse_up(&mut state);

        if let Some(sel) = state.selection {
            if matches!(state.interaction_mode, state::InteractionMode::None) {
                toolbar.update_layout(sel, state.width, state.height);
            } else {
                toolbar.hide();
            }
        } else {
            toolbar.hide();
        }

        (None, true)
    }

    pub fn handle_double_click(
        state_arc: &Arc<Mutex<state::OverlayState>>,
        x: i32,
        y: i32,
    ) -> bool {
        let state = state_arc.lock().unwrap();
        if let Some(sel) = state.selection {
            if x > sel.left && x < sel.right && y > sel.top && y < sel.bottom {
                return true; // Trigger Copy
            }
        }
        false
    }
}
