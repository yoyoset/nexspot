pub mod capture;
pub mod interaction;
pub mod render;
pub mod save;
pub mod snapping;
pub mod state;

use crate::service::win32::{self, window::SafeHWND, SendHWND};
use state::OverlayState;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    IDC_ARROW, IDC_CROSS, IDC_SIZEALL, IDC_SIZENESW, IDC_SIZENS, IDC_SIZENWSE, IDC_SIZEWE,
};

use render::toolbar;
use tauri::AppHandle;

pub struct OverlayManager {
    pub state: Arc<Mutex<OverlayState>>,
    pub windows: Vec<SendHWND>,
    pub toolbar: toolbar::Toolbar,
    pub last_render_time: Instant,
    pub app: AppHandle,
}

impl OverlayManager {
    pub fn new(app: AppHandle) -> anyhow::Result<Self> {
        let state = Arc::new(Mutex::new(OverlayState::default()));
        // Create MAIN overlay window immediately (hidden)
        let main_hwnd =
            win32::window::create_overlay_window("HyperLensOverlay", "HyperLens Overlay")?;

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
            unsafe {
                windows::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW(
                    send_hwnd.0,
                    windows::Win32::UI::WindowsAndMessaging::GWLP_USERDATA,
                    ptr as isize,
                );
            }
        }
    }

    pub fn close_and_reset(&mut self) {
        let mut state = self.state.lock().unwrap();
        state.is_visible = false;
        state.selection = None;
        state.interaction_mode = state::InteractionMode::None;
        state.hover_zone = state::HitZone::None;

        for send_hwnd in &self.windows {
            win32::window::hide_window(&win32::window::SafeHWND(send_hwnd.0));
        }
    }

    pub fn on_mouse_down(&mut self, _hwnd: HWND, x: i32, y: i32) {
        if let Some(cmd) = self.toolbar.handle_click(x, y) {
            match cmd {
                toolbar::ToolbarCommand::Cancel => self.close_and_reset(),
                toolbar::ToolbarCommand::Save => {
                    let _ = self.save_selection();
                }
                _ => {}
            }
            return;
        }

        let mut state = self.state.lock().unwrap();
        interaction::InteractionHandler::handle_mouse_down(&mut state, x, y);
    }

    pub fn on_mouse_up(&mut self, _hwnd: HWND, _x: i32, _y: i32) {
        let mut state = self.state.lock().unwrap();
        interaction::InteractionHandler::handle_mouse_up(&mut state);

        if let Some(sel) = state.selection {
            self.toolbar.update_layout(sel, state.width, state.height);
        }

        drop(state);
        let _ = self.render_frame();
    }

    pub fn on_double_click(&mut self, _hwnd: HWND, x: i32, y: i32) {
        let state = self.state.lock().unwrap();
        if let Some(sel) = state.selection {
            // Simple hit test for body
            if x > sel.left && x < sel.right && y > sel.top && y < sel.bottom {
                drop(state);
                let _ = self.save_clipboard();
            }
        }
    }

    pub fn on_mouse_move(&mut self, _hwnd: HWND, x: i32, y: i32) {
        let (mode, zone) = {
            let mut state = self.state.lock().unwrap();
            interaction::InteractionHandler::handle_mouse_move(&mut state, x, y);

            if let Some(sel) = state.selection {
                if matches!(state.interaction_mode, state::InteractionMode::None) {
                    self.toolbar.update_layout(sel, state.width, state.height);
                } else {
                    self.toolbar.hide();
                }
            } else {
                self.toolbar.hide();
            }

            (state.interaction_mode, state.hover_zone)
        };

        // Cursor Logic
        let cursor = match mode {
            state::InteractionMode::Selecting => IDC_CROSS,
            state::InteractionMode::Moving => IDC_SIZEALL,
            state::InteractionMode::Resizing(z) => match z {
                state::HitZone::Top | state::HitZone::Bottom => IDC_SIZENS,
                state::HitZone::Left | state::HitZone::Right => IDC_SIZEWE,
                state::HitZone::TopLeft | state::HitZone::BottomRight => IDC_SIZENWSE,
                state::HitZone::TopRight | state::HitZone::BottomLeft => IDC_SIZENESW,
                _ => IDC_ARROW,
            },
            state::InteractionMode::None => match zone {
                state::HitZone::Top | state::HitZone::Bottom => IDC_SIZENS,
                state::HitZone::Left | state::HitZone::Right => IDC_SIZEWE,
                state::HitZone::TopLeft | state::HitZone::BottomRight => IDC_SIZENWSE,
                state::HitZone::TopRight | state::HitZone::BottomLeft => IDC_SIZENESW,
                state::HitZone::Body => IDC_SIZEALL,
                _ => IDC_ARROW,
            },
        };
        let _ = win32::window::set_system_cursor(cursor);

        let now = Instant::now();
        if now.duration_since(self.last_render_time).as_millis() >= 16 {
            let _ = self.render_frame();
            self.last_render_time = now;
        }
    }

    pub fn render_frame(&mut self) -> anyhow::Result<()> {
        let state = self.state.lock().unwrap();
        if let Some(send_hwnd) = self.windows.first() {
            let hwnd = SafeHWND(send_hwnd.0);
            return render::render_frame(&hwnd, &state, &mut self.toolbar);
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
        let state = self.state.lock().unwrap();
        // Get the dimmed bitmap from state to paint initial frame
        let hbitmap_dim = state
            .hbitmap_dim
            .as_ref()
            .map(|h| h.0)
            .ok_or(anyhow::anyhow!("No dim bitmap"))?;

        if let Some(send_hwnd) = self.windows.first() {
            let hwnd = SafeHWND(send_hwnd.0);
            // 1. Move & Resize (Don't show yet to prevent flash of old content)
            // We use HWND_TOPMOST to guarantee it stays on top of everything
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

            // 2. Immediate paint (Refresh Layered Window)
            let hdc_screen = win32::gdi::create_compatible_dc(None)?;
            let hdc_temp = win32::gdi::create_compatible_dc(Some(&hdc_screen))?;
            win32::gdi::select_object(
                &hdc_temp,
                windows::Win32::Graphics::Gdi::HGDIOBJ(hbitmap_dim.0),
            )?;

            let _ = win32::window::update_layered_window(
                &hwnd,
                &hdc_temp,
                &windows::Win32::Foundation::POINT { x, y },
                &windows::Win32::Foundation::SIZE {
                    cx: width,
                    cy: height,
                },
                255,
                0,
            );

            // 3. Force Focus & Show
            unsafe {
                // Show NOW, after content is updated
                let _ = windows::Win32::UI::WindowsAndMessaging::ShowWindow(
                    hwnd.0,
                    windows::Win32::UI::WindowsAndMessaging::SW_SHOW,
                );

                // Critical for "First Response": Force foreground
                let _ = windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow(hwnd.0);
                let _ = windows::Win32::UI::Input::KeyboardAndMouse::SetFocus(Some(hwnd.0));
            }
        }
        Ok(())
    }
}
