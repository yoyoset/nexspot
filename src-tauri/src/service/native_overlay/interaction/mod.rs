pub mod mouse_down;
pub mod mouse_move;
pub mod mouse_up;

use crate::service::native_overlay::state::OverlayState;

pub struct InteractionHandler;

impl InteractionHandler {
    pub fn handle_mouse_down(
        state: &mut OverlayState,
        toolbar: &mut crate::service::native_overlay::render::toolbar::Toolbar,
        x: i32,
        y: i32,
    ) {
        mouse_down::handle_mouse_down(state, toolbar, x, y);
    }

    pub fn handle_mouse_move(
        state: &mut OverlayState,
        toolbar: &mut crate::service::native_overlay::render::toolbar::Toolbar,
        x: i32,
        y: i32,
    ) {
        mouse_move::handle_mouse_move(state, toolbar, x, y);
    }

    pub fn handle_mouse_up(
        state: &mut OverlayState,
        toolbar: &mut crate::service::native_overlay::render::toolbar::Toolbar,
    ) {
        mouse_up::handle_mouse_up(state, toolbar);
    }
}
