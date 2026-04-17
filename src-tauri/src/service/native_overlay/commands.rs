use super::render::toolbar::ToolType;
use super::OverlayManager;
use crate::service::native_overlay::state::DrawingTool;
use crate::service::notification::notify_error;
use tauri::{Emitter, Manager};
use uuid::Uuid;

pub struct CommandExecutor;

impl CommandExecutor {
    pub fn execute(manager: &mut OverlayManager, cmd: ToolType) {
        log::debug!("Executing Toolbar Command: {:?}", cmd);

        match cmd {
            ToolType::Cancel => manager.close_and_reset(),
            ToolType::Save => {
                if let Err(e) = manager.save_selection() {
                    log::error!("Save selection failed: {:?}", e);
                    notify_error(
                        &manager.app,
                        &crate::service::l10n::t(&manager.app, "backend.notification.save_failed_title", "Save Failed"),
                        &e.to_string(),
                    );
                }
            }
            ToolType::SaveAs => {
                if let Err(e) = manager.save_as() {
                    log::error!("SaveAs selection failed: {:?}", e);
                    notify_error(
                        &manager.app,
                        &crate::service::l10n::t(&manager.app, "backend.notification.save_failed_title", "Save Failed"),
                        &e.to_string(),
                    );
                }
            }
            ToolType::Copy => {
                if let Err(e) = manager.save_clipboard() {
                    log::error!("Copy to clipboard failed: {:?}", e);
                    notify_error(
                        &manager.app,
                        &crate::service::l10n::t(&manager.app, "backend.notification.copy_failed_title", "Copy Failed"),
                        &e.to_string(),
                    );
                }
            }
            ToolType::Pin => {
                let app = manager.app.clone();
                let state_arc = manager.state.clone();
                match crate::service::native_overlay::save::capture_to_base64(&state_arc, &app) {
                    Ok(b64) => {
                        if !b64.is_empty() {
                            let pin_id = Uuid::new_v4().to_string();
                            let pin_state = app.state::<crate::service::pin::PinState>();
                            pin_state.add_pin(pin_id, crate::service::pin::PinData::ImageBase64(b64));

                            manager.close_and_reset();

                            // Notify standard frontend
                            let _ = app.emit("pin-collection-updated", ());

                            // Open window
                            if let Err(e) = crate::service::pin::open_pin_collection_window(&app) {
                                log::error!("Failed to open pin collection: {}", e);
                                notify_error(
                                    &app,
                                    &crate::service::l10n::t(&app, "backend.notification.pin_failed_title", "Pin Failed"),
                                    &e.to_string(),
                                );
                            }
                        } else {
                            log::warn!("[Pin] Captured image base64 is empty");
                            notify_error(
                                &app,
                                &crate::service::l10n::t(&app, "backend.notification.pin_failed_title", "Pin Failed"),
                                &crate::service::l10n::t(&app, "backend.notification.capture_empty_body", "Captured image was empty"),
                            );
                        }
                    }
                    Err(e) => {
                        notify_error(
                            &app,
                            &crate::service::l10n::t(&app, "backend.notification.pin_failed_title", "Pin Failed"),
                            &e.to_string(),
                        );
                    }
                }
            }
            ToolType::Undo => {
                if let Ok(mut state) = manager.state.write() {
                    state.undo();
                }
            }
            ToolType::More => {}
            // All Tool-set changes now only set the tool ID.
            ToolType::Rect => Self::set_tool(manager, DrawingTool::Rect, ToolType::Rect),
            ToolType::Ellipse => Self::set_tool(manager, DrawingTool::Ellipse, ToolType::Ellipse),
            ToolType::Arrow => Self::set_tool(manager, DrawingTool::Arrow, ToolType::Arrow),
            ToolType::Line => Self::set_tool(manager, DrawingTool::Line, ToolType::Line),
            ToolType::Text => Self::set_tool(manager, DrawingTool::Text, ToolType::Text),
            ToolType::Number => Self::set_tool(manager, DrawingTool::Number, ToolType::Number),
            ToolType::Brush => Self::set_tool(manager, DrawingTool::Brush, ToolType::Brush),
            ToolType::Mosaic => Self::set_tool(manager, DrawingTool::Mosaic, ToolType::Mosaic),
            ToolType::Ocr => {
                let _ = manager.app.emit("overlay://trigger-ocr", ());
            }
            ToolType::Scrolling => {
                let _ = manager.app.emit("overlay://trigger-scrolling", ());
            }
        }
    }

    fn set_tool(manager: &mut OverlayManager, drawing_tool: DrawingTool, ui_tool: ToolType) {
        if let Ok(mut state) = manager.state.write() {
            if state.current_tool == drawing_tool {
                // Toggle OFF
                state.current_tool = DrawingTool::None;
                manager.toolbar.current_tool = None;
                log::debug!("Toggled tool OFF: {:?}", drawing_tool);
            } else {
                // Set New Tool
                state.current_tool = drawing_tool;
                manager.toolbar.current_tool = Some(ui_tool);

                // Special case: Mosaic and Rect default to M (Level 2) which is 4.0 stroke
                if drawing_tool == DrawingTool::Mosaic || drawing_tool == DrawingTool::Rect {
                    state.current_stroke = 4.0;
                }

                log::debug!("Switched tool to: {:?}", drawing_tool);
            }
        }
    }
}
