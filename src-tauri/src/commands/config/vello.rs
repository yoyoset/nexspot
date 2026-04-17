use crate::app_state::AppState;
use crate::service::config::ConfigError;
use crate::service::native_overlay::OverlayManager;
use crate::service::config::types::AestheticStyle;
use tauri::State;

#[tauri::command]
pub fn set_vello_enabled(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<(), ConfigError> {
    log::info!("[Config] set_vello_enabled called with: {}", enabled);
    {
        let mut config_state = state.config_state.lock().unwrap_or_else(|e| e.into_inner());
        config_state.set_vello_enabled(enabled);
    }

    // Update OverlayManager's pre-heat stream lifecycle
    let mut overlay_mgr = state.overlay_manager.lock().unwrap_or_else(|e| e.into_inner());
    if enabled {
        overlay_mgr.start_pre_heat();
        // If Vello isn't ready yet, trigger initialization
        log::info!(
            "[Config] Vello enabled. Context exists: {}",
            overlay_mgr.vello_ctx.is_some()
        );
        if overlay_mgr.vello_ctx.is_none() {
            drop(overlay_mgr); // Release lock before async init
            log::info!("[Config] Triggering init_vello_async...");
            OverlayManager::init_vello_async(app);
        }
    } else {
        log::info!("[Config] Vello disabled. Stopping pre-heat.");
        overlay_mgr.stop_pre_heat();
    }

    Ok(())
}

#[tauri::command]
pub async fn set_vello_aesthetic_style(
    state: State<'_, AppState>,
    style: AestheticStyle,
) -> Result<(), String> {
    {
        let mut config_guard = state.config_state.lock().map_err(|e| e.to_string())?;
        config_guard.config.vello_aesthetic_style = style;
        config_guard.save(); // save() returns ()
    }

    // Sync to active overlay
    if let Ok(overlay) = state.overlay_manager.lock() {
        if let Ok(mut os) = overlay.state.write() {
            os.current_style = style;
        }
    }

    Ok(())
}

#[tauri::command]
pub fn set_vello_advanced_effects(
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<(), ConfigError> {
    {
        let mut config_state = state.config_state.lock().unwrap_or_else(|e| e.into_inner());
        config_state.set_vello_advanced_effects(enabled);
    }

    // Update active overlay state immediately
    if let Ok(overlay) = state.overlay_manager.lock() {
        if let Ok(mut overlay_state) = overlay.state.write() {
            overlay_state.enable_advanced_effects = enabled;
        }
    }

    Ok(())
}

#[tauri::command]
pub fn emergency_reset_to_gdi(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), ConfigError> {
    log::warn!("[Config] EMERGENCY RESET: Forcing all engines to GDI due to hardware failure.");
    let mut c_state = state.config_state.lock().unwrap_or_else(|e| e.into_inner());

    // 1. Force global flags
    c_state.config.vello_enabled = false;
    c_state.config.selection_engine = "gdi".to_string();
    c_state.config.snapshot_engine = "gdi".to_string();

    // 2. Iterate and fix all workflows
    use crate::service::config::types::CaptureAction;
    for workflow in &mut c_state.config.workflows {
        match &mut workflow.action {
            CaptureAction::Selection { engine } => *engine = "gdi".to_string(),
            CaptureAction::Fullscreen { engine } => *engine = "gdi".to_string(),
            CaptureAction::Window { engine } => *engine = "gdi".to_string(),
            CaptureAction::Snapshot { engine, .. } => *engine = "gdi".to_string(),
        }
    }

    // 3. Persistent save
    c_state.save();

    // 4. Force stop pre-heat and cleanup
    drop(c_state); // Release lock before manager lock
    let mut overlay_mgr = state.overlay_manager.lock().unwrap_or_else(|e| e.into_inner());
    overlay_mgr.stop_pre_heat();
    // Also clear existing Vello context if any
    overlay_mgr.vello_ctx = None;
    overlay_mgr.vello_status = crate::service::native_overlay::state::VelloStatus::Ready; // Reset to pseudo-ready (Gdi path)

    // 5. Notify frontend to re-fetch config
    use tauri::Emitter;
    let _ = app.emit("vello://ready", ()); // Fake a ready event to clear errors if UI listens

    Ok(())
}
