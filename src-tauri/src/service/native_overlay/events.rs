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
                let local_x = (lparam.0 & 0xFFFF) as i16 as i32;
                let local_y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
                let (gx, gy) = {
                    let s = self.state.lock().unwrap();
                    (local_x + s.x, local_y + s.y)
                };
                self.on_mouse_down(gx, gy);
                return Some(LRESULT(0));
            }
            WindowsAndMessaging::WM_LBUTTONUP => {
                let local_x = (lparam.0 & 0xFFFF) as i16 as i32;
                let local_y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
                let (gx, gy) = {
                    let s = self.state.lock().unwrap();
                    (local_x + s.x, local_y + s.y)
                };
                self.on_mouse_up(gx, gy);
                return Some(LRESULT(0));
            }
            WindowsAndMessaging::WM_LBUTTONDBLCLK => {
                let local_x = (lparam.0 & 0xFFFF) as i16 as i32;
                let local_y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
                let (gx, gy) = {
                    let s = self.state.lock().unwrap();
                    (local_x + s.x, local_y + s.y)
                };
                self.on_double_click(gx, gy);
                return Some(LRESULT(0));
            }
            WindowsAndMessaging::WM_MOUSEMOVE => {
                let local_x = (lparam.0 & 0xFFFF) as i16 as i32;
                let local_y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
                let (gx, gy) = {
                    let s = self.state.lock().unwrap();
                    (local_x + s.x, local_y + s.y)
                };
                self.on_mouse_move(gx, gy);
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
