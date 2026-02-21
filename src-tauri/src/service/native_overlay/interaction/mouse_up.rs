use crate::service::native_overlay::state::{self, OverlayState};

pub fn handle_mouse_up(
    state: &mut OverlayState,
    toolbar: &mut crate::service::native_overlay::render::toolbar::Toolbar,
) {
    if toolbar.is_dragging_opacity {
        toolbar.is_dragging_opacity = false;
        return;
    }
    if let state::InteractionMode::Drawing = state.interaction_mode {
        if let Some(drawing) = state.current_drawing.take() {
            if drawing.points.len() >= 2
                || matches!(drawing.tool, state::DrawingTool::Number)
                || matches!(drawing.tool, state::DrawingTool::Text)
            {
                state.objects.push(drawing);

                // Continuous Drawing: Keep the current tool active.
                // User can click the tool again or press ESC to deselect.
                // state.current_tool = state::DrawingTool::None;
            }
        }
    }

    state.interaction_mode = state::InteractionMode::None;
}
