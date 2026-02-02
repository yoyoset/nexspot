use super::render::redraw;
use super::state::{emit_event, STATE};
use std::sync::atomic::{AtomicIsize, Ordering};
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::Graphics::Gdi::{GetStockObject, ScreenToClient, BLACK_BRUSH, HBRUSH};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    ReleaseCapture, SetCapture, VIRTUAL_KEY, VK_ESCAPE,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, GetCursorPos, GetSystemMetrics, GetWindowLongW,
    LoadCursorW, RegisterClassExW, SetCursor, SetForegroundWindow, SetWindowLongW, ShowWindow,
    CS_DBLCLKS, CS_HREDRAW, CS_VREDRAW, GWL_EXSTYLE, IDC_CROSS, IDC_SIZEALL, IDC_SIZENESW,
    IDC_SIZENS, IDC_SIZENWSE, IDC_SIZEWE, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN,
    SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, SW_SHOWNOACTIVATE, WM_KEYDOWN, WM_LBUTTONDBLCLK,
    WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_SETCURSOR, WNDCLASSEXW, WS_EX_LAYERED,
    WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
};

static NATIVE_OVERLAY_HWND: AtomicIsize = AtomicIsize::new(0);
const CLASS_NAME: PCWSTR = w!("HyperLensNativeOverlay");

enum Action {
    None,
    Save(String),
    Cancel,
    Plugin(String),
}

pub fn get_hwnd() -> Option<HWND> {
    let hwnd_val = NATIVE_OVERLAY_HWND.load(Ordering::SeqCst);
    if hwnd_val != 0 {
        Some(HWND(hwnd_val as *mut _))
    } else {
        None
    }
}

pub fn init() -> Result<(), String> {
    unsafe {
        use windows::Win32::UI::HiDpi::{
            SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
        };
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);

        let hmodule = GetModuleHandleW(None).map_err(|e| e.to_string())?;
        let hinstance = windows::Win32::Foundation::HINSTANCE(hmodule.0);

        let cursor = LoadCursorW(None, IDC_CROSS).map_err(|e| e.to_string())?;

        let bg_brush_handle = GetStockObject(BLACK_BRUSH);
        // HGDIOBJ from GetStockObject is usually valid, but strict check:
        if bg_brush_handle.is_invalid() {
            return Err("Failed to get stock object".to_string());
        }

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW | CS_DBLCLKS,
            lpfnWndProc: Some(window_proc),
            hInstance: hinstance,
            hbrBackground: HBRUSH(bg_brush_handle.0),
            lpszClassName: CLASS_NAME,
            hCursor: cursor,
            ..Default::default()
        };

        let atom = RegisterClassExW(&wc);
        if atom == 0 {
            return Err("Failed to register class".to_string());
        }

        let x = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let y = GetSystemMetrics(SM_YVIRTUALSCREEN);
        let width = GetSystemMetrics(SM_CXVIRTUALSCREEN);
        let height = GetSystemMetrics(SM_CYVIRTUALSCREEN);

        let hwnd = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TOPMOST,
            CLASS_NAME,
            w!("HyperLens Overlay"),
            WS_POPUP,
            x,
            y,
            width,
            height,
            None,
            None,
            Some(hinstance),
            None,
        )
        .map_err(|e| e.to_string())?;

        NATIVE_OVERLAY_HWND.store(hwnd.0 as isize, Ordering::SeqCst);
        Ok(())
    }
}

pub fn show() {
    let hwnd_val = NATIVE_OVERLAY_HWND.load(Ordering::SeqCst);
    if hwnd_val != 0 {
        unsafe {
            let hwnd = HWND(hwnd_val as *mut _);
            let _ = SetForegroundWindow(hwnd);
            let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        }
    }
}

pub fn hide() {
    let hwnd_val = NATIVE_OVERLAY_HWND.load(Ordering::SeqCst);
    if hwnd_val != 0 {
        unsafe {
            let hwnd = HWND(hwnd_val as *mut _);
            let _ = ReleaseCapture();

            use windows::Win32::UI::WindowsAndMessaging::{
                SetWindowPos, SWP_HIDEWINDOW, SWP_NOMOVE, SWP_NOREDRAW, SWP_NOSIZE, SWP_NOZORDER,
            };
            let _ = SetWindowPos(
                hwnd,
                HWND(std::ptr::null_mut()),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_HIDEWINDOW | SWP_NOREDRAW,
            );
        }
        if let Ok(mut guard) = STATE.lock() {
            if let Some(state) = guard.as_mut() {
                state.interaction_state = super::state::InteractionState::Idle;
                state.final_rect = None;
            }
        }
    }
}

