use crate::service::native_overlay::state::OverlayState;
use crate::service::win32;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

/// Capture the current selection from the bright bitmap and save it to a file asynchronously.
pub fn save_selection(state_arc: &Arc<Mutex<OverlayState>>, app: &AppHandle) -> anyhow::Result<()> {
    let mut state = match state_arc.lock() {
        Ok(s) => s,
        Err(_) => return Err(anyhow::anyhow!("State mutex poisoned")),
    };
    let sel = match state.selection {
        Some(s) => s,
        None => return Ok(()),
    };

    // Snapshot Size Update Logic
    // If in snapshot mode AND enabled in workflow, update the specific workflow config
    if state.is_snapshot_mode {
        if let Some(workflow) = &state.active_workflow {
            if let crate::service::config::types::CaptureAction::Snapshot { allow_resize, .. } =
                workflow.action
            {
                if allow_resize {
                    let sw = sel.right - sel.left;
                    let sh = sel.bottom - sel.top;
                    if sw > 0 && sh > 0 {
                        let app_inner = app.clone();
                        let workflow_id = workflow.id.clone();
                        let _ = app.run_on_main_thread(move || {
                            let app_state = app_inner.state::<crate::app_state::AppState>();
                            let lock_res = app_state.config_state.lock();
                            if let Ok(mut c_state) = lock_res {
                                // Find workflow and update dimension
                                if let Some(w) = c_state.config.workflows.iter_mut().find(|w| w.id == workflow_id) {
                                    if let crate::service::config::types::CaptureAction::Snapshot { ref mut width, ref mut height, .. } = w.action {
                                        *width = sw;
                                        *height = sh;
                                        log::info!("[Snapshot Mode] Updated Workflow '{}': {}x{}", workflow_id, sw, sh);
                                        let _ = c_state.save();
                                    }
                                }
                            }
                        });
                    }
                }
            }
        }
    }

    // Resolve Output Settings
    let (should_save_file, target_folder_opt, naming_template, format) =
        if let Some(workflow) = &state.active_workflow {
            (
                workflow.output.save_to_file,
                workflow.output.target_folder.clone(),
                workflow.output.naming_template.clone(),
                workflow.output.format.clone(),
            )
        } else {
            // Fallback (shouldn't happen with new logic, but safe default)
            (
                true,
                None,
                "%Y-%m-%d_%H-%M-%S".to_string(),
                "png".to_string(),
            )
        };

    if !should_save_file {
        return Ok(());
    }

    // Check Vello/WGC Background first
    let vello_bg_data = state.vello.background.clone();

    // Create a temporary SafeHBITMAP from Vello if available
    let temp_hbm_bright = if let Some(vello_img) = vello_bg_data {
        let hdc_screen = win32::gdi::get_dc(None)?;
        let hbm = win32::gdi::create_bitmap_from_pixels(
            &hdc_screen,
            state.width,
            state.height,
            vello_img.data.as_ref() as &[u8],
        )?;
        win32::gdi::release_dc(None, hdc_screen);
        Some(hbm)
    } else {
        None
    };

    // Use either the temp one (from Texture) or the existing one (from GDI)
    let hbm_source_ref = if temp_hbm_bright.is_some() {
        temp_hbm_bright.as_ref()
    } else {
        state.gdi.hbitmap_bright.as_ref()
    };

    if let Some(hbm_bright) = hbm_source_ref {
        let width = sel.right - sel.left;
        let height = sel.bottom - sel.top;

        if width <= 0 || height <= 0 {
            return Ok(());
        }

        // GDI: Prepare a full-screen composition buffer
        let hdc_screen = win32::gdi::get_dc(None)?;
        let hdc_mem_comp = win32::gdi::create_compatible_dc(Some(&hdc_screen))?;
        let hbm_comp =
            win32::gdi::create_compatible_bitmap(&hdc_screen, state.width, state.height)?;
        let prev_comp = win32::gdi::select_object(
            &hdc_mem_comp,
            windows::Win32::Graphics::Gdi::HGDIOBJ(hbm_comp.0 .0),
        )?;

        // 1. Draw Background into composition buffer
        let hdc_mem_src = win32::gdi::create_compatible_dc(Some(&hdc_screen))?;
        let prev_src = win32::gdi::select_object(
            &hdc_mem_src,
            windows::Win32::Graphics::Gdi::HGDIOBJ(hbm_bright.0 .0),
        )?;
        win32::gdi::bit_blt(
            &hdc_mem_comp,
            0,
            0,
            state.width,
            state.height,
            &hdc_mem_src,
            0,
            0,
            windows::Win32::Graphics::Gdi::SRCCOPY,
        )?;
        win32::gdi::select_object(&hdc_mem_src, prev_src)?;

        // 2. Draw All Objects into composition buffer
        crate::service::native_overlay::render::drawing::draw_all_objects(
            &hdc_mem_comp,
            &mut state,
        )?;

        // 3. Crop the selection from the composite buffer
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
            &hdc_mem_comp,
            sel.left,
            sel.top,
            windows::Win32::Graphics::Gdi::SRCCOPY,
        )?;

        // Clean up composition buffer
        win32::gdi::select_object(&hdc_mem_comp, prev_comp)?;

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
        // Clone strings for closure
        let template_clone = naming_template.clone();
        let format_clone = format.clone();

        tauri::async_runtime::spawn_blocking(move || {
            // Reconstruct HBITMAP
            let raw_hbm = windows::Win32::Graphics::Gdi::HBITMAP(hbm_ptr as *mut std::ffi::c_void);
            // Re-wrap to ensure Drop calls DeleteObject
            let _safe_hbm = crate::service::win32::gdi::SafeHBITMAP(raw_hbm);

            let captures_dir = if let Some(custom_path) = target_folder_opt {
                std::path::PathBuf::from(custom_path)
            } else {
                let state = app_handle.state::<crate::app_state::AppState>();
                let config_state = match state.config_state.lock() {
                    Ok(c) => c,
                    Err(_) => return,
                };
                config_state.resolve_save_path(&app_handle)
            };

            if !captures_dir.exists() {
                let _ = std::fs::create_dir_all(&captures_dir);
            }

            let now = chrono::Local::now();
            let filename = now.format(&template_clone).to_string();
            // Append extension if not present in template (assuming template is just name base)
            // But usually template is just the name part.
            let full_filename = if filename
                .to_lowercase()
                .ends_with(&format!(".{}", format_clone))
            {
                filename
            } else {
                format!("{}.{}", filename, format_clone)
            };

            let file_path = captures_dir.join(&full_filename);

            // Save (Currently only BMP/PNG supported by save_bitmap_to_file helper?
            // Need to ensure save_bitmap_to_file supports format or update it)
            // Existing logic uses `image` crate encoding.

            let quality = {
                let state = app_handle.state::<crate::app_state::AppState>();
                let config_state = match state.config_state.lock() {
                    Ok(c) => c,
                    Err(_) => return,
                };
                config_state.config.jpg_quality
            };

            if let Err(e) =
                crate::service::win32::bitmap::save_bitmap_to_file(raw_hbm, &file_path, quality)
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

            use crate::service::l10n::{self, L10nKey};
            let _ = app_handle
                .notification()
                .builder()
                .title(l10n::t(&app_handle, L10nKey::NotificationSavedTitle))
                .body(format!(
                    "{}: {}",
                    l10n::t(&app_handle, L10nKey::NotificationSavedBody),
                    file_path.display()
                ))
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
    let mut state = match state_arc.lock() {
        Ok(s) => s,
        Err(_) => return Err(anyhow::anyhow!("State mutex poisoned")),
    };
    let sel = match state.selection {
        Some(s) => s,
        None => return Ok(()),
    };

    // Snapshot Size Memory: Update config with latest size if in snapshot mode
    if state.is_snapshot_mode {
        if let Some(workflow) = &state.active_workflow {
            if let crate::service::config::types::CaptureAction::Snapshot { allow_resize, .. } =
                workflow.action
            {
                if allow_resize {
                    let sw = sel.right - sel.left;
                    let sh = sel.bottom - sel.top;
                    if sw > 0 && sh > 0 {
                        let app_inner = app.clone();
                        let workflow_id = workflow.id.clone();
                        let _ = app.run_on_main_thread(move || {
                            let app_state = app_inner.state::<crate::app_state::AppState>();
                            let lock_res = app_state.config_state.lock();
                            if let Ok(mut c_state) = lock_res {
                                // Find workflow and update dimension
                                if let Some(w) = c_state.config.workflows.iter_mut().find(|w| w.id == workflow_id) {
                                    if let crate::service::config::types::CaptureAction::Snapshot { ref mut width, ref mut height, .. } = w.action {
                                        *width = sw;
                                        *height = sh;
                                        log::info!("[Snapshot Mode] Updated Workflow '{}' (via Clipboard): {}x{}", workflow_id, sw, sh);
                                        let _ = c_state.save();
                                    }
                                }
                            }
                        });
                    }
                }
            }
        }
    }

    // Check Vello/WGC Background first
    let vello_bg_data = state.vello.background.clone();

    // Create a temporary SafeHBITMAP from Vello if available
    let temp_hbm_bright = if let Some(vello_img) = vello_bg_data {
        let hdc_screen = win32::gdi::get_dc(None)?;
        let hbm = win32::gdi::create_bitmap_from_pixels(
            &hdc_screen,
            state.width,
            state.height,
            vello_img.data.as_ref() as &[u8],
        )?;
        win32::gdi::release_dc(None, hdc_screen);
        Some(hbm)
    } else {
        None
    };

    // Use either the temp one (from Texture) or the existing one (from GDI)
    let hbm_source_ref = if temp_hbm_bright.is_some() {
        temp_hbm_bright.as_ref()
    } else {
        state.gdi.hbitmap_bright.as_ref()
    };

    if let Some(hbm_bright) = hbm_source_ref {
        let width = sel.right - sel.left;
        let height = sel.bottom - sel.top;

        if width <= 0 || height <= 0 {
            return Ok(());
        }

        // GDI: Prepare a full-screen composition buffer
        let hdc_screen = win32::gdi::get_dc(None)?;
        let hdc_mem_comp = win32::gdi::create_compatible_dc(Some(&hdc_screen))?;
        let hbm_comp =
            win32::gdi::create_compatible_bitmap(&hdc_screen, state.width, state.height)?;
        let prev_comp = win32::gdi::select_object(
            &hdc_mem_comp,
            windows::Win32::Graphics::Gdi::HGDIOBJ(hbm_comp.0 .0),
        )?;

        // 1. Draw Background into composition buffer
        let hdc_mem_src = win32::gdi::create_compatible_dc(Some(&hdc_screen))?;
        let prev_src = win32::gdi::select_object(
            &hdc_mem_src,
            windows::Win32::Graphics::Gdi::HGDIOBJ(hbm_bright.0 .0),
        )?;
        win32::gdi::bit_blt(
            &hdc_mem_comp,
            0,
            0,
            state.width,
            state.height,
            &hdc_mem_src,
            0,
            0,
            windows::Win32::Graphics::Gdi::SRCCOPY,
        )?;
        win32::gdi::select_object(&hdc_mem_src, prev_src)?;

        // 2. Draw All Objects into composition buffer
        crate::service::native_overlay::render::drawing::draw_all_objects(
            &hdc_mem_comp,
            &mut state,
        )?;

        // 3. Crop the selection from the composite buffer
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
            &hdc_mem_comp,
            sel.left,
            sel.top,
            windows::Win32::Graphics::Gdi::SRCCOPY,
        )?;

        // Clean up composition buffer
        win32::gdi::select_object(&hdc_mem_comp, prev_comp)?;

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

        use crate::service::l10n::{self, L10nKey};
        let _ = app
            .notification()
            .builder()
            .title(l10n::t(app, L10nKey::NotificationCopiedTitle))
            .body(l10n::t(app, L10nKey::NotificationCopiedBody))
            .show();

        use tauri::Emitter;
        let _ = app.emit("screenshot-copied", ());
    }

    Ok(())
}
