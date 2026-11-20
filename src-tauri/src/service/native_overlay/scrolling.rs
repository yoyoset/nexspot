use crate::service::native_overlay::manager::OverlayManager;
use crate::service::stitch::{StitchFrameReport, Stitcher};
use crate::service::win32;
use crate::service::win32::bitmap::bitmap_to_rgba_image;
use crate::service::win32::window::SafeHWND;
use image::RgbaImage;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};
use windows::Win32::Foundation::{HWND, POINT, RECT};

const CAPTURE_INTERVAL_MS: u64 = 260;
const SCROLL_SETTLE_MS: u64 = 220;
const NO_GROWTH_LIMIT: u8 = 4;
const MAX_CAPTURE_FRAMES: u16 = 80;
const WHEEL_STEP: i32 = -360;
const FINISH_REASON_MANUAL: &str = "manual";
const FINISH_REASON_NO_GROWTH: &str = "no_growth";
const FINISH_REASON_LIMIT: &str = "frame_limit";
const FINISH_REASON_ERROR: &str = "error";

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScrollingTargetWindow {
    pub hwnd: isize,
    pub process_id: u32,
    pub class_name: String,
    pub title: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScrollingCaptureResult {
    pub path: String,
    pub width: u32,
    pub height: u32,
    pub reason: &'static str,
    pub error_message: Option<String>,
    pub captured_frames: u16,
    pub appended_frames: u16,
    pub stagnant_frames: u8,
    pub total_added_px: u32,
    pub duration_ms: u128,
    pub target_window: Option<ScrollingTargetWindow>,
    pub max_capture_frames: u16,
    pub no_growth_limit: u8,
    pub wheel_step: i32,
    pub stitch_reports: Vec<StitchFrameReport>,
}

#[derive(Clone, Debug, Default)]
struct ScrollingSessionStats {
    captured_frames: u16,
    appended_frames: u16,
    stagnant_frames: u8,
    total_added_px: u32,
    duration_ms: u128,
    target_window: Option<ScrollingTargetWindow>,
    error_message: Option<String>,
    stitch_reports: Vec<StitchFrameReport>,
}

pub fn start_scrolling_session(
    app: AppHandle,
    manager_arc: Arc<Mutex<OverlayManager>>,
) -> anyhow::Result<()> {
    let (selection, stitcher_arc) = {
        let mgr = manager_arc.lock().unwrap_or_else(|e| e.into_inner());
        let selection = {
            let mut state = mgr.state.write().unwrap_or_else(|e| e.into_inner());
            if state.is_scrolling {
                anyhow::bail!("Scrolling capture is already running");
            }
            let selection = state.selection.ok_or_else(|| {
                anyhow::anyhow!("Select an area before starting scrolling capture")
            })?;
            if selection.right <= selection.left || selection.bottom <= selection.top {
                anyhow::bail!("Invalid scrolling selection");
            }
            state.is_scrolling = true;
            state.scroll_stitched_path = None;
            state.scroll_result_data = None;
            state.is_visible = false;
            selection
        };

        hide_overlay_windows(&mgr);
        (selection, mgr.scroll_stitcher.clone())
    };

    std::thread::sleep(Duration::from_millis(120));
    let center = rect_center(selection);
    let target_window = describe_window_under_point(center);
    let first_frame = match capture_screen_rect(selection) {
        Ok(frame) => frame,
        Err(e) => {
            reset_failed_scrolling_start(&manager_arc);
            return Err(anyhow::anyhow!(
                "Failed to capture initial scrolling frame: {}",
                e
            ));
        }
    };
    {
        let mut stitcher_lock = stitcher_arc.lock().unwrap_or_else(|e| e.into_inner());
        *stitcher_lock = Some(Stitcher::new(first_frame));
    }
    let _ = app.emit(
        "scrolling://started",
        serde_json::json!({
            "shortcut": "Esc",
            "direction": "down",
            "targetWindow": target_window.clone(),
            "selection": {
                "left": selection.left,
                "top": selection.top,
                "right": selection.right,
                "bottom": selection.bottom,
                "width": selection.right - selection.left,
                "height": selection.bottom - selection.top,
            },
            "settings": scrolling_runtime_settings(),
        }),
    );

    let app_clone = app.clone();
    let manager_clone = manager_arc.clone();
    tauri::async_runtime::spawn(async move {
        log::info!("[Scrolling] Started auto-scroll capture loop");
        let started_at = Instant::now();
        let mut stagnant_ticks = 0u8;
        let mut captured_frames = 1u16;
        let mut appended_frames = 0u16;
        let mut total_added_px = 0u32;
        let mut stitch_reports = Vec::new();
        let mut finish_reason = FINISH_REASON_LIMIT;
        let mut error_message: Option<String> = None;

        while is_scrolling_active(&manager_clone) && captured_frames < MAX_CAPTURE_FRAMES {
            if is_escape_pressed() {
                finish_reason = FINISH_REASON_MANUAL;
                log::info!("[Scrolling] Manual stop requested via Esc");
                break;
            }

            scroll_under_cursor(center);
            tokio::time::sleep(Duration::from_millis(SCROLL_SETTLE_MS)).await;

            if !is_scrolling_active(&manager_clone) {
                log::info!("[Scrolling] Capture loop stopped during scroll settle");
                return;
            }

            if is_escape_pressed() {
                finish_reason = FINISH_REASON_MANUAL;
                log::info!("[Scrolling] Manual stop requested via Esc");
                break;
            }

            let result = capture_screen_rect(selection).and_then(|img| {
                let stitcher_arc = {
                    let mgr = manager_clone.lock().unwrap_or_else(|e| e.into_inner());
                    mgr.scroll_stitcher.clone()
                };
                let mut stitcher_lock = stitcher_arc.lock().unwrap_or_else(|e| e.into_inner());
                if let Some(ref mut stitcher) = *stitcher_lock {
                    let report = stitcher.add_frame_with_report(&img)?;
                    Ok((report, stitcher.current_image.height()))
                } else {
                    anyhow::bail!("Stitcher lost")
                }
            });

            match result {
                Ok((report, total_height)) => {
                    captured_frames += 1;
                    let added = report.added_px;
                    let rejected_reason = report.rejected_reason;
                    stitch_reports.push(report.clone());
                    if added > 0 {
                        stagnant_ticks = 0;
                        appended_frames += 1;
                        total_added_px = total_added_px.saturating_add(added);
                        let _ = app_clone.emit(
                            "scrolling://progress",
                            serde_json::json!({
                                "height": total_height,
                                "added": added,
                                "capturedFrames": captured_frames,
                                "appendedFrames": appended_frames,
                                "stagnantFrames": stagnant_ticks,
                                "stitchReport": report,
                            }),
                        );
                    } else {
                        stagnant_ticks = stagnant_ticks.saturating_add(1);
                        let _ = app_clone.emit(
                            "scrolling://progress",
                            serde_json::json!({
                                "height": total_height,
                                "added": 0,
                                "capturedFrames": captured_frames,
                                "appendedFrames": appended_frames,
                                "stagnantFrames": stagnant_ticks,
                                "phase": "checking_stop",
                                "rejectedReason": rejected_reason,
                                "stitchReport": report,
                            }),
                        );
                        if stagnant_ticks >= NO_GROWTH_LIMIT {
                            finish_reason = FINISH_REASON_NO_GROWTH;
                            log::info!("[Scrolling] Auto-stop: no new content detected");
                            break;
                        }
                    }
                }
                Err(e) => {
                    finish_reason = FINISH_REASON_ERROR;
                    let message = e.to_string();
                    log::error!("[Scrolling] Loop error: {}", message);
                    error_message = Some(message.clone());
                    crate::service::notification::emit_capture_error(
                        &app_clone,
                        "scrolling",
                        "loop_error",
                        &crate::service::l10n::t(
                            &app_clone,
                            "backend.notification.scrolling_failed_title",
                            "Scrolling Capture Failed",
                        ),
                        &message,
                    );
                    let _ = app_clone.emit(
                        "scrolling://progress",
                        serde_json::json!({
                            "height": 0,
                            "added": 0,
                            "capturedFrames": captured_frames,
                            "appendedFrames": appended_frames,
                            "stagnantFrames": stagnant_ticks,
                            "phase": "error",
                            "message": message,
                        }),
                    );
                    break;
                }
            }

            tokio::time::sleep(Duration::from_millis(CAPTURE_INTERVAL_MS)).await;
            if !is_scrolling_active(&manager_clone) {
                log::info!("[Scrolling] Capture loop stopped during capture interval");
                return;
            }
        }

        if finish_reason == FINISH_REASON_LIMIT && captured_frames < MAX_CAPTURE_FRAMES {
            finish_reason = FINISH_REASON_MANUAL;
        }

        let stats = ScrollingSessionStats {
            captured_frames,
            appended_frames,
            stagnant_frames: stagnant_ticks,
            total_added_px,
            duration_ms: started_at.elapsed().as_millis(),
            target_window,
            error_message,
            stitch_reports,
        };

        if !is_scrolling_active(&manager_clone) {
            log::info!("[Scrolling] Capture loop observed an external stop");
            return;
        }

        match finish_scrolling_session(app_clone, manager_clone, true, finish_reason, Some(stats)) {
            Ok(path) => log::info!("[Scrolling] Finished: {}", path),
            Err(e) => log::error!("[Scrolling] Finish failed: {}", e),
        }
    });

    Ok(())
}

pub fn stop_scrolling_session(
    app: AppHandle,
    manager_arc: Arc<Mutex<OverlayManager>>,
) -> anyhow::Result<String> {
    finish_scrolling_session(app, manager_arc, true, FINISH_REASON_MANUAL, None)
}

fn finish_scrolling_session(
    app: AppHandle,
    manager_arc: Arc<Mutex<OverlayManager>>,
    open_preview: bool,
    reason: &'static str,
    stats: Option<ScrollingSessionStats>,
) -> anyhow::Result<String> {
    let final_image = {
        let mgr = manager_arc.lock().unwrap_or_else(|e| e.into_inner());
        let mut state = mgr.state.write().unwrap_or_else(|e| e.into_inner());
        state.is_scrolling = false;

        let mut stitcher_lock = mgr
            .scroll_stitcher
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(stitcher) = stitcher_lock.take() {
            stitcher.current_image
        } else {
            anyhow::bail!("No active scrolling session");
        }
    };

    let cache_dir = app.path().app_cache_dir()?;
    if !cache_dir.exists() {
        std::fs::create_dir_all(&cache_dir)?;
    }
    let filename = format!("scrolling_{}.png", uuid::Uuid::new_v4());
    let path = cache_dir.join(filename);
    final_image.save(&path)?;
    let path_str = path.to_string_lossy().to_string();

    let _ = cleanup_scrolling_cache(&app);

    let stats = stats.unwrap_or(ScrollingSessionStats {
        captured_frames: 1,
        appended_frames: 0,
        stagnant_frames: 0,
        total_added_px: 0,
        duration_ms: 0,
        target_window: None,
        error_message: None,
        stitch_reports: Vec::new(),
    });
    let result = ScrollingCaptureResult {
        path: path_str.clone(),
        width: final_image.width(),
        height: final_image.height(),
        reason,
        error_message: stats.error_message,
        captured_frames: stats.captured_frames,
        appended_frames: stats.appended_frames,
        stagnant_frames: stats.stagnant_frames,
        total_added_px: stats.total_added_px,
        duration_ms: stats.duration_ms,
        target_window: stats.target_window,
        max_capture_frames: MAX_CAPTURE_FRAMES,
        no_growth_limit: NO_GROWTH_LIMIT,
        wheel_step: WHEEL_STEP,
        stitch_reports: stats.stitch_reports,
    };

    {
        let mgr = manager_arc.lock().unwrap_or_else(|e| e.into_inner());
        let mut state = mgr.state.write().unwrap_or_else(|e| e.into_inner());
        state.scroll_stitched_path = Some(path_str.clone());
        state.scroll_result_data = Some(result.clone());
    }

    if open_preview {
        open_scrolling_preview(&app);
    }

    let _ = app.emit("scrolling://finished", result);

    crate::service::activity::log_activity(&app, "scrolling", Some(path_str.clone()), None);

    Ok(path_str)
}

fn capture_screen_rect(rect: RECT) -> anyhow::Result<RgbaImage> {
    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;
    if width <= 0 || height <= 0 {
        anyhow::bail!("Invalid capture rect");
    }

    let hdc_screen = win32::gdi::get_dc(None)?;
    let hdc_mem = win32::gdi::create_compatible_dc(Some(&hdc_screen))?;
    let hbm = win32::gdi::create_compatible_bitmap(&hdc_screen, width, height)?;
    let prev =
        win32::gdi::select_object(&hdc_mem, windows::Win32::Graphics::Gdi::HGDIOBJ(hbm.0 .0))?;
    let rop = windows::Win32::Graphics::Gdi::ROP_CODE(
        windows::Win32::Graphics::Gdi::SRCCOPY.0 | 0x40000000,
    );
    win32::gdi::bit_blt(
        &hdc_mem,
        0,
        0,
        width,
        height,
        &hdc_screen,
        rect.left,
        rect.top,
        rop,
    )?;
    win32::gdi::select_object(&hdc_mem, prev)?;
    win32::gdi::release_dc(None, hdc_screen);

    bitmap_to_rgba_image(hbm.0)
}

fn hide_overlay_windows(manager: &OverlayManager) {
    for hwnd in manager.gdi_hwnds.values() {
        win32::window::hide_window(&SafeHWND(hwnd.0));
    }
    for hwnd in manager.vello_hwnds.values() {
        win32::window::hide_window(&SafeHWND(hwnd.0));
    }
}

fn is_scrolling_active(manager_arc: &Arc<Mutex<OverlayManager>>) -> bool {
    let mgr = manager_arc.lock().unwrap_or_else(|e| e.into_inner());
    mgr.state
        .read()
        .map(|state| state.is_scrolling)
        .unwrap_or(false)
}

fn reset_failed_scrolling_start(manager_arc: &Arc<Mutex<OverlayManager>>) {
    let mgr = manager_arc.lock().unwrap_or_else(|e| e.into_inner());
    {
        let mut state = mgr.state.write().unwrap_or_else(|e| e.into_inner());
        state.is_scrolling = false;
        state.scroll_stitched_path = None;
        state.scroll_result_data = None;
    }
    {
        let mut stitcher = mgr
            .scroll_stitcher
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        *stitcher = None;
    }
}

fn rect_center(rect: RECT) -> POINT {
    POINT {
        x: rect.left + (rect.right - rect.left) / 2,
        y: rect.top + (rect.bottom - rect.top) / 2,
    }
}

fn scrolling_runtime_settings() -> serde_json::Value {
    serde_json::json!({
        "captureIntervalMs": CAPTURE_INTERVAL_MS,
        "scrollSettleMs": SCROLL_SETTLE_MS,
        "noGrowthLimit": NO_GROWTH_LIMIT,
        "maxCaptureFrames": MAX_CAPTURE_FRAMES,
        "wheelStep": WHEEL_STEP,
    })
}

fn describe_window_under_point(point: POINT) -> Option<ScrollingTargetWindow> {
    unsafe {
        let hwnd = windows::Win32::UI::WindowsAndMessaging::WindowFromPoint(point);
        describe_window(hwnd)
    }
}

fn describe_window(hwnd: HWND) -> Option<ScrollingTargetWindow> {
    if hwnd.is_invalid() {
        return None;
    }

    unsafe {
        let mut process_id = 0u32;
        let _ = windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId(
            hwnd,
            Some(&mut process_id),
        );

        let mut class_buf = [0u16; 256];
        let class_len =
            windows::Win32::UI::WindowsAndMessaging::GetClassNameW(hwnd, &mut class_buf);
        let class_name = String::from_utf16_lossy(&class_buf[..class_len.max(0) as usize]);

        let title_len = windows::Win32::UI::WindowsAndMessaging::GetWindowTextLengthW(hwnd);
        let mut title_buf = vec![0u16; title_len.max(0) as usize + 1];
        let copied = if title_buf.len() > 1 {
            windows::Win32::UI::WindowsAndMessaging::GetWindowTextW(hwnd, &mut title_buf)
        } else {
            0
        };
        let title = String::from_utf16_lossy(&title_buf[..copied.max(0) as usize]);

        Some(ScrollingTargetWindow {
            hwnd: hwnd.0 as isize,
            process_id,
            class_name,
            title,
        })
    }
}

fn scroll_under_cursor(point: POINT) {
    unsafe {
        let _ = windows::Win32::UI::WindowsAndMessaging::SetCursorPos(point.x, point.y);
        let hwnd = windows::Win32::UI::WindowsAndMessaging::WindowFromPoint(point);
        if !hwnd.is_invalid() {
            let _ = windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow(hwnd);
        }
        windows::Win32::UI::Input::KeyboardAndMouse::mouse_event(
            windows::Win32::UI::Input::KeyboardAndMouse::MOUSEEVENTF_WHEEL,
            0,
            0,
            WHEEL_STEP,
            0,
        );
    }
}

fn is_escape_pressed() -> bool {
    unsafe {
        (windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(
            windows::Win32::UI::Input::KeyboardAndMouse::VK_ESCAPE.0 as i32,
        ) as u16
            & 0x8000)
            != 0
    }
}

fn open_scrolling_preview(app: &AppHandle) {
    let preview_url = tauri::WebviewUrl::App("index.html#scrolling-preview".into());
    let _ = tauri::WebviewWindowBuilder::new(app, "scrolling-preview", preview_url)
        .title("Scrolling Preview")
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .inner_size(800.0, 600.0)
        .center()
        .build();
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
    start_scrolling_session(app, mgr).map_err(|e: anyhow::Error| e.to_string())
}

#[tauri::command]
pub async fn stop_scrolling(app: AppHandle) -> Result<String, String> {
    let state = app.state::<crate::app_state::AppState>();
    let mgr = state.overlay_manager.clone();
    stop_scrolling_session(app, mgr).map_err(|e: anyhow::Error| e.to_string())
}

#[tauri::command]
pub async fn get_last_scrolled_image(app: AppHandle) -> Result<Option<String>, String> {
    let state = app.state::<crate::app_state::AppState>();
    let manager = state
        .overlay_manager
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let state_lock = manager.state.read().unwrap_or_else(|e| e.into_inner());
    Ok(state_lock.scroll_stitched_path.clone())
}

#[tauri::command]
pub async fn get_last_scrolling_result(
    app: AppHandle,
) -> Result<Option<ScrollingCaptureResult>, String> {
    let state = app.state::<crate::app_state::AppState>();
    let manager = state
        .overlay_manager
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let state_lock = manager.state.read().unwrap_or_else(|e| e.into_inner());
    Ok(state_lock.scroll_result_data.clone())
}

fn current_scrolling_image_path(app: &AppHandle) -> Result<PathBuf, String> {
    let state = app.state::<crate::app_state::AppState>();
    let manager = state
        .overlay_manager
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let state_lock = manager.state.read().unwrap_or_else(|e| e.into_inner());
    state_lock
        .scroll_result_data
        .as_ref()
        .map(|result| PathBuf::from(&result.path))
        .or_else(|| state_lock.scroll_stitched_path.as_ref().map(PathBuf::from))
        .ok_or_else(|| "No scrolling capture result is available".to_string())
}

fn validated_scrolling_image_path(app: &AppHandle, requested: &str) -> Result<PathBuf, String> {
    let expected = current_scrolling_image_path(app)?;
    let expected = expected
        .canonicalize()
        .map_err(|e| format!("Current scrolling image is unavailable: {}", e))?;
    let requested = PathBuf::from(requested)
        .canonicalize()
        .map_err(|e| format!("Requested scrolling image is unavailable: {}", e))?;

    if requested != expected {
        return Err("Requested image does not match the current scrolling capture".to_string());
    }

    Ok(requested)
}

#[tauri::command]
pub async fn save_scrolled_image_to(
    app: AppHandle,
    source: String,
    destination: String,
) -> Result<(), String> {
    let source_path = validated_scrolling_image_path(&app, &source)?;
    std::fs::copy(source_path, destination).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn copy_image_to_clipboard(app: AppHandle, path: String) -> Result<(), String> {
    use crate::service::win32::clipboard::set_clipboard_image_from_file;
    let source_path = validated_scrolling_image_path(&app, &path)?;
    set_clipboard_image_from_file(source_path.as_path()).map_err(|e: anyhow::Error| e.to_string())
}
