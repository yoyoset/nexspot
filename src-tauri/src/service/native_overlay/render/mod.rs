use crate::service::native_overlay::state::OverlayState;
use crate::service::win32;
use windows::Win32::Foundation::{POINT, SIZE};

pub mod drawing;
pub mod magnifier;
pub mod selection;
pub mod toolbar;
pub mod vello_engine;

pub fn render_frame(
    hwnd: &win32::window::SafeHWND,
    app: &tauri::AppHandle,
    state: &mut OverlayState,
    toolbar: &mut toolbar::Toolbar,
    vello_ctx: &Option<std::sync::Arc<vello_engine::VelloContext>>,
) -> anyhow::Result<()> {
    if !state.is_visible {
        return Ok(());
    }

    let width = state.width;
    let height = state.height;

    if state.capture_engine == crate::service::native_overlay::state::CaptureEngine::Wgc {
        if let Some(ctx) = vello_ctx {
            // 0. Ensure Window Style for Vello (DWM Composition)
            // We need to DISABLE WS_EX_LAYERED and ENABLE DWM Blur to allow DirectX Swapchain to show through with transparency
            let _ = win32::window::set_layered_attribute(hwnd, false);
            let _ = win32::window::enable_transparency_composition(hwnd);

            // 1. Update Vello Scene from State & Toolbar
            if let Ok(mut scene) = ctx.scene.lock() {
                vello_engine::renderer::render_state_to_scene(state, toolbar, &mut scene);
            }

            // 2. Execute Vello Render to Surface
            if let Err(e) = ctx.render(hwnd.0, width as u32, height as u32) {
                log::error!("Vello render failed: {:?}", e);
            }

            // Note: We DO NOT call UpdateLayeredWindow here because we are using DWM composition now.
            // Calling UpdateLayeredWindow with WS_EX_LAYERED off would fail or fail to produce transparency.
        }
        return Ok(());
    }

    // Ensure Window Style for GDI (Layered Window)
    let _ = win32::window::set_layered_attribute(hwnd, true);

    // 1. Prepare Backbuffer
    // We get a screen DC just for creating the compatible bitmap and for update_layered_window reference
    let hdc_screen = win32::gdi::get_dc(None)?;
    let hbm_buffer = win32::gdi::create_compatible_bitmap(&hdc_screen, width, height)?;
    let hdc_mem = win32::gdi::create_compatible_dc(Some(&hdc_screen))?;
    let prev_hbm_buffer = win32::gdi::select_object(
        &hdc_mem,
        windows::Win32::Graphics::Gdi::HGDIOBJ(hbm_buffer.0 .0),
    )?;

    // 0. DO NOT use SetWindowOrgEx. It can cause bugs on memory DCs in multi-monitor setups.
    // Instead, we explicitly map all global coordinates to device coordinates by subtracting state.x and state.y.

    // 2. Draw Background
    if let Some(hbm_dim) = &state.gdi.hbitmap_dim {
        let hdc_src = win32::gdi::create_compatible_dc(Some(&hdc_screen))?;
        let prev_hbm_src = win32::gdi::select_object(
            &hdc_src,
            windows::Win32::Graphics::Gdi::HGDIOBJ(hbm_dim.0 .0),
        )?;

        win32::gdi::bit_blt(
            &hdc_mem,
            0, // Device X
            0, // Device Y
            width,
            height,
            &hdc_src,
            0, // Device Src X
            0, // Device Src Y
            windows::Win32::Graphics::Gdi::SRCCOPY,
        )?;

        win32::gdi::select_object(&hdc_src, prev_hbm_src)?;
    }

    // 3. Highlight Selection (Cutout)
    if let Some(sel) = state.selection {
        let sw = sel.right - sel.left;
        let sh = sel.bottom - sel.top;
        if sw > 0 && sh > 0 {
            if let Some(hbm_bright) = &state.gdi.hbitmap_bright {
                let hdc_src = win32::gdi::create_compatible_dc(None)?;
                let prev = win32::gdi::select_object(
                    &hdc_src,
                    windows::Win32::Graphics::Gdi::HGDIOBJ(hbm_bright.0 .0),
                )?;
                win32::gdi::bit_blt(
                    &hdc_mem,
                    sel.left - state.x, // Device
                    sel.top - state.y,  // Device
                    sw,
                    sh,
                    &hdc_src,
                    sel.left - state.x, // Device mapping on src bitmap
                    sel.top - state.y,  // Device mapping on src bitmap
                    windows::Win32::Graphics::Gdi::SRCCOPY,
                )?;
                win32::gdi::select_object(&hdc_src, prev)?;
            }

            // Draw Selection Border and Handles
            // Since draw_selection_overlay uses global coordinates in sel, we must shift hdc_mem origin temporarily for it
            unsafe {
                let _ = windows::Win32::Graphics::Gdi::SetWindowOrgEx(
                    hdc_mem.0, state.x, state.y, None,
                );
            }
            selection::draw_selection_overlay(&hdc_mem, &sel, state)?;
            unsafe {
                let _ = windows::Win32::Graphics::Gdi::SetWindowOrgEx(hdc_mem.0, 0, 0, None);
            }
        }
    }

    // Establish Global Logical Space for GDI UI & Objects
    // After performing device-bound BitBlt operations above, we set WindowOrg back to state.x, state.y.
    // This allows objects, toolbar, and magnifier to draw using their existing global coordinates
    // without needing manual `- state.x` everywhere.
    unsafe {
        let _ = windows::Win32::Graphics::Gdi::SetWindowOrgEx(hdc_mem.0, state.x, state.y, None);
    }

    // 4. Draw Drawing Objects
    drawing::draw_all_objects(&hdc_mem, state)?;

    // 5. Draw UI Elements
    toolbar.draw(
        &hdc_mem,
        app,
        state.current_color,
        state.current_font_size,
        state.current_stroke,
        state.current_is_filled,
        state.current_opacity,
        state.current_glow,
    )?;

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
        magnifier::draw_magnifier(&hdc_mem, state.mouse_x, state.mouse_y, state)?;
    }

    // 6. Draw Custom Brush/Mosaic Circle Preview
    if !is_over_toolbar
        && matches!(
            state.current_tool,
            crate::service::native_overlay::state::DrawingTool::Brush
                | crate::service::native_overlay::state::DrawingTool::Mosaic
        )
    {
        // Only show preview if inside selection (if selection exists)
        let is_in_selection = if let Some(sel) = state.selection {
            state.mouse_x >= sel.left
                && state.mouse_x <= sel.right
                && state.mouse_y >= sel.top
                && state.mouse_y <= sel.bottom
        } else {
            true // No selection yet, allow everywhere (though drawing usually needs selection)
        };

        if is_in_selection {
            let radius = if state.current_tool
                == crate::service::native_overlay::state::DrawingTool::Brush
            {
                (state.current_stroke / 2.0).max(2.0) as i32
            } else {
                20 // Mosaic size matches renderer
            };

            let pen = win32::gdi::create_pen(windows::Win32::Graphics::Gdi::PS_SOLID, 1, 0xFFFFFF)?; // White outline
            let old_p = win32::gdi::select_object(
                &hdc_mem,
                windows::Win32::Graphics::Gdi::HGDIOBJ(pen.0 .0),
            )?;

            let hollow_brush =
                win32::gdi::get_stock_object(windows::Win32::Graphics::Gdi::HOLLOW_BRUSH)?;
            let old_b = win32::gdi::select_object(&hdc_mem, hollow_brush)?;

            let _ = win32::gdi::ellipse(
                &hdc_mem,
                state.mouse_x - radius,
                state.mouse_y - radius,
                state.mouse_x + radius,
                state.mouse_y + radius,
            );

            win32::gdi::select_object(&hdc_mem, old_b)?;
            win32::gdi::select_object(&hdc_mem, old_p)?;
        }
    }

    // 7. Restore Origin before UpdateLayeredWindow
    // Since UpdateLayeredWindow respects WindowOrg, we must reset it to (0,0)
    // so pptSrc={x:0, y:0} maps perfectly to Device={0,0} of hbm_buffer.
    unsafe {
        let _ = windows::Win32::Graphics::Gdi::SetWindowOrgEx(hdc_mem.0, 0, 0, None);
    }

    // 8. Update Layered Window
    let update_res = win32::window::update_layered_window(
        hwnd,
        &hdc_mem,
        &windows::Win32::Foundation::POINT {
            x: state.x,
            y: state.y,
        },
        &windows::Win32::Foundation::SIZE {
            cx: width,
            cy: height,
        },
        255,
        0,
    );

    // Cleanup: IMPORTANT to avoid GDI handle leaks
    let _ = win32::gdi::select_object(&hdc_mem, prev_hbm_buffer);
    win32::gdi::release_dc(None, hdc_screen);
    // Note: SafeHDC and SafeHBITMAP will delete their handles when dropped

    update_res
}
