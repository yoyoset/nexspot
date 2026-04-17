use crate::service::native_overlay::input_handler::InputHandler;
use crate::service::native_overlay::manager::OverlayManager;
use crate::service::win32;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging;

impl win32::window::WindowEventHandler for OverlayManager {
    fn on_message(
        &mut self,
        _hwnd: HWND,
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
            WindowsAndMessaging::WM_CHAR => {
                if InputHandler::handle_char(&self.state, wparam.0 as u32) {
                    let _ = self.render_frame();
                }
                return Some(LRESULT(0));
            }
            WindowsAndMessaging::WM_SETCURSOR => {
                let hit_zone_low = (lparam.0 & 0xFFFF) as u32;
                if hit_zone_low == WindowsAndMessaging::HTCLIENT {
                    if let Ok(cursor) = InputHandler::get_current_cursor(&self.state, &self.toolbar)
                    {
                        let _ = win32::window::set_system_cursor(cursor);
                        return Some(LRESULT(1));
                    }
                }
            }
            WindowsAndMessaging::WM_LBUTTONDOWN => {
                let mut cursor = windows::Win32::Foundation::POINT::default();
                unsafe {
                    let _ = windows::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut cursor);
                }
                let (nx, ny, hwnd_to_capture) = {
                    let s = self.state.read().unwrap();
                    let h = self.active_hwnd(s.capture_engine, &s.monitor_id).unwrap().0;
                    (cursor.x, cursor.y, h)
                };
                unsafe {
                    let _ =
                        windows::Win32::UI::Input::KeyboardAndMouse::SetCapture(hwnd_to_capture);
                }
                self.on_mouse_down(nx, ny);
                return Some(LRESULT(0));
            }
            WindowsAndMessaging::WM_LBUTTONUP => {
                let mut cursor = windows::Win32::Foundation::POINT::default();
                unsafe {
                    let _ = windows::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut cursor);
                }
                let (nx, ny) = {
                    let _s = self.state.read().unwrap();
                    (cursor.x, cursor.y)
                };
                unsafe {
                    let _ = windows::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture();
                }
                self.on_mouse_up(nx, ny);
                return Some(LRESULT(0));
            }
            WindowsAndMessaging::WM_LBUTTONDBLCLK => {
                let mut cursor = windows::Win32::Foundation::POINT::default();
                unsafe {
                    let _ = windows::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut cursor);
                }
                let (nx, ny) = {
                    let _s = self.state.read().unwrap();
                    (cursor.x, cursor.y)
                };
                self.on_double_click(nx, ny);
                return Some(LRESULT(0));
            }
            WindowsAndMessaging::WM_MOUSEMOVE => {
                let mut cursor = windows::Win32::Foundation::POINT::default();
                unsafe {
                    let _ = windows::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut cursor);
                }
                let (nx, ny) = {
                    let _s = self.state.read().unwrap();
                    (cursor.x, cursor.y)
                };
                log::trace!("[Mouse] Move Local:({},{})", nx, ny);
                self.on_mouse_move(nx, ny);
                return Some(LRESULT(0));
            }
            WindowsAndMessaging::WM_TIMER => {
                if wparam.0 == 1 {
                    let _ = self.render_frame();
                }
                return Some(LRESULT(0));
            }
            _ => {}
        }
        None
    }
}
