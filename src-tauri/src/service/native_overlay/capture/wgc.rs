use crate::service::native_overlay::state::OverlayState;
use crate::service::win32;
use std::sync::{Arc, RwLock, Mutex};
use windows::Win32::Foundation::{POINT, RECT};

pub fn capture_wgc(
    state: &Arc<RwLock<OverlayState>>,
    _stream_states: Option<
        Arc<
            Mutex<
                std::collections::HashMap<
                    String,
                    Arc<Mutex<crate::service::win32::wgc::capture::StreamState>>,
                >,
            >,
        >,
    >,
    target_monitor_id: &str,
    target_monitor_friendly_name: &str,
    target_monitor_rect: &RECT,
    cursor: &POINT,
) -> anyhow::Result<(i32, i32, i32, i32)> {
    log::info!("Starting WGC Mode (HMONITOR Focus)");

    log::info!(
        "WGC Target HMONITOR: {} (Friendly: {})",
        target_monitor_id,
        target_monitor_friendly_name
    );

    let mut captured_via_stream = false;
    // Stream optimization
    if let Some(states_map_arc) = _stream_states {
        if let Ok(map) = states_map_arc.lock() {
            // Direct lookup by HMONITOR ID - Guaranteed stable
            if let Some(ss) = map.get(target_monitor_id) {
                if let Ok(lock) = ss.lock() {
                    if lock.is_alive {
                        if let Some(img) = lock.image.clone() {
                            log::info!(
                                "[WGC] ✓ Using STREAM frame for HMONITOR {}",
                                target_monitor_id
                            );
                            if let Ok(mut s) = state.write() {
                                s.vello.background = Some(img);
                                captured_via_stream = true;
                            }
                        }
                    }
                }
            } else {
                log::warn!("[WGC] No stream entry for HMONITOR {}", target_monitor_id);
            }
        }
    }

    if !captured_via_stream {
        log::info!(
            "[WGC] Attempting One-shot capture for HMONITOR {}...",
            target_monitor_id
        );

        match crate::service::win32::wgc::capture::capture_monitor_to_vello(
            target_monitor_id,
            target_monitor_friendly_name,
            Some(*target_monitor_rect),
        ) {
            Ok((img, _size)) => {
                log::info!("[WGC] ✓ One-shot capture SUCCESS");
                if let Ok(mut s) = state.write() {
                    s.vello.background = Some(img);
                }
            }
            Err(e) => {
                log::error!("[WGC] ✗ One-shot capture FAILED: {:?}", e);
            }
        }
    }

    let final_x = target_monitor_rect.left;
    let final_y = target_monitor_rect.top;
    let final_w = target_monitor_rect.right - target_monitor_rect.left;
    let final_h = target_monitor_rect.bottom - target_monitor_rect.top;

    let snap_rects = win32::window::enumerate_visible_windows();

    if let Ok(mut s) = state.write() {
        s.capture_x = final_x;
        s.capture_y = final_y;
        s.width = final_w;
        s.height = final_h;
        s.mouse_x = cursor.x;
        s.mouse_y = cursor.y;
        s.window_rects = snap_rects;
        s.selection = None;
        s.is_capturing = false;
        s.gdi.hbitmap_bright = None;
        s.gdi.hbitmap_dim = None;
        s.restrict_to_monitor = Some(*target_monitor_rect);
    }

    Ok((final_x, final_y, final_w, final_h))
}
