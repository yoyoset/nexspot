use crate::service::native_overlay::commands::CommandExecutor;
use crate::service::native_overlay::input_handler::InputHandler;
use crate::service::native_overlay::manager::OverlayManager;
use crate::service::native_overlay::render::toolbar;
use tauri_plugin_dialog::DialogExt;

impl OverlayManager {
    pub fn on_mouse_down(&mut self, x: i32, y: i32) {
        log::trace!("[Handler] Mouse Down at ({}, {})", x, y);
        if InputHandler::handle_mouse_down(&self.state, &mut self.toolbar, x, y) {
            let _ = self.render_frame();
        }
    }

    pub fn on_mouse_up(&mut self, x: i32, y: i32) {
        let (cmd, needs_render) =
            InputHandler::handle_mouse_up(&self.state, &mut self.toolbar, x, y);

        if let Some(tool) = cmd {
            self.execute_toolbar_command(tool);
        }

        if needs_render {
            let _ = self.render_frame();
        }
    }

    pub fn on_mouse_move(&mut self, x: i32, y: i32) {
        log::trace!("[Handler] Mouse Move to ({}, {})", x, y);
        if InputHandler::handle_mouse_move(&self.state, &mut self.toolbar, x, y) {
            let _ = self.render_frame();
        }

        let now = std::time::Instant::now();
        if now.duration_since(self.last_render_time).as_millis() >= 16 {
            let _ = self.render_frame();
            self.last_render_time = now;
        }
    }

    pub fn on_double_click(&mut self, x: i32, y: i32) {
        if InputHandler::handle_double_click(&self.state, x, y) {
            let quick_save = {
                use tauri::Manager;
                let app_state = self.app.state::<crate::AppState>();
                let config_guard = app_state.config_state.lock().unwrap_or_else(|e| e.into_inner());
                config_guard.config.quick_save
            };

            if quick_save {
                let _ = crate::service::native_overlay::save::save_selection(&self.state, &self.app);
                let _ = crate::service::native_overlay::save::copy_to_clipboard(&self.state, &self.app);
                self.close_and_reset();
            } else {
                let _ = self.save_clipboard();
            }
        }
    }

    pub fn execute_toolbar_command(&mut self, cmd: toolbar::ToolType) {
        if matches!(cmd, toolbar::ToolType::More) {
            let confirmed = self
                .app
                .dialog()
                .message("Switching to Advanced Mode (Direct2D/Vello) will provide higher quality rendering but may take a moment to initialize GPU resources.\n\nContinue?")
                .title("Advanced Mode Initiation")
                .kind(tauri_plugin_dialog::MessageDialogKind::Info)
                .buttons(tauri_plugin_dialog::MessageDialogButtons::OkCancel)
                .blocking_show();

            if !confirmed {
                log::info!("Advanced mode promotion cancelled by user");
                return;
            }

            if let Err(e) = self.upgrade_to_vello() {
                log::error!("Failed to upgrade to Vello: {:?}", e);
                let _ = self
                    .app
                    .dialog()
                    .message(format!("Failed to initialize Advanced Mode: {:?}", e))
                    .title("Error")
                    .kind(tauri_plugin_dialog::MessageDialogKind::Error)
                    .blocking_show();
                return;
            }
        }
        CommandExecutor::execute(self, cmd);
    }
}
