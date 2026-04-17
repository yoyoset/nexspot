use crate::service::native_overlay::state::OverlayState;
use std::sync::{Arc, RwLock};
use tauri::{AppHandle, Manager, Emitter};
use super::render::{render_snapshot, update_snapshot_size_config};
use crate::service::l10n;
use tauri_plugin_notification::NotificationExt;

pub fn save_selection(state_arc: &Arc<RwLock<OverlayState>>, app: &AppHandle) -> anyhow::Result<()> {
    // 1. Update config if needed
    update_snapshot_size_config(state_arc, app);

    // 2. Resolve Output Settings
    let (should_save_file, target_folder_opt, naming_template, format, _active_wf_opt) = {
        let state = state_arc.read().unwrap_or_else(|e| e.into_inner());
        if let Some(workflow) = &state.active_workflow {
            (
                workflow.output.save_to_file,
                workflow.output.target_folder.clone(),
                workflow.output.naming_template.clone(),
                workflow.output.format.clone(),
                Some(workflow.clone()),
            )
        } else {
            (true, None, "%Y-%m-%d_%H-%M-%S".to_string(), "png".to_string(), None)
        }
    };

    if !should_save_file {
        return Ok(());
    }

    // 3. Render
    let rendered = render_snapshot(state_arc, app)?;
    let hbm_ptr = rendered.hbitmap.leak().0 as usize;
    
    // 4. Save to Disk
    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let raw_hbm = windows::Win32::Graphics::Gdi::HBITMAP(hbm_ptr as *mut std::ffi::c_void);
        let _safe_hbm = crate::service::win32::gdi::SafeHBITMAP(raw_hbm);
        
        let app_state = app_handle.state::<crate::app_state::AppState>();
        let captures_dir = if let Some(cp) = target_folder_opt {
            std::path::PathBuf::from(cp)
        } else {
            let c_state = app_state.config_state.lock().unwrap_or_else(|e| e.into_inner());
            c_state.resolve_save_path(&app_handle)
        };
        
        if !captures_dir.exists() {
            let _ = std::fs::create_dir_all(&captures_dir);
        }
        
        let now = chrono::Local::now();
        let mut filename = now.format(&naming_template).to_string();
        if !filename.to_lowercase().ends_with(&format!(".{}", format)) {
            filename.push_str(&format!(".{}", format));
        }
        let file_path = captures_dir.join(filename);
        
        let quality = app_handle.state::<crate::app_state::AppState>().config_state.lock().unwrap_or_else(|e| e.into_inner()).config.jpg_quality;
        
        if let Err(e) = crate::service::win32::bitmap::save_bitmap_to_file(raw_hbm, &file_path, quality) {
            log::error!("GDI Save fail: {:?}", e);
        } else {
            let _ = app_handle.notification().builder()
                .title(l10n::t(&app_handle, "backend.notification.saved_title", "Saved"))
                .body(format!("{}: {}", l10n::t(&app_handle, "backend.notification.saved_body", "Image saved locally"), file_path.display()))
                .show();
            let _ = app_handle.emit("screenshot-saved", ());
        }
    });

    Ok(())
}
pub fn save_selection_to_path(
    state_arc: Arc<RwLock<crate::service::native_overlay::state::OverlayState>>,
    app: &AppHandle,
    path: String,
) -> anyhow::Result<()> {
    // 0. Update config if needed
    update_snapshot_size_config(&state_arc, app);

    // 1. Render
    let rendered = render_snapshot(&state_arc, app)?;
    let hbm_ptr = rendered.hbitmap.leak().0 as usize;
    
    // 2. Save to Disk
    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let raw_hbm = windows::Win32::Graphics::Gdi::HBITMAP(hbm_ptr as *mut std::ffi::c_void);
        let _safe_hbm = crate::service::win32::gdi::SafeHBITMAP(raw_hbm);
        
        let app_state = app_handle.state::<crate::app_state::AppState>();
        let path_buf = std::path::PathBuf::from(path);
        let quality = app_state.config_state.lock().unwrap_or_else(|e| e.into_inner()).config.jpg_quality;
        
        if let Err(e) = crate::service::win32::bitmap::save_bitmap_to_file(raw_hbm, &path_buf, quality) {
            log::error!("GDI SaveAs fail: {:?}", e);
        } else {
            let _ = app_handle.notification().builder()
                .title(l10n::t(&app_handle, "backend.notification.saved_title", "Saved"))
                .body(format!("{}: {}", l10n::t(&app_handle, "backend.notification.saved_body", "Image saved locally"), path_buf.display()))
                .show();
            let _ = app_handle.emit("screenshot-saved", ());
        }
    });

    Ok(())
}
