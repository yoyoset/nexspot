pub mod capture;
pub mod input_handler;
pub mod interaction;
pub mod ocr;
pub mod render;
pub mod save;
pub mod snapping;
pub mod state;

use crate::service::win32::{self, window::SafeHWND, SendHWND};
use state::OverlayState;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use windows::Win32::Foundation::HWND;

use input_handler::InputHandler;
use render::toolbar;
use tauri::{AppHandle, Manager};
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging;

pub struct OverlayManager {
    pub state: Arc<Mutex<OverlayState>>,
    pub windows: Vec<SendHWND>,
    pub toolbar: toolbar::Toolbar,
    pub last_render_time: Instant,
    pub app: AppHandle,
}

impl win32::window::WindowEventHandler for OverlayManager {
    fn on_message(
        &mut self,
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> Option<LRESULT> {
        match msg {
            WindowsAndMessaging::WM_KEYDOWN => {
                if wparam.0 == windows::Win32::UI::Input::KeyboardAndMouse::VK_ESCAPE.0 as usize {
                    self.close_and_reset();
                }
                return Some(LRESULT(0));
            }
            WindowsAndMessaging::WM_LBUTTONDOWN => {
                let x = (lparam.0 & 0xFFFF) as i16 as i32;
                let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
                self.on_mouse_down(hwnd, x, y);
                return Some(LRESULT(0));
            }
            WindowsAndMessaging::WM_LBUTTONUP => {
                let x = (lparam.0 & 0xFFFF) as i16 as i32;
                let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
                self.on_mouse_up(hwnd, x, y);
                return Some(LRESULT(0));
            }
            WindowsAndMessaging::WM_LBUTTONDBLCLK => {
                let x = (lparam.0 & 0xFFFF) as i16 as i32;
                let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
                self.on_double_click(hwnd, x, y);
                return Some(LRESULT(0));
            }
            WindowsAndMessaging::WM_MOUSEMOVE => {
                let x = (lparam.0 & 0xFFFF) as i16 as i32;
                let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
                self.on_mouse_move(hwnd, x, y);
                return Some(LRESULT(0));
            }
            _ => {}
        }
        None
    }
}

impl OverlayManager {
    pub fn new(app: AppHandle) -> anyhow::Result<Self> {
        let state = Arc::new(Mutex::new(OverlayState::default()));
        // Create MAIN overlay window immediately (hidden)
        let main_hwnd =
            win32::window::create_overlay_window("HyperLensOverlay", "HyperLens Overlay")?;

        // 1. Register Custom Font from memory (Embedded)
        static FONT_DATA: &[u8] = include_bytes!("../../../resources/remixicon.ttf");
        match win32::gdi::add_font_mem(FONT_DATA) {
            Ok(_) => log::info!("Registered remixicon font from embedded memory"),
            Err(e) => {
                log::error!("Failed to register embedded remixicon font: {:?}", e);
                // Fallback to file system if embedding fails (unlikely, but safe)
                if let Ok(resource_dir) = app.path().resource_dir() {
                    let font_path = resource_dir.join("remixicon.ttf");
                    if font_path.exists() {
                        let _ = win32::gdi::register_font(&font_path);
                    }
                }
            }
        }

        Ok(Self {
            state,
            windows: vec![SendHWND(main_hwnd.0)],
            toolbar: toolbar::Toolbar::new(),
            last_render_time: Instant::now(),
            app,
        })
    }

    pub fn set_user_data(&self, ptr: *mut Self) {
        for send_hwnd in &self.windows {
            win32::window::set_window_handler(
                send_hwnd.0,
                ptr as *mut dyn win32::window::WindowEventHandler,
            );
        }
    }

    pub fn close_and_reset(&mut self) {
        let mut state = self.state.lock().unwrap();
        state.is_visible = false;
        state.selection = None;
        state.interaction_mode = state::InteractionMode::None;
        state.hover_zone = state::HitZone::None;
        state.gdi_cache.clear();

        for send_hwnd in &self.windows {
            win32::window::hide_window(&win32::window::SafeHWND(send_hwnd.0));
        }
    }

    pub fn on_mouse_down(&mut self, _hwnd: HWND, x: i32, y: i32) {
        if InputHandler::handle_mouse_down(&self.state, &mut self.toolbar, x, y) {
            let _ = self.render_frame();
        }
    }

    pub fn on_mouse_up(&mut self, _hwnd: HWND, x: i32, y: i32) {
        let (cmd, needs_render) =
            InputHandler::handle_mouse_up(&self.state, &mut self.toolbar, x, y);

        if let Some(tool) = cmd {
            self.execute_toolbar_command(tool);
        }

        if needs_render {
            let _ = self.render_frame();
        }
    }

