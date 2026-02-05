use crate::service::native_overlay::state::OverlayState;
use crate::service::win32;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

/// Capture the current selection from the bright bitmap and save it to a file asynchronously.
pub fn save_selection(state_arc: &Arc<Mutex<OverlayState>>, app: &AppHandle) -> anyhow::Result<()> {
    let state = state_arc.lock().unwrap();
    let sel = match state.selection {
        Some(s) => s,
        None => return Ok(()),
    };

    if let Some(hbm_bright) = &state.hbitmap_bright {
        let width = sel.right - sel.left;
        let height = sel.bottom - sel.top;

        if width <= 0 || height <= 0 {
            return Ok(());
        }

        // GDI: Crop
        let hdc_screen = win32::gdi::get_dc(None)?;
        let hdc_mem_src = win32::gdi::create_compatible_dc(Some(&hdc_screen))?;
        let prev_src = win32::gdi::select_object(
            &hdc_mem_src,
            windows::Win32::Graphics::Gdi::HGDIOBJ(hbm_bright.0 .0),
        )?;

        let hdc_mem_dst = win32::gdi::create_compatible_dc(Some(&hdc_screen))?;
        let hbm_crop = win32::gdi::create_compatible_bitmap(&hdc_screen, width, height)?;
        let prev_dst = win32::gdi::select_object(
            &hdc_mem_dst,
            windows::Win32::Graphics::Gdi::HGDIOBJ(hbm_crop.0 .0),
        )?;

        win32::gdi::bit_blt(
            &hdc_mem_dst,
            0,
            0,
            width,
            height,
            &hdc_mem_src,
            sel.left,
            sel.top,
            windows::Win32::Graphics::Gdi::SRCCOPY,
        )?;

        // Restore & Clean
        win32::gdi::select_object(&hdc_mem_dst, prev_dst)?;
        win32::gdi::select_object(&hdc_mem_src, prev_src)?;
        // hdc_mem_dst, hdc_mem_src, hdc_screen drop here. hbm_crop (SafeHBITMAP) remains.

        // Transfer ownership to worker
        let raw_hbm = hbm_crop.0;
        std::mem::forget(hbm_crop);
        // Pass as usize to bypass Send check (it's a raw handle)
        let hbm_ptr = raw_hbm.0 as usize;

        let app_handle = app.clone();

        tauri::async_runtime::spawn_blocking(move || {
            // Reconstruct HBITMAP
            let raw_hbm = windows::Win32::Graphics::Gdi::HBITMAP(hbm_ptr as *mut std::ffi::c_void);
            // Re-wrap to ensure Drop calls DeleteObject
            let _safe_hbm = crate::service::win32::gdi::SafeHBITMAP(raw_hbm);

            let captures_dir = {
                let state = app_handle.state::<crate::app_state::AppState>();
                let config_state = state.config_state.lock().unwrap();
                config_state.resolve_save_path(&app_handle)
            };

            if !captures_dir.exists() {
                let _ = std::fs::create_dir_all(&captures_dir);
            }

            let now = chrono::Local::now();
            let filename = now.format("%Y-%m-%d_%H-%M-%S.png").to_string();
            let file_path = captures_dir.join(&filename);

            // Save
            if let Err(e) = crate::service::win32::bitmap::save_bitmap_to_file(raw_hbm, &file_path)
            {
                log::error!("Failed to save: {:?}", e);
                let _ = app_handle
                    .notification()
                    .builder()
                    .title("Save Failed")
                    .body(&format!("Error: {}", e))
                    .show();
                return;
            }

            log::info!("Saved crop to {:?}", file_path);

            let _ = app_handle
                .notification()
                .builder()
                .title("Screenshot Saved")
                .body(&format!("Saved to {}", file_path.display()))
                .show();

            use tauri::Emitter;
            let _ = app_handle.emit("screenshot-saved", ());
        });
    }

    Ok(())
}
// ... existing imports ...
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
};

/// Capture and Copy to Clipboard
pub fn copy_to_clipboard(
    state_arc: &Arc<Mutex<OverlayState>>,
    app: &AppHandle,
) -> anyhow::Result<()> {
    let state = state_arc.lock().unwrap();
    let sel = match state.selection {
        Some(s) => s,
        None => return Ok(()),
    };

    if let Some(hbm_bright) = &state.hbitmap_bright {
        let width = sel.right - sel.left;
        let height = sel.bottom - sel.top;

        if width <= 0 || height <= 0 {
            return Ok(());
        }

        // GDI: Crop
        let hdc_screen = win32::gdi::get_dc(None)?;
        let hdc_mem_src = win32::gdi::create_compatible_dc(Some(&hdc_screen))?;
        let prev_src = win32::gdi::select_object(
            &hdc_mem_src,
            windows::Win32::Graphics::Gdi::HGDIOBJ(hbm_bright.0 .0),
        )?;

        let hdc_mem_dst = win32::gdi::create_compatible_dc(Some(&hdc_screen))?;
        let hbm_crop = win32::gdi::create_compatible_bitmap(&hdc_screen, width, height)?;
        let prev_dst = win32::gdi::select_object(
            &hdc_mem_dst,
            windows::Win32::Graphics::Gdi::HGDIOBJ(hbm_crop.0 .0),
        )?;

        win32::gdi::bit_blt(
            &hdc_mem_dst,
            0,
            0,
            width,
            height,
            &hdc_mem_src,
            sel.left,
            sel.top,
            windows::Win32::Graphics::Gdi::SRCCOPY,
        )?;

        // Restore & Clean
        win32::gdi::select_object(&hdc_mem_dst, prev_dst)?;
        win32::gdi::select_object(&hdc_mem_src, prev_src)?;

        // Clipboard
        unsafe {
            if OpenClipboard(None).is_ok() {
                let _ = EmptyClipboard();
                // CF_BITMAP = 2
                let _ = SetClipboardData(2, Some(HANDLE(hbm_crop.0 .0 as *mut std::ffi::c_void)));
                let _ = CloseClipboard();
                // System now owns the HBITMAP, do not drop it.
                std::mem::forget(hbm_crop);
            }
        }

        let _ = app
            .notification()
            .builder()
            .title("Copied")
            .body("Image copied to clipboard")
            .show();

        use tauri::Emitter;
        let _ = app.emit("screenshot-copied", ());
    }

    Ok(())
}
