use crate::service::native_overlay::state::{OverlayState, CaptureEngine};
use crate::service::win32;
use crate::service::win32::gdi::SafeHBITMAP;
use std::sync::{Arc, RwLock};
use tauri::{AppHandle, Manager};
use windows::Win32::Foundation::RECT;

pub struct RenderedSnapshot {
    pub hbitmap: SafeHBITMAP,
    pub width: i32,
    pub height: i32,
}

pub fn render_snapshot(
    state_arc: &Arc<RwLock<OverlayState>>,
    app: &AppHandle,
) -> anyhow::Result<RenderedSnapshot> {
    let (sel, capture_engine, canvas_w, canvas_h, cap_x, cap_y) = {
        let state = state_arc.read().map_err(|_| anyhow::anyhow!("State lock poisoned"))?;
        (
            state.selection.unwrap_or(RECT {
                left: 0,
                top: 0,
                right: state.width,
                bottom: state.height,
            }),
            state.capture_engine,
            state.width,
            state.height,
            state.capture_x,
            state.capture_y,
        )
    };

    let width = sel.right - sel.left;
    let height = sel.bottom - sel.top;
    if width <= 0 || height <= 0 {
        anyhow::bail!("Invalid selection size");
    }

    // 1. Vello / WGC Path
    if capture_engine == CaptureEngine::Wgc {
        let vello_ctx_opt = app.state::<crate::AppState>().overlay_manager.lock().ok().and_then(|mgr| mgr.vello_ctx.clone());
        if let Some(ctx) = vello_ctx_opt {
            let mut scene = vello::Scene::new();
            {
                use crate::service::native_overlay::render::vello_engine::renderer::render_state_to_scene;
                if let Ok(mut state_lock) = state_arc.write() {
                    state_lock.finalize_all_objects();
                    render_state_to_scene(&state_lock, None, &ctx, &mut scene);
                }
            }
            
            // 不能直接 block_on：本函数可能在 tokio worker 上被调（OCR/滚动），
            // runtime 内嵌套 block_on 会 panic。改为 spawn 到 runtime + 通道同步等待，
            // 在任意线程上下文都安全（普通线程/UI 线程/worker 均可）。
            let image_data = {
                let (tx, rx) = std::sync::mpsc::channel();
                let ctx_task = ctx.clone();
                let (w, h) = (canvas_w as u32, canvas_h as u32);
                tauri::async_runtime::spawn(async move {
                    let res = ctx_task.render_to_image(w, h, &scene).await;
                    let _ = tx.send(res);
                });
                rx.recv()
                    .map_err(|_| anyhow::anyhow!("Offscreen render channel closed"))??
            };

            let full_pixels = image_data.data.as_ref();
            let row_bytes = width as usize * 4;
            let mut cp = Vec::with_capacity(row_bytes * height as usize);
            let stride = (canvas_w * 4) as usize;
            // 选区为世界坐标，画布原点为 (cap_x, cap_y)；转换为画布局部坐标后再索引，
            // 避免负坐标 (多显示器) 经 `as usize` 回绕导致乘法溢出崩溃。
            let local_left = sel.left - cap_x;
            let local_top = sel.top - cap_y;
            for y in 0..height {
                let src_y = local_top + y;
                let mut wrote = false;
                if src_y >= 0 && (src_y as i64) < canvas_h as i64 && local_left >= 0 {
                    let row_start = src_y as usize * stride + local_left as usize * 4;
                    if row_start + row_bytes <= full_pixels.len() {
                        cp.extend_from_slice(&full_pixels[row_start..row_start + row_bytes]);
                        wrote = true;
                    }
                }
                if !wrote {
                    // 行越界 → 补透明，保证 cp 尺寸恒为 width*height*4
                    cp.extend(std::iter::repeat(0u8).take(row_bytes));
                }
            }
            // Swap BGRA for GDI compatibility and set alpha to 255
            for chunk in cp.chunks_exact_mut(4) {
                chunk.swap(0, 2);
                chunk[3] = 255;
            }
            
            let hdc_screen = win32::gdi::get_dc(None)?;
            let hbm = win32::gdi::create_bitmap_from_pixels(&hdc_screen, width, height, &cp)?;
            win32::gdi::release_dc(None, hdc_screen);
            
            return Ok(RenderedSnapshot { hbitmap: hbm, width, height });
        }
    }

    // 2. GDI Fallback Path
    let vello_bg_data = {
        let state = state_arc.read().unwrap_or_else(|e| e.into_inner());
        state.vello.background.clone()
    };

    let temp_hbm_bright = if let Some(vello_img) = vello_bg_data {
        let hdc_screen = win32::gdi::get_dc(None)?;
        let mut pixels = vello_img.data.as_ref().to_vec();
        for chunk in pixels.chunks_exact_mut(4) {
            chunk.swap(0, 2);
        }
        let hbm = win32::gdi::create_bitmap_from_pixels(&hdc_screen, canvas_w, canvas_h, &pixels)?;
        win32::gdi::release_dc(None, hdc_screen);
        Some(hbm)
    } else {
        None
    };

    let hbm_crop = {
        let mut state = state_arc.write().unwrap_or_else(|e| e.into_inner());
        state.finalize_all_objects();

        let hbm_source = if temp_hbm_bright.is_some() {
            temp_hbm_bright.as_ref()
        } else {
            state.gdi.hbitmap_bright.as_ref()
        };

        if let Some(hbm_bright) = hbm_source {
            let hdc_screen = win32::gdi::get_dc(None)?;
            let hdc_mem_comp = win32::gdi::create_compatible_dc(Some(&hdc_screen))?;
            let hbm_comp = win32::gdi::create_compatible_bitmap(&hdc_screen, canvas_w, canvas_h)?;
            let prev_comp = win32::gdi::select_object(&hdc_mem_comp, windows::Win32::Graphics::Gdi::HGDIOBJ(hbm_comp.0.0))?;

            let hdc_mem_src = win32::gdi::create_compatible_dc(Some(&hdc_screen))?;
            let prev_src = win32::gdi::select_object(&hdc_mem_src, windows::Win32::Graphics::Gdi::HGDIOBJ(hbm_bright.0 .0))?;
            win32::gdi::bit_blt(&hdc_mem_comp, 0, 0, canvas_w, canvas_h, &hdc_mem_src, 0, 0, windows::Win32::Graphics::Gdi::SRCCOPY)?;
            win32::gdi::select_object(&hdc_mem_src, prev_src)?;


            {
                let mut cache = crate::service::win32::gdi::GdiCache::default();
                let graphics = crate::service::win32::gdiplus::GraphicsWrapper::new(hdc_mem_comp.0)?;
                
                let (cx, cy) = (state.capture_x, state.capture_y);

                // Align GDI+ with Global coordinates by offsetting by the capture origin.
                let _ = graphics.translate(-(cx as f32), -(cy as f32));
                
                crate::service::native_overlay::render::drawing::draw_all_objects(
                    &hdc_mem_comp, 
                    Some(&graphics), 
                    &mut state, 
                    &mut cache,
                    cx,
                    cy,
                )?;
            }


            let hdc_mem_dst = win32::gdi::create_compatible_dc(Some(&hdc_screen))?;
            let hbm_crop = win32::gdi::create_compatible_bitmap(&hdc_screen, width, height)?;
            let prev_dst = win32::gdi::select_object(&hdc_mem_dst, windows::Win32::Graphics::Gdi::HGDIOBJ(hbm_crop.0.0))?;

            let src_x = sel.left - state.capture_x;
            let src_y = sel.top - state.capture_y;
            win32::gdi::bit_blt(&hdc_mem_dst, 0, 0, width, height, &hdc_mem_comp, src_x, src_y, windows::Win32::Graphics::Gdi::SRCCOPY)?;
            
            win32::gdi::select_object(&hdc_mem_comp, prev_comp)?;
            win32::gdi::select_object(&hdc_mem_dst, prev_dst)?;
            
            hbm_crop
        } else {
            anyhow::bail!("No source bitmap available");
        }
    };

    Ok(RenderedSnapshot { hbitmap: hbm_crop, width, height })
}

