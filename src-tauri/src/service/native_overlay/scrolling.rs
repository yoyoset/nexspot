use crate::service::native_overlay::manager::OverlayManager;
use crate::service::native_overlay::save::render::render_snapshot;
use crate::service::win32::bitmap::bitmap_to_rgba_image;
use crate::service::stitch::Stitcher;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Manager, Emitter};
use std::sync::atomic::{AtomicBool, Ordering};

pub struct ScrollingSession {
    pub is_active: Arc<AtomicBool>,
}

pub fn start_scrolling_session(
    app: AppHandle,
    manager_arc: Arc<Mutex<OverlayManager>>,
) -> anyhow::Result<()> {
    // 1. Prepare initial frame
    let first_frame = {
        let mgr = manager_arc.lock().unwrap();
        let snapshot = render_snapshot(&mgr.state, &app)?;
        bitmap_to_rgba_image(snapshot.hbitmap.0)?
    };

    // 2. Initialize Stitcher in Manager
    {
        let mgr = manager_arc.lock().unwrap();
        let mut stitcher_lock = mgr.scroll_stitcher.lock().unwrap();
        *stitcher_lock = Some(Stitcher::new(first_frame));
        
        let mut state = mgr.state.write().unwrap();
        state.is_scrolling = true;
    }

    // 3. Spawn capture loop
    let is_active = Arc::new(AtomicBool::new(true));
    let is_active_clone = is_active.clone();
    let app_clone = app.clone();
    
    tauri::async_runtime::spawn(async move {
        log::info!("[Scrolling] Started background capture loop");
        let mut interval = tokio::time::interval(Duration::from_millis(150));
        
        while is_active_clone.load(Ordering::SeqCst) {
            interval.tick().await;

            // Capture and stitch
            let result = {
                let state = app_clone.state::<crate::app_state::AppState>();
                let mgr = state.overlay_manager.lock().unwrap();
                
                let inner_res: anyhow::Result<u32> = match render_snapshot(&mgr.state, &app_clone) {
                    Ok(snapshot) => {
                        match bitmap_to_rgba_image(snapshot.hbitmap.0) {
                            Ok(img) => {
                                let mut stitcher_lock = mgr.scroll_stitcher.lock().unwrap();
                                if let Some(ref mut stitcher) = *stitcher_lock {
                                    stitcher.add_frame(&img)
                                } else {
                                    Err(anyhow::anyhow!("Stitcher lost"))
                                }
                            }
                            Err(e) => Err(e),
                        }
                    }
                    Err(e) => Err(e),
                };
                inner_res
            };

            match result {
                Ok(new_h) => {
                    if new_h > 0 {
                        // Notify progress
                        let _ = app_clone.emit("scrolling://progress", new_h);
                    }
                }
                Err(e) => {
                    log::error!("[Scrolling] Loop error: {}", e);
                    break;
                }
            }
        }
        log::info!("[Scrolling] Background capture loop stopped");
    });

    Ok(())
}

pub fn stop_scrolling_session(
    app: AppHandle,
    manager_arc: Arc<Mutex<OverlayManager>>,
) -> anyhow::Result<String> {
    // 1. Stop loop and get result
    let final_image = {
        let mgr = manager_arc.lock().unwrap();
        let mut state = mgr.state.write().unwrap();
        state.is_scrolling = false;
        
        let mut stitcher_lock = mgr.scroll_stitcher.lock().unwrap();
        if let Some(stitcher) = stitcher_lock.take() {
            stitcher.current_image
        } else {
            anyhow::bail!("No active scrolling session");
        }
    };

    // 2. Save result to cache
    let cache_dir = app.path().app_cache_dir()?;
    if !cache_dir.exists() {
        std::fs::create_dir_all(&cache_dir)?;
    }
    let filename = format!("scrolling_{}.png", uuid::Uuid::new_v4());
    let path = cache_dir.join(filename);
    
    final_image.save(&path)?;
    let path_str = path.to_string_lossy().to_string();

    // 3. Cleanup old cache files
    let _ = cleanup_scrolling_cache(&app);

    {
        let mgr = manager_arc.lock().unwrap();
        let mut state = mgr.state.write().unwrap();
        state.scroll_stitched_path = Some(path_str.clone());
    }

    // 3. Open Preview Window
    let preview_url = tauri::WebviewUrl::App("index.html#scrolling-preview".into());
    let _ = tauri::WebviewWindowBuilder::new(&app, "scrolling-preview", preview_url)
        .title("Scrolling Preview")
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .inner_size(800.0, 600.0)
        .center()
        .build();

    // 4. Log activity
    crate::service::activity::log_activity(
        &app,
        "scrolling",
        Some(path_str.clone()),
        None
    );

    Ok(path_str)
}

fn cleanup_scrolling_cache(app: &AppHandle) -> anyhow::Result<()> {
    let cache_dir = app.path().app_cache_dir()?;
    if !cache_dir.exists() {
        return Ok(());
    }

    let now = std::time::SystemTime::now();
    let one_day = Duration::from_secs(24 * 3600);

    for entry in std::fs::read_dir(cache_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && path.extension().map_or(false, |ext| ext == "png") {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.starts_with("scrolling_") {
                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            if let Ok(elapsed) = now.duration_since(modified) {
                                if elapsed > one_day {
                                    let _ = std::fs::remove_file(path);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn start_scrolling(app: AppHandle) -> Result<(), String> {
    let state = app.state::<crate::app_state::AppState>();
    let mgr = state.overlay_manager.clone();
    start_scrolling_session(app, mgr)
        .map_err(|e: anyhow::Error| e.to_string())
}

#[tauri::command]
pub async fn stop_scrolling(app: AppHandle) -> Result<String, String> {
    let state = app.state::<crate::app_state::AppState>();
    let mgr = state.overlay_manager.clone();
    stop_scrolling_session(app, mgr)
        .map_err(|e: anyhow::Error| e.to_string())
}

#[tauri::command]
pub async fn get_last_scrolled_image(app: AppHandle) -> Result<Option<String>, String> {
    let state = app.state::<crate::app_state::AppState>();
    let manager = state.overlay_manager.lock().unwrap();
    let state_lock = manager.state.read().unwrap();
    Ok(state_lock.scroll_stitched_path.clone())
}

#[tauri::command]
pub async fn save_scrolled_image_to(source: String, destination: String) -> Result<(), String> {
    std::fs::copy(source, destination).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn copy_image_to_clipboard(path: String) -> Result<(), String> {
    use crate::service::win32::clipboard::set_clipboard_image_from_file;
    set_clipboard_image_from_file(std::path::Path::new(&path))
        .map_err(|e: anyhow::Error| e.to_string())
}
