use super::render::toolbar::ToolType;
use super::OverlayManager;
use crate::service::native_overlay::state::DrawingTool;
use crate::service::notification::{notify_error, notify_warn};
use tauri::Manager;

pub struct CommandExecutor;

impl CommandExecutor {
    pub fn execute(manager: &mut OverlayManager, cmd: ToolType) {
        log::debug!("Executing Toolbar Command: {:?}", cmd);

        match cmd {
            ToolType::Cancel => manager.close_and_reset(),
            ToolType::Save => {
                if let Err(e) = manager.save_selection() {
                    log::error!("Save selection failed: {:?}", e);
                    notify_error(&manager.app, "Save Failed", &e.to_string());
                }
            }
            ToolType::Copy => {
                if let Err(e) = manager.save_clipboard() {
                    log::error!("Copy to clipboard failed: {:?}", e);
                    notify_error(&manager.app, "Copy Failed", &e.to_string());
                }
            }
            ToolType::Pin => {
                log::info!("Pin requested - Not fully implemented yet");
                notify_warn(&manager.app, "Pin", "Pinning is coming soon!");
            }
            ToolType::More => {}
            // All Tool-set changes now only set the tool ID.
            // The renderer (drawing/mod.rs) will handle engine-specific impl.
            ToolType::Rect => Self::set_tool(manager, DrawingTool::Rect, ToolType::Rect),
            ToolType::Ellipse => Self::set_tool(manager, DrawingTool::Ellipse, ToolType::Ellipse),
            ToolType::Arrow => Self::set_tool(manager, DrawingTool::Arrow, ToolType::Arrow),
            ToolType::Line => Self::set_tool(manager, DrawingTool::Line, ToolType::Line),
            ToolType::Text => Self::set_tool(manager, DrawingTool::Text, ToolType::Text),
            ToolType::Number => Self::set_tool(manager, DrawingTool::Number, ToolType::Number),
            ToolType::Brush => Self::set_tool(manager, DrawingTool::Brush, ToolType::Brush),
            ToolType::Mosaic => Self::set_tool(manager, DrawingTool::Mosaic, ToolType::Mosaic),
            ToolType::Macro(id) => {
                let prompt = {
                    let state = manager.state.lock().unwrap();
                    state
                        .tool_registry
                        .macros
                        .iter()
                        .find(|m| m.id == id)
                        .map(|m| m.prompt.clone())
                };

                if let Some(prompt) = prompt {
                    log::info!("Executing AI Macro: {} -> {}", id, prompt);
                    Self::execute(manager, ToolType::AiExecute(prompt));
                } else {
                    log::error!("AI Macro not found in registry: {}", id);
                    notify_error(
                        &manager.app,
                        "Macro Error",
                        &format!("AI Shortcut '{}' not found in current overlay state.", id),
                    );
                }
            }
            ToolType::AiExecute(prompt) => {
                log::info!("Executing AI Action: {}", prompt);
                let pin_id = format!(
                    "pin-{}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_micros()
                );
                let content = format!(
                    "# AI Processing...\n> **Prompt:** {}\n\nAnalyzing captured region...",
                    prompt
                );

                let state = manager.app.state::<crate::service::pin::PinState>();
                state.add_pin(pin_id.clone(), content);

                if let Err(e) = crate::service::pin::create_text_pin_window(&manager.app, &pin_id) {
                    log::error!("Failed to create pin window: {:?}", e);
                    notify_error(&manager.app, "AI Start Failed", &e.to_string());
                } else {
                    manager.close_and_reset();
                }
            }
        }
    }

    fn set_tool(manager: &mut OverlayManager, drawing_tool: DrawingTool, ui_tool: ToolType) {
        if let Ok(mut state) = manager.state.lock() {
            if state.current_tool == drawing_tool {
                // Toggle OFF
                state.current_tool = DrawingTool::None;
                manager.toolbar.current_tool = None;
                log::debug!("Toggled tool OFF: {:?}", drawing_tool);
            } else {
                // Set New Tool
                state.current_tool = drawing_tool;
                manager.toolbar.current_tool = Some(ui_tool);
                log::debug!("Switched tool to: {:?}", drawing_tool);
            }
        }
    }
}
