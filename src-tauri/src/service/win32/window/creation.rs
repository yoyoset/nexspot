use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::PCWSTR;
use super::types::SafeHWND;
use super::wnd_proc::wnd_proc;

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
            lpszClassName: PCWSTR(class_name_u16.as_ptr()),
            ..Default::default()
        };

        RegisterClassExW(&wnd_class);

        let hwnd = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOPMOST,
            PCWSTR(class_name_u16.as_ptr()),
            PCWSTR(title_u16.as_ptr()),
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

pub fn destroy_window(hwnd: &SafeHWND) {
    unsafe {
        let _ = windows::Win32::UI::WindowsAndMessaging::DestroyWindow(hwnd.0);
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

pub fn show_window(
    hwnd: &SafeHWND,
    cmd_show: SHOW_WINDOW_CMD,
) -> bool {
    unsafe { windows::Win32::UI::WindowsAndMessaging::ShowWindow(hwnd.0, cmd_show).into() }
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

pub fn set_system_cursor(id: PCWSTR) -> anyhow::Result<()> {
    unsafe {
        let h_cursor = LoadCursorW(None, id)?;
        SetCursor(Some(h_cursor));
        Ok(())
    }
}
