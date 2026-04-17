use crate::service::native_overlay::state::OverlayState;
use crate::service::native_overlay::manager::MonitorRenderContext;
use crate::service::win32;

pub mod drawing;
pub mod magnifier;
pub mod selection;
pub mod toolbar;
pub mod vello_engine;

pub fn render_frame(
    hwnd: &win32::window::SafeHWND,
    app: &tauri::AppHandle,
    state: &mut OverlayState,
    ctx: &mut MonitorRenderContext,
    toolbar: &mut toolbar::Toolbar,
    vello_ctx: &Option<std::sync::Arc<vello_engine::VelloContext>>,
    monitor_id: &str,
    monitor_rect: windows::Win32::Foundation::RECT,
    vello_scene: Option<&vello::Scene>,
) -> anyhow::Result<()> {
    if !state.is_visible {
        return Ok(());
    }

    let win_w = monitor_rect.right - monitor_rect.left;
    let win_h = monitor_rect.bottom - monitor_rect.top;

    // --- 0. Update Toolbar Layout (ONLY for Vello or if explicit) ---
    // Note: For GDI multi-monitor, toolbar layout is updated in the input handler 
    // to avoid per-monitor clamping conflicts.
    if state.capture_engine == crate::service::native_overlay::state::CaptureEngine::Wgc {
        if let Some(sel) = state.selection {
            toolbar.update_layout(
                sel,
                monitor_rect.left,
                monitor_rect.top,
                win_w,
                win_h,
                state.enable_advanced_effects,
                state.capture_engine,
            );
        }
    }

    if state.capture_engine == crate::service::native_overlay::state::CaptureEngine::Wgc {
        if let Some(ctx) = vello_ctx {
            // 0. Ensure Window Style for Vello (DWM Composition)
            let _ = win32::window::set_layered_attribute(hwnd, false);
            let _ = win32::window::enable_transparency_composition(hwnd);

            // 1. Scene building is now handled at the Frame level in actions.rs 
            // to ensure "One Frame, One Scene" performance across multiple monitors.

            // 2. Execute Vello Render to Surface
            if let Some(scene) = vello_scene {
                if let Err(e) = ctx.render(hwnd.0, monitor_id, win_w as u32, win_h as u32, scene) {
                    log::error!("Vello render failed: {:?}", e);
                }
            }
        }
        return Ok(());
    }

    // Ensure Window Style for GDI (Layered Window)
    let _ = win32::window::set_layered_attribute(hwnd, true);

    // 1. Prepare/Reuse Backbuffer (PREMIUM PERFORMANCE)
    let hdc_screen = win32::gdi::get_dc(None)?;
    
    // Check if we need to re-initialize the backbuffer
    let needs_reinit = ctx.hdc_backbuffer.is_none() 
        || ctx.hbm_backbuffer.is_none()
        || ctx.backbuffer_size != (win_w, win_h);

    if needs_reinit {
        log::debug!("Re-initializing GDI backbuffer for monitor {} (size: {}x{})", monitor_id, win_w, win_h);
        let hbm_buffer = win32::gdi::create_compatible_bitmap(&hdc_screen, win_w, win_h)?;
        let hdc_mem = win32::gdi::create_compatible_dc(Some(&hdc_screen))?;
        
        ctx.hbm_backbuffer = Some(hbm_buffer);
        ctx.hdc_backbuffer = Some(hdc_mem);
        ctx.backbuffer_size = (win_w, win_h);
    }

    let hdc_mem_raw = ctx.hdc_backbuffer.as_ref().unwrap().0;
    let hbm_buffer_raw = ctx.hbm_backbuffer.as_ref().unwrap().0;

    // Create a non-owning wrapper to avoid persistent borrow of state
    let hdc_mem = win32::gdi::SafeHDC(hdc_mem_raw, win32::gdi::Disposer::None);

    let prev_hbm_buffer = win32::gdi::select_object(
        &hdc_mem,
        windows::Win32::Graphics::Gdi::HGDIOBJ(hbm_buffer_raw.0),
    )?;

    // 2. Draw Background
    if let Some(hbm_dim) = &state.gdi.hbitmap_dim {
        // OPTIMIZATION: Reuse background source DC
        if ctx.hdc_bg_src.is_none() {
            ctx.hdc_bg_src = Some(win32::gdi::create_compatible_dc(Some(&hdc_screen))?);
        }
        let hdc_src = ctx.hdc_bg_src.as_ref().unwrap();
        
        let prev_hbm_src = win32::gdi::select_object(
            hdc_src,
            windows::Win32::Graphics::Gdi::HGDIOBJ(hbm_dim.0 .0),
        )?;

        let src_x = monitor_rect.left - state.capture_x;
        let src_y = monitor_rect.top - state.capture_y;

        win32::gdi::bit_blt(
            &hdc_mem,
            0, // Device X
            0, // Device Y
            win_w,
            win_h,
            hdc_src,
            src_x,
            src_y,
            windows::Win32::Graphics::Gdi::SRCCOPY,
        )?;

        win32::gdi::select_object(hdc_src, prev_hbm_src)?;
    }

    // 3. Highlight Selection (Cutout)
    if let Some(sel) = state.selection {
        // --- SEAMLESS CROSS-SCREEN LOGIC ---
        // Calculate intersection between Global Selection and this Monitor
        let draw_left = sel.left.max(monitor_rect.left);
        let draw_right = sel.right.min(monitor_rect.right);
        let draw_top = sel.top.max(monitor_rect.top);
        let draw_bottom = sel.bottom.min(monitor_rect.bottom);

        let sw = draw_right - draw_left;
        let sh = draw_bottom - draw_top;

        if sw > 0 && sh > 0 {
            if let Some(hbm_bright) = &state.gdi.hbitmap_bright {
                // OPTIMIZATION: Use hdc_selection_src as a generic scratch DC for high-frequency BitBlts
                if ctx.hdc_selection_src.is_none() {
                    ctx.hdc_selection_src = Some(win32::gdi::create_compatible_dc(None)?);
                }
                let hdc_src = ctx.hdc_selection_src.as_ref().unwrap();

                let prev = win32::gdi::select_object(
                    hdc_src,
                    windows::Win32::Graphics::Gdi::HGDIOBJ(hbm_bright.0 .0),
                )?;

                // Map absolute intersection to local space for both Target and Source
                let local_target_x = draw_left - monitor_rect.left;
                let local_target_y = draw_top - monitor_rect.top;
                let local_source_x = draw_left - state.capture_x; 
                let local_source_y = draw_top - state.capture_y;

                win32::gdi::bit_blt(
                    &hdc_mem,
                    local_target_x,
                    local_target_y,
                    sw,
                    sh,
                    hdc_src,
                    local_source_x,
                    local_source_y,
                    windows::Win32::Graphics::Gdi::SRCCOPY,
                )?;
                win32::gdi::select_object(hdc_src, prev)?;
            }

            // Draw Selection Border and Handles
            selection::draw_selection_overlay(
                &hdc_mem,
                &sel,
                state,
                &mut ctx.cache,
                monitor_rect.left,
                monitor_rect.top,
            )?;
        }
    }

    // --- PREMIUM PERFORMANCE FIX: Single Context Injection ---
    // Initialize GDI+ context ONCE per frame and share it with tools/UI.
    {
        if ctx.graphics.is_none() {
            ctx.graphics = Some(crate::service::win32::gdiplus::GraphicsWrapper::new(hdc_mem.0)?);
        }
        let graphics = ctx.graphics.as_mut().unwrap();
        
        // --- SYNC COORDINATE SPACE ---
        let _ = graphics.reset_transform();
        // Align GDI+ with Global coordinates by offsetting by the monitor's top-left.
        let _ = graphics.translate(-(monitor_rect.left as f32), -(monitor_rect.top as f32));

        // 4. Draw Drawing Objects
        drawing::draw_all_objects(&hdc_mem, Some(&graphics), state, &mut ctx.cache)?;

        // 5. Draw UI Elements (Toolbar)
        let tb_rect = &toolbar.rect;
        let m_rect = &monitor_rect;
        let intersect_w = (tb_rect.right.min(m_rect.right) - tb_rect.left.max(m_rect.left)).max(0);
        let intersect_h = (tb_rect.bottom.min(m_rect.bottom) - tb_rect.top.max(m_rect.top)).max(0);

        if intersect_w > 0 && intersect_h > 0 {
            toolbar.draw(
                &graphics,
                &hdc_mem,
                app,
                state.current_color,
                state.current_font_size,
                state.current_stroke,
                state.current_is_filled,
                state.current_opacity,
                state.current_glow,
            )?;
        }
    }

    // Draw magnifier logic
    let is_adjusting = matches!(
        state.interaction_mode,
        crate::service::native_overlay::state::InteractionMode::Selecting
            | crate::service::native_overlay::state::InteractionMode::Resizing(_)
    );

    let is_outside = if let Some(sel) = state.selection {
        state.mouse_x < sel.left
            || state.mouse_x > sel.right
            || state.mouse_y < sel.top
            || state.mouse_y > sel.bottom
    } else {
        true
    };

    let is_over_toolbar = state.mouse_x >= toolbar.rect.left
        && state.mouse_x < toolbar.rect.right
        && state.mouse_y >= toolbar.rect.top
        && state.mouse_y < toolbar.rect.bottom;

    if (is_adjusting || is_outside) && !is_over_toolbar {
        magnifier::draw_magnifier(&hdc_mem, state.mouse_x, state.mouse_y, state, ctx)?;
    }

    // 6. Draw Custom Brush/Mosaic Circle Preview
    if !is_over_toolbar
        && matches!(
            state.current_tool,
            crate::service::native_overlay::state::DrawingTool::Brush
                | crate::service::native_overlay::state::DrawingTool::Mosaic
        )
    {
        let is_in_selection = if let Some(sel) = state.selection {
            state.mouse_x >= sel.left
                && state.mouse_x <= sel.right
                && state.mouse_y >= sel.top
                && state.mouse_y <= sel.bottom
        } else {
            true 
        };

        if is_in_selection {
            let radius = if state.current_tool
                == crate::service::native_overlay::state::DrawingTool::Brush
            {
                (state.current_stroke / 2.0).max(2.0) as i32
            } else {
                20 // Mosaic size matches renderer
            };

            let pen = ctx.cache.get_gdi_pen(windows::Win32::Graphics::Gdi::PS_SOLID.0 as _, 1, 0xFFFFFF)?; // White outline
            let old_p = win32::gdi::select_object(
                &hdc_mem,
                windows::Win32::Graphics::Gdi::HGDIOBJ(pen.0 .0),
            )?;

            let hollow_brush =
                win32::gdi::get_stock_object(windows::Win32::Graphics::Gdi::HOLLOW_BRUSH)?;
            let old_b = win32::gdi::select_object(&hdc_mem, hollow_brush)?;

            let _ = win32::gdi::ellipse(
                &hdc_mem,
                state.mouse_x - radius - monitor_rect.left,
                state.mouse_y - radius - monitor_rect.top,
                state.mouse_x + radius - monitor_rect.left,
                state.mouse_y + radius - monitor_rect.top,
            );

            win32::gdi::select_object(&hdc_mem, old_b)?;
            win32::gdi::select_object(&hdc_mem, old_p)?;
        }
    }

    // 7. No Origin Shift needed before UpdateLayeredWindow as pptDst handles it

    // 8. Update Layered Window
    let update_res = win32::window::update_layered_window(
        hwnd,
        &hdc_mem,
        &windows::Win32::Foundation::POINT {
            x: monitor_rect.left,
            y: monitor_rect.top,
        },
        &windows::Win32::Foundation::SIZE {
            cx: win_w,
            cy: win_h,
        },
        255,
        0,
    );

    let _ = win32::gdi::select_object(&hdc_mem, prev_hbm_buffer);
    win32::gdi::release_dc(None, hdc_screen);

    update_res
}