pub fn update_snapshot_size_config(state_arc: &Arc<RwLock<OverlayState>>, app: &AppHandle) {
    let (sel, active_wf, is_snapshot) = {
        let state = match state_arc.read() {
            Ok(s) => s,
            Err(_) => return,
        };
        (
            state.selection.unwrap_or(RECT {
                left: 0,
                top: 0,
                right: state.width,
                bottom: state.height,
            }),
            state.active_workflow.clone(),
            state.is_snapshot_mode,
        )
    };

    if is_snapshot {
        if let Some(workflow) = &active_wf {
            if let crate::service::config::types::CaptureAction::Snapshot { allow_resize, .. } = workflow.action {
                if allow_resize {
                    let sw = sel.right - sel.left;
                    let sh = sel.bottom - sel.top;
                    if sw > 0 && sh > 0 {
                        let app_inner = app.clone();
                        let workflow_id = workflow.id.clone();
                        let _ = app.run_on_main_thread(move || {
                            let app_state = app_inner.state::<crate::AppState>();
                            let mut c_state = app_state.config_state.lock().unwrap_or_else(|e| e.into_inner());
                            let mut changed = false;

                            // 1. Global Sync (Last Adjusted)
                            if c_state.config.snapshot_width != sw || c_state.config.snapshot_height != sh {
                                c_state.config.snapshot_width = sw;
                                c_state.config.snapshot_height = sh;
                                changed = true;
                            }

                            // 2. Workflow Specific Sync
                            if let Some(w) = c_state.config.workflows.iter_mut().find(|w| w.id == workflow_id) {
                                if let crate::service::config::types::CaptureAction::Snapshot { width, height, .. } = &mut w.action {
                                    if *width != sw || *height != sh {
                                        *width = sw;
                                        *height = sh;
                                        changed = true;
                                    }
                                }
                            }

                            if changed {
                                c_state.save();
                                
                                // 3. CRITICAL: Refresh hotkey map in AppState
                                // The hotkey listener uses a cached map in AppState. If we don't refresh it,
                                // the next Alt+S will still use the old workflow/config data.
                                let new_map = c_state.register_all();
                                if let Ok(mut map_lock) = app_state.hotkey_map.write() {
                                    *map_lock = new_map;
                                    log::info!("[ConfigSync] Hotkey map refreshed with new snapshot dimensions.");
                                }
                            }
                        });
                    }
                }
            }
        }
    }
}