pub fn destroy() {
    let hwnd_val = NATIVE_OVERLAY_HWND.load(Ordering::SeqCst);
    if hwnd_val != 0 {
        unsafe {
            let _ = ReleaseCapture();
            let _ = DestroyWindow(HWND(hwnd_val as *mut _));
        }
        NATIVE_OVERLAY_HWND.store(0, Ordering::SeqCst);
    }
    if let Ok(mut guard) = STATE.lock() {
        if let Some(state) = guard.take() {
            let _ = state;
        }
    }
}

pub fn set_input_passthrough(enable: bool) {
    let hwnd_val = NATIVE_OVERLAY_HWND.load(Ordering::SeqCst);
    if hwnd_val != 0 {
        unsafe {
            let hwnd = HWND(hwnd_val as *mut _);
            let style = GetWindowLongW(hwnd, GWL_EXSTYLE);
            let new_style = if enable {
                style | WS_EX_TRANSPARENT.0 as i32
            } else {
                style & !WS_EX_TRANSPARENT.0 as i32
            };

            if style != new_style {
                let _ = SetWindowLongW(hwnd, GWL_EXSTYLE, new_style);
                use windows::Win32::UI::WindowsAndMessaging::{
                    SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOREDRAW, SWP_NOSIZE, SWP_NOZORDER,
                };
                let _ = windows::Win32::UI::WindowsAndMessaging::SetWindowPos(
                    hwnd,
                    HWND(std::ptr::null_mut()),
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED | SWP_NOREDRAW,
                );
            }
        }
    }
}

pub fn set_topmost(enable: bool) {
    let hwnd_val = NATIVE_OVERLAY_HWND.load(Ordering::SeqCst);
    if hwnd_val != 0 {
        unsafe {
            let hwnd = HWND(hwnd_val as *mut _);
            let hwnd_insert_after = if enable {
                windows::Win32::UI::WindowsAndMessaging::HWND_TOPMOST
            } else {
                windows::Win32::UI::WindowsAndMessaging::HWND_NOTOPMOST
            };
            let _ = windows::Win32::UI::WindowsAndMessaging::SetWindowPos(
                hwnd,
                hwnd_insert_after,
                0,
                0,
                0,
                0,
                windows::Win32::UI::WindowsAndMessaging::SWP_NOMOVE
                    | windows::Win32::UI::WindowsAndMessaging::SWP_NOSIZE
                    | windows::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE,
            );
        }
    }
}

fn get_normalized_rect(p1: POINT, p2: POINT) -> (i32, i32, i32, i32) {
    let left = std::cmp::min(p1.x, p2.x);
    let top = std::cmp::min(p1.y, p2.y);
    let width = (p1.x - p2.x).abs();
    let height = (p1.y - p2.y).abs();
    (left, top, width, height)
}

