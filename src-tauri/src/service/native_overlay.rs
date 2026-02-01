use std::ffi::c_void;
use std::sync::atomic::{AtomicIsize, Ordering};
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, SIZE, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetStockObject, SelectObject,
    AC_SRC_ALPHA, AC_SRC_OVER, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, BLACK_BRUSH, BLENDFUNCTION,
    DIB_RGB_COLORS, HBRUSH,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, GetSystemMetrics, RegisterClassExW, ShowWindow,
    UpdateLayeredWindow, CS_HREDRAW, CS_VREDRAW, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN,
    SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, SW_HIDE, SW_SHOWNOACTIVATE, ULW_ALPHA, WINDOW_EX_STYLE,
    WNDCLASSEXW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_POPUP,
};

// Start ID 1718
// ... (imports are handled above)

static NATIVE_OVERLAY_HWND: AtomicIsize = AtomicIsize::new(0);

// WS_EX_TRANSPARENT = 0x20 - makes window click-through
// const WS_EX_CLICK_THROUGH: WINDOW_EX_STYLE = WINDOW_EX_STYLE(0x20);

// We need CLICK_THROUGH?
// Actually, if we want the USER to select on WebView, the user clicks WebView.
// This NativeOverlay is UNDER WebView. So it doesn't matter if it catches clicks or not,
// because WebView covers it.
// However, to be safe, let's keep it transparent to mouse for now, just incase WebView has gaps?
// No, WebView is full screen.
// But wait, if native overlay is TOPMOST and WebView is also TOPMOST?
// We must ensure Z-order.
// For now, keep existing styles.

const CLASS_NAME: PCWSTR = w!("HyperLensNativeOverlay");

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

/// Initialize the native overlay window (call once at startup)
pub fn init() -> Result<(), String> {
    unsafe {
        let hmodule = GetModuleHandleW(None).map_err(|e| e.to_string())?;
        let hinstance = windows::Win32::Foundation::HINSTANCE(hmodule.0);

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            hInstance: hinstance,
            hbrBackground: HBRUSH(GetStockObject(BLACK_BRUSH).0),
            lpszClassName: CLASS_NAME,
            ..Default::default()
        };

        RegisterClassExW(&wc);

        // Get virtual screen bounds (covers all monitors)
        let x = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let y = GetSystemMetrics(SM_YVIRTUALSCREEN);
        let width = GetSystemMetrics(SM_CXVIRTUALSCREEN);
        let height = GetSystemMetrics(SM_CYVIRTUALSCREEN);

        crate::service::logger::log(
            "NativeOverlay",
            &format!("Creating at ({},{}) size {}x{}", x, y, width, height),
        );

        // WS_EX_TRANSPARENT (0x20) makes it click-through.
        // WS_EX_LAYERED required for transparency.
        let hwnd = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WINDOW_EX_STYLE(0x20), // WS_EX_TRANSPARENT
            CLASS_NAME,
            w!("HyperLens Overlay"),
            WS_POPUP,
            x,
            y,
            width,
            height,
            None,
            None,
            Some(&hinstance),
            None,
        )
        .map_err(|e| e.to_string())?;

        // Initial: Set 40% opacity black - REMOVED because it conflicts with UpdateLayeredWindow

        NATIVE_OVERLAY_HWND.store(hwnd.0 as isize, Ordering::SeqCst);

        crate::service::logger::log(
            "NativeOverlay",
            &format!("Initialized: pos=({}, {}), size={}x{}", x, y, width, height),
        );

        Ok(())
    }
}

