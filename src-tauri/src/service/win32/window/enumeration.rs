use windows::core::BOOL;
use windows::Win32::Foundation::{HWND, LPARAM, RECT};
use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, IsIconic, IsWindowVisible, GetWindowRect};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    if IsWindowVisible(hwnd).as_bool() && !IsIconic(hwnd).as_bool() {
        let mut rect = RECT::default();
        let mut success = false;

        // 1. Try DWM Extended Frame Bounds
        let ptr = &mut rect as *mut _ as *mut std::ffi::c_void;
        let size = std::mem::size_of::<RECT>() as u32;
        if DwmGetWindowAttribute(hwnd, DWMWA_EXTENDED_FRAME_BOUNDS, ptr, size).is_ok() {
            success = true;
        }
        // 2. Fallback
        else if GetWindowRect(hwnd, &mut rect).is_ok() {
            success = true;
        }

        if success {
            let w = rect.right - rect.left;
            let h = rect.bottom - rect.top;
            if w > 10 && h > 10 {
                let vec_ptr = lparam.0 as *mut Vec<RECT>;
                (&mut *vec_ptr).push(rect);
            }
        }
    }
    BOOL(1)
}

pub fn enumerate_visible_windows() -> Vec<RECT> {
    unsafe {
        let mut rects = Vec::with_capacity(128);
        let ptr = &mut rects as *mut Vec<RECT> as isize;
        let _ = EnumWindows(Some(enum_windows_proc), LPARAM(ptr));
        rects
    }
}