    fn execute_toolbar_command(&mut self, cmd: toolbar::ToolType) {
        match cmd {
            toolbar::ToolType::Cancel => self.close_and_reset(),
            toolbar::ToolType::Save => {
                let _ = self.save_selection();
            }
            toolbar::ToolType::Copy => {
                let _ = self.save_clipboard();
            }
            toolbar::ToolType::Pin => {
                log::info!("Pin requested");
            }
            // Drawing Tools
            toolbar::ToolType::Rect => {
                let mut state = self.state.lock().unwrap();
                state.current_tool = state::DrawingTool::Rect;
                self.toolbar.current_tool = Some(toolbar::ToolType::Rect);
            }
            toolbar::ToolType::Ellipse => {
                let mut state = self.state.lock().unwrap();
                state.current_tool = state::DrawingTool::Ellipse;
                self.toolbar.current_tool = Some(toolbar::ToolType::Ellipse);
            }
            toolbar::ToolType::Arrow => {
                let mut state = self.state.lock().unwrap();
                state.current_tool = state::DrawingTool::Arrow;
                self.toolbar.current_tool = Some(toolbar::ToolType::Arrow);
            }
            toolbar::ToolType::Line => {
                let mut state = self.state.lock().unwrap();
                state.current_tool = state::DrawingTool::Line;
                self.toolbar.current_tool = Some(toolbar::ToolType::Line);
            }
            toolbar::ToolType::Brush => {
                let mut state = self.state.lock().unwrap();
                state.current_tool = state::DrawingTool::Brush;
                self.toolbar.current_tool = Some(toolbar::ToolType::Brush);
            }
            toolbar::ToolType::Mosaic => {
                let mut state = self.state.lock().unwrap();
                state.current_tool = state::DrawingTool::Mosaic;
                self.toolbar.current_tool = Some(toolbar::ToolType::Mosaic);
            }
            toolbar::ToolType::Text => {
                let mut state = self.state.lock().unwrap();
                state.current_tool = state::DrawingTool::Text;
                self.toolbar.current_tool = Some(toolbar::ToolType::Text);
            }
            toolbar::ToolType::Number => {
                let mut state = self.state.lock().unwrap();
                state.current_tool = state::DrawingTool::Number;
                self.toolbar.current_tool = Some(toolbar::ToolType::Number);
            }
            toolbar::ToolType::Ocr => {
                let _ = self.save_ocr();
            }
            _ => {
                log::info!("Command not implemented or handled elsewhere: {:?}", cmd);
            }
        }
    }

    pub fn on_double_click(&mut self, _hwnd: HWND, x: i32, y: i32) {
        if InputHandler::handle_double_click(&self.state, x, y) {
            let mode = {
                let state = self.state.lock().unwrap();
                state.capture_mode
            };

            if mode == state::CaptureMode::Ocr {
                let _ = self.save_ocr();
            } else {
                let _ = self.save_clipboard();
            }
        }
    }

    fn save_ocr(&mut self) -> anyhow::Result<()> {
        ocr::perform_ocr(&self.state, &self.app)?;
        self.close_and_reset();
        Ok(())
    }

    pub fn on_mouse_move(&mut self, _hwnd: HWND, x: i32, y: i32) {
        if InputHandler::handle_mouse_move(&self.state, &mut self.toolbar, x, y) {
            let _ = self.render_frame();
        }

        let now = Instant::now();
        if now.duration_since(self.last_render_time).as_millis() >= 16 {
            let _ = self.render_frame();
            self.last_render_time = now;
        }
    }

    pub fn render_frame(&mut self) -> anyhow::Result<()> {
        let mut state = self.state.lock().unwrap();
        if let Some(send_hwnd) = self.windows.first() {
            let hwnd = SafeHWND(send_hwnd.0);
            return render::render_frame(&hwnd, &mut state, &mut self.toolbar);
        }
        Ok(())
    }

    fn save_selection(&mut self) -> anyhow::Result<()> {
        save::save_selection(&self.state, &self.app)?;
        self.close_and_reset();
        Ok(())
    }

    fn save_clipboard(&mut self) -> anyhow::Result<()> {
        save::copy_to_clipboard(&self.state, &self.app)?;
        self.close_and_reset();
        Ok(())
    }

    pub fn show_overlay_at(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> anyhow::Result<()> {
        let mode = {
            let state = self.state.lock().unwrap();
            if state.hbitmap_dim.is_none() {
                return Err(anyhow::anyhow!("No dim bitmap"));
            }
            state.capture_mode
        };

        // Rebuild toolbar for the current mode
        self.toolbar.rebuild_for_mode(mode);

        if let Some(send_hwnd) = self.windows.first() {
            let hwnd = SafeHWND(send_hwnd.0);
            // 1. Move & Resize
            unsafe {
                let _ = windows::Win32::UI::WindowsAndMessaging::SetWindowPos(
                    hwnd.0,
                    Some(windows::Win32::UI::WindowsAndMessaging::HWND_TOPMOST),
                    x,
                    y,
                    width,
                    height,
                    windows::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE,
                );
            }

            // 2. Immediate full render (includes magnifier, UI, etc.)
            let _ = self.render_frame();

            // 3. Force Focus & Show
            unsafe {
                let _ = windows::Win32::UI::WindowsAndMessaging::ShowWindow(
                    hwnd.0,
                    windows::Win32::UI::WindowsAndMessaging::SW_SHOW,
                );
                let _ = windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow(hwnd.0);
                let _ = windows::Win32::UI::Input::KeyboardAndMouse::SetFocus(Some(hwnd.0));
            }
        }
        Ok(())
    }
}
