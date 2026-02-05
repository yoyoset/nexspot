use crate::service::native_overlay::state::OverlayState;
use crate::service::ocr;
use crate::service::win32;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager};

/// Performs OCR on the current selection.
pub fn perform_ocr(state_arc: &Arc<Mutex<OverlayState>>, app: &AppHandle) -> anyhow::Result<()> {
    let (hbm_crop, engine_id) = {
        let state = state_arc.lock().unwrap();
        let sel = match state.selection {
            Some(s) => s,
            None => return Ok(()),
        };

        let hbm_bright = match &state.hbitmap_bright {
            Some(h) => h,
            None => return Ok(()),
        };

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

        win32::gdi::select_object(&hdc_mem_dst, prev_dst)?;
        win32::gdi::select_object(&hdc_mem_src, prev_src)?;

        let app_state = app.state::<crate::app_state::AppState>();
        let config_state = app_state.config_state.lock().unwrap();
        let engine_id = config_state.config.ocr_engine.clone();

        (hbm_crop, engine_id)
    };

    // Transfer ownership to worker
    let raw_hbm = hbm_crop.0;
    std::mem::forget(hbm_crop);
    let hbm_ptr = raw_hbm.0 as usize;
    let app_handle = app.clone();

    // Process in background
    tauri::async_runtime::spawn_blocking(move || {
        // Reconstruct Handle
        let raw_hbm = windows::Win32::Graphics::Gdi::HBITMAP(hbm_ptr as *mut std::ffi::c_void);
        let _safe_hbm = crate::service::win32::gdi::SafeHBITMAP(raw_hbm); // Ensure cleanup

        let bytes_res = win32::bitmap::bitmap_to_png_bytes(raw_hbm);
        if let Ok(bytes) = bytes_res {
            let engine = ocr::get_engine(&engine_id);
            match engine.recognize(&bytes) {
                Ok(text) => {
                    log::info!("OCR Success: {} chars", text.len());
                    let _ = app_handle.emit("ocr-result", text);
                }
                Err(e) => {
                    log::error!("OCR Failed: {:?}", e);
                    let _ = app_handle.emit("ocr-error", e.to_string());
                }
            }
        } else if let Err(e) = bytes_res {
            log::error!("Bitmap conversion failed: {:?}", e);
        }
    });

    Ok(())
}
