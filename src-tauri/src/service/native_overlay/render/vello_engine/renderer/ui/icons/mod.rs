use crate::service::native_overlay::render::toolbar::types::ToolbarButton;

pub mod tools;
pub mod actions;

pub fn draw_button_icon(scene: &mut vello::Scene, btn: &ToolbarButton, is_active: bool) {
    use crate::service::native_overlay::render::toolbar::types::ToolType;

    match btn.tool {
        ToolType::Rect | ToolType::Ellipse | ToolType::Line | ToolType::Arrow | ToolType::Brush => {
            tools::draw_tool_icon(scene, btn, is_active);
        }
        ToolType::Text => {
            // Text is small enough to stay here or move to tools
            tools::draw_tool_icon(scene, btn, is_active);
        }
        ToolType::Mosaic | ToolType::Number | ToolType::Ocr | ToolType::Scrolling => {
            tools::draw_tool_icon(scene, btn, is_active);
        }
        ToolType::Undo | ToolType::Save | ToolType::SaveAs | ToolType::Cancel | ToolType::Pin | ToolType::Copy | ToolType::More => {
            actions::draw_action_icon(scene, btn, is_active);
        }
    }
}