fn hit_test(x: i32, y: i32, rect: Option<windows::Win32::Foundation::RECT>) -> i32 {
    if let Some(r) = rect {
        let handle_size = 20;
        let half_h = handle_size / 2;
        let anchors = [
            (r.left, r.top),
            (r.right, r.top),
            (r.right, r.bottom),
            (r.left, r.bottom),
            ((r.left + r.right) / 2, r.top),
            (r.right, (r.top + r.bottom) / 2),
            ((r.left + r.right) / 2, r.bottom),
            (r.left, (r.top + r.bottom) / 2),
        ];
        for (i, &(ax, ay)) in anchors.iter().enumerate() {
            if x >= ax - half_h && x <= ax + half_h && y >= ay - half_h && y <= ay + half_h {
                return i as i32;
            }
        }
        if x >= r.left && x <= r.right && y >= r.top && y <= r.bottom {
            return 8;
        }
    }
    -1
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    use super::state::InteractionState;

    match msg {
        WM_SETCURSOR => {
            if let Ok(guard) = STATE.lock() {
                if let Some(state) = guard.as_ref() {
                    let mut cursor_name = IDC_CROSS;
                    if let Some(rect) = state.final_rect {
                        let mut pt = POINT::default();
                        let _ = GetCursorPos(&mut pt);
                        let _ = ScreenToClient(Some(hwnd), &mut pt);
                        let hit = hit_test(pt.x, pt.y, Some(rect));
                        cursor_name = match hit {
                            0 | 2 => IDC_SIZENWSE,
                            1 | 3 => IDC_SIZENESW,
                            4 | 6 => IDC_SIZENS,
                            5 | 7 => IDC_SIZEWE,
                            8 => IDC_SIZEALL,
                            _ => IDC_CROSS,
                        };
                    }
                    if let Ok(cursor) = LoadCursorW(None, cursor_name) {
                        SetCursor(cursor);
                    }
                }
            }
            LRESULT(1)
        }
        WM_KEYDOWN => {
            let key = VIRTUAL_KEY(wparam.0 as u16);
            if key == VK_ESCAPE {
                emit_event("selection-cancel", "{}");
                hide();
            }
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
            let _ = SetCapture(hwnd);
            if let Ok(mut guard) = STATE.lock() {
                if let Some(state) = guard.as_mut() {
                    match state.interaction_state {
                        InteractionState::Idle => {
                            state.interaction_state = InteractionState::Creating;
                            state.start_pos = POINT { x, y };
                            state.current_pos = POINT { x, y };
                            state.final_rect = None;
                        }
                        InteractionState::Selected => {
                            let hit = hit_test(x, y, state.final_rect);
                            let tb_hit = state.toolbar.hit_test(x, y);
                            if tb_hit.is_some() {
                                // Toolbar logic handled in UP
                            } else if (0..=7).contains(&hit) {
                                state.interaction_state = InteractionState::Resizing(hit as usize);
                                state.start_pos = POINT { x, y };
                            } else if hit == 8 {
                                state.interaction_state = InteractionState::Moving;
                                state.start_pos = POINT { x, y };
                            } else {
                                state.interaction_state = InteractionState::Creating;
                                state.start_pos = POINT { x, y };
                                state.current_pos = POINT { x, y };
                                state.final_rect = None;
                            }
                        }
                        _ => {}
                    }
                    redraw(hwnd, state);
                }
            }
            LRESULT(0)
        }
        WM_LBUTTONDBLCLK => {
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
            let mut action = Action::None;
            if let Ok(mut guard) = STATE.lock() {
                if let Some(state) = guard.as_mut() {
                    if state.interaction_state == InteractionState::Selected {
                        let hit = hit_test(x, y, state.final_rect);
                        if hit == 8 {
                            if let Some(r) = state.final_rect {
                                let (l, t, w, h) = get_normalized_rect(
                                    POINT {
                                        x: r.left,
                                        y: r.top,
                                    },
                                    POINT {
                                        x: r.right,
                                        y: r.bottom,
                                    },
                                );
                                action = Action::Save(format!(
                                    r#"{{ "x": {}, "y": {}, "width": {}, "height": {} }}"#,
                                    l, t, w, h
                                ));
                            }
                        }
                    }
                }
            }
            match action {
                Action::Save(payload) => {
                    hide();
                    emit_event("capture-save", &payload);
                }
                _ => {}
            }
            LRESULT(0)
        }
        WM_MOUSEMOVE => {
            if let Ok(mut guard) = STATE.lock() {
                if let Some(state) = guard.as_mut() {
                    let x = (lparam.0 & 0xFFFF) as i16 as i32;
                    let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
                    match state.interaction_state {
                        InteractionState::Creating => {
                            state.current_pos = POINT { x, y };
                            redraw(hwnd, state);
                        }
                        InteractionState::Moving => {
                            if let Some(mut r) = state.final_rect {
                                let dx = x - state.start_pos.x;
                                let dy = y - state.start_pos.y;
                                r.left += dx;
                                r.top += dy;
                                r.right += dx;
                                r.bottom += dy;
                                state.final_rect = Some(r);
                                state.start_pos = POINT { x, y };
                                state.toolbar.update_layout(r, state.width, state.height);
                            }
                            redraw(hwnd, state);
                        }
                        InteractionState::Resizing(idx) => {
                            if let Some(mut r) = state.final_rect {
                                match idx {
                                    0 => {
                                        r.left = x;
                                        r.top = y;
                                    }
                                    1 => {
                                        r.right = x;
                                        r.top = y;
                                    }
                                    2 => {
                                        r.right = x;
                                        r.bottom = y;
                                    }
                                    3 => {
                                        r.left = x;
                                        r.bottom = y;
                                    }
                                    4 => {
                                        r.top = y;
                                    }
                                    5 => {
                                        r.right = x;
                                    }
                                    6 => {
                                        r.bottom = y;
                                    }
                                    7 => {
                                        r.left = x;
                                    }
                                    _ => {}
                                }
                                state.final_rect = Some(r);
                                state.toolbar.update_layout(r, state.width, state.height);
                            }
                            redraw(hwnd, state);
                        }
                        InteractionState::Selected => {
                            if state.toolbar.on_mouse_move(x, y) {
                                redraw(hwnd, state);
                            }
                        }
                        _ => {}
                    }
                }
            }
            LRESULT(0)
        }
        WM_LBUTTONUP => {
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
            let _ = ReleaseCapture();
            let mut action = Action::None;
            if let Ok(mut guard) = STATE.lock() {
                if let Some(state) = guard.as_mut() {
                    if state.interaction_state == InteractionState::Creating {
                        state.current_pos = POINT { x, y };
                        let (l, t, w, h) = get_normalized_rect(state.start_pos, state.current_pos);
                        if w > 5 && h > 5 {
                            let r = windows::Win32::Foundation::RECT {
                                left: l,
                                top: t,
                                right: l + w,
                                bottom: t + h,
                            };
                            state.final_rect = Some(r);
                            state.interaction_state = InteractionState::Selected;
                            state.toolbar.update_layout(r, state.width, state.height);
                        } else {
                            state.interaction_state = InteractionState::Idle;
                            state.final_rect = None;
                        }
                    } else if state.interaction_state == InteractionState::Moving {
                        state.interaction_state = InteractionState::Selected;
                    } else if let InteractionState::Resizing(_) = state.interaction_state {
                        state.interaction_state = InteractionState::Selected;
                        if let Some(r) = state.final_rect {
                            let (l, t, w, h) = get_normalized_rect(
                                POINT {
                                    x: r.left,
                                    y: r.top,
                                },
                                POINT {
                                    x: r.right,
                                    y: r.bottom,
                                },
                            );
                            state.final_rect = Some(windows::Win32::Foundation::RECT {
                                left: l,
                                top: t,
                                right: l + w,
                                bottom: t + h,
                            });
                        }
                    } else if state.interaction_state == InteractionState::Selected {
                        if let Some(r) = state.final_rect {
                            state.toolbar.update_layout(r, state.width, state.height);
                        }
                        let hit = state.toolbar.hit_test(x, y);
                        if let Some(cmd) = hit {
                            match cmd {
                                super::toolbar::ToolbarCommand::Save => {
                                    if let Some(r) = state.final_rect {
                                        let (l, t, w, h) = get_normalized_rect(
                                            POINT {
                                                x: r.left,
                                                y: r.top,
                                            },
                                            POINT {
                                                x: r.right,
                                                y: r.bottom,
                                            },
                                        );
                                        action = Action::Save(format!(
                                            r#"{{ "x": {}, "y": {}, "width": {}, "height": {} }}"#,
                                            l, t, w, h
                                        ));
                                    }
                                }
                                super::toolbar::ToolbarCommand::Cancel => {
                                    action = Action::Cancel;
                                }
                                super::toolbar::ToolbarCommand::Plugin(id) => {
                                    action = Action::Plugin(id);
                                }
                            }
                        }
                    }
                    redraw(hwnd, state);
                }
            }
            match action {
                Action::Save(payload) => {
                    hide();
                    emit_event("capture-save", &payload);
                }
                Action::Cancel => {
                    hide();
                    emit_event("selection-cancel", "{}");
                }
                Action::Plugin(id) => {
                    emit_event("plugin-trigger", &id);
                }
                Action::None => {}
            }
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