/// Updates the overlay with a new image buffer (RGBA), applies dimming, and renders via GDI.
/// This bypasses WebView rendering for zero-latency display.
pub fn update_with_buffer(width: u32, height: u32, rgba_pixels: &[u8]) {
    let hwnd_val = NATIVE_OVERLAY_HWND.load(Ordering::SeqCst);
    if hwnd_val == 0 {
        return;
    }
    let hwnd = HWND(hwnd_val as *mut _);

    unsafe {
        let hdc_screen = windows::Win32::Graphics::Gdi::GetDC(None);
        let hdc_mem = CreateCompatibleDC(hdc_screen);

        // Define Bitmap Header
        // Note: Height is negative to create a top-down DIB
        let mut bi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width as i32,
                biHeight: -(height as i32),
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut p_bits: *mut c_void = std::ptr::null_mut();
        let hbitmap = CreateDIBSection(hdc_mem, &bi, DIB_RGB_COLORS, &mut p_bits, None, 0);

        if hbitmap.is_err() || p_bits.is_null() {
            crate::service::logger::log("NativeOverlay", "Failed to create DIB section");
            DeleteDC(hdc_mem);
            windows::Win32::Graphics::Gdi::ReleaseDC(None, hdc_screen);
            return;
        }
        let hbitmap = hbitmap.unwrap();

        // Pixel Processing (Directly into DIB memory)
        // 1. Copy pixels
        // 2. Swizzle RGBA -> BGRA (Windows GDI expects BGRA)
        // 3. Apply Dimming (Multiply by 0.6)
        // 4. Set Alpha to 255 (Opaque) - because we want the image itself to be solid,
        //    but maybe slightly transparent?
        //    User wants "Frozen Screen", which is solid. But usually "Screenshot" implies dragging over a static image.
        //    We want it to look like the screen is frozen and dimmed.
        //    If we just draw the image opaque, it's bright.
        //    So we dim the RGB values.

        let dest_slice =
            std::slice::from_raw_parts_mut(p_bits as *mut u8, (width * height * 4) as usize);
        let src_len = rgba_pixels.len();

        // Simd optimizable, but simple loop for now
        // Assuming rgba_pixels matches w*h*4
        let len = std::cmp::min(dest_slice.len(), src_len);

        for i in (0..len).step_by(4) {
            let r = rgba_pixels[i];
            let g = rgba_pixels[i + 1];
            let b = rgba_pixels[i + 2];
            // let a = rgba_pixels[i+3]; // Capture is typically 255

            // BGRA Swizzle (No Dimming)
            // B
            dest_slice[i] = b;
            // G
            dest_slice[i + 1] = g;
            // R
            dest_slice[i + 2] = r;
            // A
            dest_slice[i + 3] = 255;
        }

        let old_bitmap = SelectObject(hdc_mem, hbitmap);

        // Update Layered Window
        let point = POINT {
            x: GetSystemMetrics(SM_XVIRTUALSCREEN),
            y: GetSystemMetrics(SM_YVIRTUALSCREEN),
        };
        let size = SIZE {
            cx: width as i32,
            cy: height as i32,
        };
        let pt_src = POINT { x: 0, y: 0 };

        let blend = BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            BlendFlags: 0,
            SourceConstantAlpha: 255, // 255 = Use per-pixel alpha (which is 255). Window is opaque.
            AlphaFormat: AC_SRC_ALPHA as u8,
        };

        // Note: UpdateLayeredWindow might fail if window was set with LWA_ALPHA via SetLayeredWindowAttributes?
        // But usually it overrides.
        let result = UpdateLayeredWindow(
            hwnd,
            hdc_screen,
            Some(&point),
            Some(&size),
            hdc_mem,
            Some(&pt_src),
            COLORREF(0),
            Some(&blend),
            ULW_ALPHA, // Use blend function
        );

        if let Err(e) = result {
            crate::service::logger::log(
                "NativeOverlay",
                &format!("UpdateLayeredWindow failed: {}", e),
            );
        } else {
            crate::service::logger::log("NativeOverlay", "Bitmap updated successfully");
        }

        // Cleanup
        SelectObject(hdc_mem, old_bitmap);
        DeleteObject(hbitmap);
        DeleteDC(hdc_mem);
        windows::Win32::Graphics::Gdi::ReleaseDC(None, hdc_screen);
    }
}

/// Show the native overlay instantly
pub fn show() {
    let hwnd_val = NATIVE_OVERLAY_HWND.load(Ordering::SeqCst);
    if hwnd_val != 0 {
        unsafe {
            let hwnd = HWND(hwnd_val as *mut _);
            // SW_SHOWNOACTIVATE ensures it doesn't steal focus
            let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        }
        crate::service::logger::log("NativeOverlay", "Shown");
    }
}

/// Hide the native overlay
pub fn hide() {
    let hwnd_val = NATIVE_OVERLAY_HWND.load(Ordering::SeqCst);
    if hwnd_val != 0 {
        unsafe {
            let hwnd = HWND(hwnd_val as *mut _);
            let _ = ShowWindow(hwnd, SW_HIDE);
        }
        crate::service::logger::log("NativeOverlay", "Hidden");
    }
}

/// Get the HWND for Z-order management
pub fn get_hwnd() -> Option<HWND> {
    let hwnd_val = NATIVE_OVERLAY_HWND.load(Ordering::SeqCst);
    if hwnd_val != 0 {
        Some(HWND(hwnd_val as *mut _))
    } else {
        None
    }
}

/// Cleanup on exit
pub fn destroy() {
    let hwnd_val = NATIVE_OVERLAY_HWND.load(Ordering::SeqCst);
    if hwnd_val != 0 {
        unsafe {
            let hwnd = HWND(hwnd_val as *mut _);
            let _ = DestroyWindow(hwnd);
        }
        NATIVE_OVERLAY_HWND.store(0, Ordering::SeqCst);
    }
}
