use crate::service::native_overlay::state::OverlayState;
use crate::service::win32;
use std::sync::{Arc, RwLock, Mutex};
use windows::Win32::Foundation::RECT;

pub mod gdi;
pub mod wgc;

pub fn perform_capture(
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
) -> anyhow::Result<(i32, i32, i32, i32)> {
    let start = std::time::Instant::now();
    let monitors = win32::monitor::enumerate_monitors()?;
    log::info!("Monitors enumerated in {:?}", start.elapsed());

    let mut union_rect = RECT::default();
    if let Some(first) = monitors.first() {
        union_rect = first.rect;
    }
    for m in &monitors {
        union_rect.left = union_rect.left.min(m.rect.left);
        union_rect.top = union_rect.top.min(m.rect.top);
        union_rect.right = union_rect.right.max(m.rect.right);
        union_rect.bottom = union_rect.bottom.max(m.rect.bottom);
    }

    // 1. Check for Mixed DPI
    let mut has_mixed_dpi = false;
    if let Some(first) = monitors.first() {
        for m in &monitors {
            if m.dpi_x != first.dpi_x || m.dpi_y != first.dpi_y {
                has_mixed_dpi = true;
                break;
            }
        }
    }

    // 2. Identify Active Monitor from Cursor Position
    let mut cursor = windows::Win32::Foundation::POINT::default();
    unsafe {
        let _ = windows::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut cursor);
    };

    let mut target_monitor_rect = monitors
        .iter()
        .find(|m| m.is_primary)
        .map(|m| m.rect)
        .unwrap_or(union_rect);
    let mut target_monitor_hmonitor = monitors
        .iter()
        .find(|m| m.is_primary)
        .map(|m| m.hmonitor.to_string())
        .unwrap_or_default();
    let mut target_monitor_friendly_name = monitors
        .iter()
        .find(|m| m.is_primary)
        .map(|m| m.friendly_name.clone())
        .unwrap_or_default();

    for m in monitors.iter() {
        if cursor.x >= m.rect.left
            && cursor.x < m.rect.right
            && cursor.y >= m.rect.top
            && cursor.y < m.rect.bottom
        {
            target_monitor_rect = m.rect;
            target_monitor_hmonitor = m.hmonitor.to_string();
            target_monitor_friendly_name = m.friendly_name.clone();
            log::info!(
                "[Capture] Active Monitor Found: {:?}. Rect: {:?}",
                target_monitor_friendly_name,
                target_monitor_rect
            );
            break;
        }
    }

    let engine = {
        let s = match state.read() {
            Ok(s) => s,
            Err(_) => return Err(anyhow::anyhow!("State lock poisoned")),
        };
        s.capture_engine
    };

    let result = match engine {
        crate::service::native_overlay::state::CaptureEngine::Gdi => gdi::capture_gdi(
            state,
            &monitors,
            &target_monitor_rect,
            has_mixed_dpi,
            &union_rect,
            &cursor,
        )?,
        crate::service::native_overlay::state::CaptureEngine::Wgc => wgc::capture_wgc(
            state,
            _stream_states,
            &target_monitor_hmonitor,
            &target_monitor_friendly_name,
            &target_monitor_rect,
            &cursor,
        )?,
    };

    log::info!("Total perform_capture took {:?}", start.elapsed());
    Ok(result)
}
