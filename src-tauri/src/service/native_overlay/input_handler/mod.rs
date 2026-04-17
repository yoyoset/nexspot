use crate::service::native_overlay::render::toolbar;
use crate::service::native_overlay::state;
use std::sync::{Arc, RwLock};

pub mod mouse;
pub mod keyboard;
pub mod cursor;

pub struct InputHandler;

impl InputHandler {
    pub fn handle_char(state: &Arc<RwLock<state::OverlayState>>, char_code: u32) -> bool {
        keyboard::handle_char(state, char_code)
    }

    pub fn handle_mouse_down(
        state: &Arc<RwLock<state::OverlayState>>,
        toolbar: &mut toolbar::Toolbar,
        x: i32,
        y: i32,
    ) -> bool {
        mouse::handle_mouse_down(state, toolbar, x, y)
    }

    pub fn handle_mouse_move(
        state: &Arc<RwLock<state::OverlayState>>,
        toolbar: &mut toolbar::Toolbar,
        x: i32,
        y: i32,
    ) -> bool {
        mouse::handle_mouse_move(state, toolbar, x, y)
    }

    pub fn handle_mouse_up(
        state: &Arc<RwLock<state::OverlayState>>,
        toolbar: &mut toolbar::Toolbar,
        x: i32,
        y: i32,
    ) -> (Option<toolbar::ToolType>, bool) {
        mouse::handle_mouse_up(state, toolbar, x, y)
    }

    pub fn handle_double_click(
        state: &Arc<RwLock<state::OverlayState>>,
        x: i32,
        y: i32,
    ) -> bool {
        mouse::handle_double_click(state, x, y)
    }

    pub fn handle_keyboard(
        state: &Arc<RwLock<state::OverlayState>>,
        toolbar: &mut toolbar::Toolbar,
        vk: u16,
        is_down: bool,
    ) -> bool {
        keyboard::handle_keyboard(state, toolbar, vk, is_down)
    }

    pub fn handle_cursor(
        state: &Arc<RwLock<state::OverlayState>>,
        toolbar: &toolbar::Toolbar,
        x: i32,
        y: i32,
    ) -> bool {
        cursor::handle_cursor(state, toolbar, x, y)
    }

    pub fn get_current_cursor(
        state: &Arc<RwLock<state::OverlayState>>,
        toolbar: &toolbar::Toolbar,
    ) -> anyhow::Result<windows::core::PCWSTR> {
        cursor::get_current_cursor(state, toolbar)
    }
}
