use crate::service::win32::gdi::SafeHDC;
use windows::core::BOOL;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, SIZE, WPARAM};
use windows::Win32::Graphics::Gdi::{AC_SRC_OVER, BLENDFUNCTION};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, RegisterClassExW, SetWindowPos, UpdateLayeredWindow,
    CS_DBLCLKS, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, SET_WINDOW_POS_FLAGS, SWP_NOACTIVATE,
    ULW_ALPHA, WNDCLASSEXW, WS_EX_LAYERED, WS_EX_TOPMOST, WS_POPUP,
};

pub struct SafeHWND(pub(crate) HWND);

/// Trait for handling Win32 window messages.
pub trait WindowEventHandler {
    fn on_message(
        &mut self,
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> Option<LRESULT>;
}

/// A thin wrapper to store a trait object in GWLP_USERDATA.
/// This avoids the "fat pointer" storage problem in 64-bit pointers.
struct Dispatcher {
    handler: *mut dyn WindowEventHandler,
}

pub unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let ptr = windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
        hwnd,
        windows::Win32::UI::WindowsAndMessaging::GWLP_USERDATA,
    );

    if ptr != 0 {
        let dispatcher = &mut *(ptr as *mut Dispatcher);

        // Handle cleanup
        if msg == windows::Win32::UI::WindowsAndMessaging::WM_NCDESTROY {
            let _ = Box::from_raw(dispatcher); // Take ownership and drop
            windows::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW(
                hwnd,
                windows::Win32::UI::WindowsAndMessaging::GWLP_USERDATA,
                0,
            );
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
        windows::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW(
            hwnd,
            windows::Win32::UI::WindowsAndMessaging::GWLP_USERDATA,
            Box::into_raw(dispatcher) as isize,
        );
    }
}

pub fn hide_window(hwnd: &SafeHWND) {
    unsafe {
        let _ = windows::Win32::UI::WindowsAndMessaging::ShowWindow(
            hwnd.0,
            windows::Win32::UI::WindowsAndMessaging::SW_HIDE,
        );
    }
}

pub fn create_overlay_window(class_name: &str, title: &str) -> anyhow::Result<SafeHWND> {
    unsafe {
        let h_instance = windows::Win32::System::LibraryLoader::GetModuleHandleW(None)?;

        let class_name_u16: Vec<u16> = class_name.encode_utf16().chain(Some(0)).collect();
        let title_u16: Vec<u16> = title.encode_utf16().chain(Some(0)).collect();

        let wnd_class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW | CS_DBLCLKS,
            lpfnWndProc: Some(wnd_proc),
            hInstance: h_instance.into(),
            lpszClassName: windows::core::PCWSTR(class_name_u16.as_ptr()),
            ..Default::default()
        };

        RegisterClassExW(&wnd_class);

        let hwnd = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOPMOST,
            windows::core::PCWSTR(class_name_u16.as_ptr()),
            windows::core::PCWSTR(title_u16.as_ptr()),
            WS_POPUP,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            1, // Initial width
            1, // Initial height
            None,
            None,
            Some(h_instance.into()),
            None,
        )?;

        if hwnd.is_invalid() {
            anyhow::bail!("Failed to create window");
        }

        Ok(SafeHWND(hwnd))
    }
}

pub fn set_window_pos(
    hwnd: &SafeHWND,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    flags: SET_WINDOW_POS_FLAGS,
) -> anyhow::Result<()> {
    unsafe {
        SetWindowPos(hwnd.0, None, x, y, w, h, flags | SWP_NOACTIVATE)?;
        Ok(())
    }
}

pub fn update_layered_window(
    hwnd: &SafeHWND,
    hdc_src: &SafeHDC,
    point: &POINT,
    size: &SIZE,
    alpha: u8,
    alpha_format: u8,
) -> anyhow::Result<()> {
    unsafe {
        let blend = BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            BlendFlags: 0,
            SourceConstantAlpha: alpha,
            AlphaFormat: alpha_format,
        };

        UpdateLayeredWindow(
            hwnd.0,
            None,
            Some(point),
            Some(size),
            Some(hdc_src.0),
            Some(&POINT { x: 0, y: 0 }),
            COLORREF(0),
            Some(&blend),
            ULW_ALPHA,
        )?;
        Ok(())
    }
}
pub fn show_window(
    hwnd: &SafeHWND,
    cmd_show: windows::Win32::UI::WindowsAndMessaging::SHOW_WINDOW_CMD,
) -> bool {
    unsafe { windows::Win32::UI::WindowsAndMessaging::ShowWindow(hwnd.0, cmd_show).into() }
}

pub fn set_system_cursor(id: windows::core::PCWSTR) -> anyhow::Result<()> {
    unsafe {
        let h_cursor = windows::Win32::UI::WindowsAndMessaging::LoadCursorW(None, id)?;
        windows::Win32::UI::WindowsAndMessaging::SetCursor(Some(h_cursor));
        Ok(())
    }
}

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};
    use windows::Win32::UI::WindowsAndMessaging::{GetWindowRect, IsIconic, IsWindowVisible};

    if IsWindowVisible(hwnd).as_bool() && !IsIconic(hwnd).as_bool() {
        let mut rect = windows::Win32::Foundation::RECT::default();
        let mut success = false;

        // 1. Try DWM Extended Frame Bounds (Actual Visual Bounds, excluding shadow)
        let ptr = &mut rect as *mut _ as *mut std::ffi::c_void;
        let size = std::mem::size_of::<windows::Win32::Foundation::RECT>() as u32;
        if DwmGetWindowAttribute(hwnd, DWMWA_EXTENDED_FRAME_BOUNDS, ptr, size).is_ok() {
            success = true;
        }
        // 2. Fallback for non-DWM windows
        else if GetWindowRect(hwnd, &mut rect).is_ok() {
            success = true;
        }

        if success {
            let w = rect.right - rect.left;
            let h = rect.bottom - rect.top;
            if w > 10 && h > 10 {
                let vec_ptr = lparam.0 as *mut Vec<windows::Win32::Foundation::RECT>;
                (&mut *vec_ptr).push(rect);
            }
        }
    }
    BOOL(1)
}

pub fn enumerate_visible_windows() -> Vec<windows::Win32::Foundation::RECT> {
    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::EnumWindows;
        let mut rects = Vec::with_capacity(128);
        let ptr = &mut rects as *mut Vec<windows::Win32::Foundation::RECT> as isize;
        let _ = EnumWindows(Some(enum_windows_proc), LPARAM(ptr));
        rects
    }
}
