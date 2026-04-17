use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{DefWindowProcW, GWLP_USERDATA, WM_NCDESTROY, GetWindowLongPtrW, SetWindowLongPtrW};
use super::types::{Dispatcher, SafeHWND, WindowEventHandler};

pub unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);

    if ptr != 0 {
        let dispatcher = &mut *(ptr as *mut Dispatcher);

        // Handle cleanup
        if msg == WM_NCDESTROY {
            let _ = Box::from_raw(dispatcher); // Take ownership and drop
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            return DefWindowProcW(hwnd, msg, wparam, lparam);
        }

        if let Some(res) = (*dispatcher.handler).on_message(hwnd, msg, wparam, lparam) {
            return res;
        }
    }

    DefWindowProcW(hwnd, msg, wparam, lparam)
}

/// Binds a handler to a window's GWLP_USERDATA.
/// The handler must outlive the window or be cleaned up manually.
pub fn set_window_handler(hwnd: HWND, handler: *mut dyn WindowEventHandler) {
    let dispatcher = Box::new(Dispatcher { handler });
    unsafe {
        SetWindowLongPtrW(
            hwnd,
            GWLP_USERDATA,
            Box::into_raw(dispatcher) as isize,
        );
    }
}

pub fn remove_window_handler(hwnd: &SafeHWND) {
    unsafe {
        let ptr = GetWindowLongPtrW(hwnd.0, GWLP_USERDATA);

        if ptr != 0 {
            // Set to 0 FIRST to prevent concurrent access
            SetWindowLongPtrW(hwnd.0, GWLP_USERDATA, 0);
            // Reconstruct Box to drop it
            let _ = Box::from_raw(ptr as *mut Dispatcher);
        }
    }
}
