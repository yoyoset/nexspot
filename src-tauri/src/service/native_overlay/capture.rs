use crate::service::native_overlay::state::OverlayState;
use crate::service::win32;
use std::sync::{Arc, Mutex};
use windows::Win32::Foundation::RECT;

pub fn perform_capture(
    state: &Arc<Mutex<OverlayState>>,
    _stream_states: Option<
        Arc<
            Mutex<
                std::collections::HashMap<
                    usize,
                    Arc<Mutex<crate::service::win32::wgc::capture::StreamState>>,
                >,
            >,
        >,
    >,
) -> anyhow::Result<(i32, i32, i32, i32)> {
    let start = std::time::Instant::now();
    let monitors = win32::monitor::enumerate_monitors()?;
    log::info!("Monitors enumerated in {:?}", start.elapsed());

    // ... (lines 14-91 remain the same)
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

    let width = union_rect.right - union_rect.left;
    let height = union_rect.bottom - union_rect.top;
    let x = union_rect.left;
    let y = union_rect.top;
    log::info!("Virtual Screen: {}x{} at ({}, {})", width, height, x, y);

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

    let mut target_monitor_index = 0;
    let mut target_monitor_rect = monitors
        .iter()
        .find(|m| m.is_primary)
        .map(|m| m.rect)
        .unwrap_or(union_rect);
    let mut target_monitor_name = monitors
        .iter()
        .find(|m| m.is_primary)
        .map(|m| m.name.clone())
        .unwrap_or_default();
    let mut target_monitor_friendly_name = monitors
        .iter()
        .find(|m| m.is_primary)
        .map(|m| m.friendly_name.clone())
        .unwrap_or_default();

    if let Some(pos) = monitors.iter().position(|m| m.is_primary) {
        target_monitor_index = pos;
    }

    for (i, m) in monitors.iter().enumerate() {
        if cursor.x >= m.rect.left
            && cursor.x < m.rect.right
            && cursor.y >= m.rect.top
            && cursor.y < m.rect.bottom
        {
            target_monitor_rect = m.rect;
            target_monitor_name = m.name.clone();
            target_monitor_friendly_name = m.friendly_name.clone();
            target_monitor_index = i;
            break;
        }
    }

    let engine = {
        let s = match state.lock() {
            Ok(s) => s,
            Err(_) => return Err(anyhow::anyhow!("State mutex poisoned")),
        };
        s.capture_engine
    };

    match engine {
        crate::service::native_overlay::state::CaptureEngine::Gdi => {
            log::info!("Starting GDI Mode (Virtual Screen Capture)");
            let t_cap = std::time::Instant::now();
            let hdc_screen = win32::gdi::get_dc(None)?;
            let hdc_mem = win32::gdi::create_compatible_dc(Some(&hdc_screen))?;
            let hbm_screen = win32::gdi::create_compatible_bitmap(&hdc_screen, width, height)?;
            let prev_hbm_screen = win32::gdi::select_object(
                &hdc_mem,
                windows::Win32::Graphics::Gdi::HGDIOBJ(hbm_screen.0 .0),
            )?;

            win32::gdi::bit_blt(
                &hdc_mem,
                0,
                0,
                width,
                height,
                &hdc_screen,
                x,
                y,
                windows::Win32::Graphics::Gdi::SRCCOPY,
            )?;

            log::info!("GDI BitBlt took {:?}", t_cap.elapsed());

            // 3. Pre-render Dimmed Background
            let t_dim = std::time::Instant::now();
            let hdc_dim = win32::gdi::create_compatible_dc(Some(&hdc_screen))?;
            let hbm_dim = win32::gdi::create_compatible_bitmap(&hdc_screen, width, height)?;
            let prev_hbm_dim = win32::gdi::select_object(
                &hdc_dim,
                windows::Win32::Graphics::Gdi::HGDIOBJ(hbm_dim.0 .0),
            )?;

            // Copy Screen to Dim
            win32::gdi::bit_blt(
                &hdc_dim,
                0,
                0,
                width,
                height,
                &hdc_mem,
                0,
                0,
                windows::Win32::Graphics::Gdi::SRCCOPY,
            )?;

            // Fast Darken: Hardware Accelerated AlphaBlend (GDI)
            {
                let hdc_black = win32::gdi::create_compatible_dc(None)?;
                let hbm_black = win32::gdi::create_compatible_bitmap(&hdc_screen, width, height)?;
                let prev_hbm_black = win32::gdi::select_object(
                    &hdc_black,
                    windows::Win32::Graphics::Gdi::HGDIOBJ(hbm_black.0 .0),
                )?;

                let brush = win32::gdi::create_solid_brush(0x000000)?;
                win32::gdi::fill_rect(
                    &hdc_black,
                    &RECT {
                        left: 0,
                        top: 0,
                        right: width,
                        bottom: height,
                    },
                    &brush,
                );

                unsafe {
                    use windows::Win32::Graphics::Gdi::{
                        GdiAlphaBlend, AC_SRC_OVER, BLENDFUNCTION,
                    };
                    let blend = BLENDFUNCTION {
                        BlendOp: AC_SRC_OVER as u8,
                        BlendFlags: 0,
                        SourceConstantAlpha: 120, // ~47% Darken
                        AlphaFormat: 0,
                    };

                    let _ = GdiAlphaBlend(
                        hdc_dim.0,
                        0,
                        0,
                        width,
                        height,
                        hdc_black.0,
                        0,
                        0,
                        width,  // Full width match (no stretching)
                        height, // Full height match (no stretching)
                        blend,
                    );
                }
                win32::gdi::select_object(&hdc_black, prev_hbm_black)?;
            }

            // Restore & Cleanup
            win32::gdi::select_object(&hdc_mem, prev_hbm_screen)?;
            win32::gdi::select_object(&hdc_dim, prev_hbm_dim)?;
            win32::gdi::release_dc(None, hdc_screen);

            log::info!("GDI Dimming took {:?}", t_dim.elapsed());

            // Enumerate Windows for Snapping
            let t_wc = std::time::Instant::now();
            let snap_rects = win32::window::enumerate_visible_windows();
            log::info!(
                "Enumerated {} Windows in {:?}",
                snap_rects.len(),
                t_wc.elapsed()
            );

            // 4. Update State
            if let Ok(mut s) = state.lock() {
                s.x = x;
                s.y = y;
                s.width = width;
                s.height = height;
                s.gdi.hbitmap_bright = Some(hbm_screen);
                s.gdi.hbitmap_dim = Some(hbm_dim);
                s.window_rects = snap_rects;
                s.is_visible = true;
                s.selection = None;
                s.is_capturing = false; // Reset capturing flag
                s.vello.background = None; // Reset Vello data in GDI mode
                s.restrict_to_monitor = if has_mixed_dpi {
                    Some(target_monitor_rect)
                } else {
                    None
                };
            }
        }
        crate::service::native_overlay::state::CaptureEngine::Wgc => {
            log::info!("Starting WGC Mode (Strict Separation + Single Monitor Focus)");

            // Snapshot mode specific check: ensure we really have a monitor if snapshotting
            log::info!(
                "WGC Target Monitor: {} (Friendly: {})",
                target_monitor_name,
                target_monitor_friendly_name
            );

            let mut captured_via_stream = false;
            // Stream optimization
            if let Some(states_map_arc) = _stream_states {
                log::info!(
                    "[WGC DIAG] Stream states map provided. Trying monitor index {}...",
                    target_monitor_index
                );
                if let Ok(map) = states_map_arc.lock() {
                    log::info!(
                        "[WGC DIAG] Stream map has {} entries. Keys: {:?}",
                        map.len(),
                        map.keys().collect::<Vec<_>>()
                    );
                    let ss_opt = map.get(&target_monitor_index);
                    if let Some(ss) = ss_opt {
                        if let Ok(lock) = ss.lock() {
                            log::info!("[WGC DIAG] Stream state for idx {}: is_alive={}, has_image={}, size={:?}", 
                                target_monitor_index, lock.is_alive, lock.image.is_some(), lock.size);
                            if lock.is_alive {
                                if let Some(img) = lock.image.clone() {
                                    log::info!(
                                        "[WGC DIAG] ✓ Using STREAM frame: {}x{}",
                                        img.width,
                                        img.height
                                    );
                                    if let Ok(mut s) = state.lock() {
                                        s.vello.background = Some(img);
                                        captured_via_stream = true;
                                    }
                                } else {
                                    log::warn!(
                                        "[WGC DIAG] Stream alive but NO image yet for idx {}",
                                        target_monitor_index
                                    );
                                }
                            } else {
                                log::warn!("[WGC DIAG] Stream for monitor {} is DEAD! Falling back to 1-shot.", target_monitor_index);
                            }
                        }
                    } else {
                        log::warn!(
                            "[WGC DIAG] No stream entry for monitor index {}!",
                            target_monitor_index
                        );
                    }
                }
            } else {
                log::warn!("[WGC DIAG] No stream states map provided at all! wgc_stream is None.");
            }

            if !captured_via_stream {
                log::info!(
                    "[WGC DIAG] Attempting One-shot capture for Monitor idx {}...",
                    target_monitor_index
                );
                match crate::service::win32::wgc::capture::capture_monitor_to_vello(
                    target_monitor_index,
                    &target_monitor_name,
                    &target_monitor_friendly_name,
                    Some(target_monitor_rect),
                ) {
                    Ok((img, size)) => {
                        log::info!(
                            "[WGC DIAG] ✓ One-shot captured: {}x{} (size tuple: {:?})",
                            img.width,
                            img.height,
                            size
                        );
                        if let Ok(mut s) = state.lock() {
                            s.vello.background = Some(img);
                        }
                    }
                    Err(e) => {
                        log::error!("[WGC DIAG] ✗ One-shot capture FAILED: {:?}", e);
                    }
                }
            }

            let final_x = target_monitor_rect.left;
            let final_y = target_monitor_rect.top;
            let final_w = target_monitor_rect.right - target_monitor_rect.left;
            let final_h = target_monitor_rect.bottom - target_monitor_rect.top;

            let snap_rects = win32::window::enumerate_visible_windows();

            if let Ok(mut s) = state.lock() {
                s.x = final_x;
                s.y = final_y;
                s.width = final_w;
                s.height = final_h;
                s.mouse_x = cursor.x;
                s.mouse_y = cursor.y;
                s.window_rects = snap_rects;
                s.is_visible = true;
                s.selection = None;
                s.is_capturing = false; // Reset capturing flag
                s.gdi.hbitmap_bright = None; // Reset GDI data in Vello mode
                s.gdi.hbitmap_dim = None;
            }

            return Ok((final_x, final_y, final_w, final_h));
        }
    }

    // Fallback reset
    if let Ok(mut s) = state.lock() {
        s.is_capturing = false;
    }

    log::info!("Total perform_capture took {:?}", start.elapsed());
    Ok((x, y, width, height))
}
