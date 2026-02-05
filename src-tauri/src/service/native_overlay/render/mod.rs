use crate::service::native_overlay::state::OverlayState;
use crate::service::win32;
use windows::Win32::Foundation::{POINT, SIZE};

pub mod drawing;
pub mod magnifier;
pub mod selection;
pub mod toolbar;

pub fn render_frame(
    hwnd: &win32::window::SafeHWND,
    state: &mut OverlayState,
    toolbar: &mut toolbar::Toolbar,
) -> anyhow::Result<()> {
    if !state.is_visible {
        return Ok(());
    }

    let width = state.width;
    let height = state.height;

    // 1. Prepare Backbuffer
    // We get a screen DC just for creating the compatible bitmap and for update_layered_window reference
    let hdc_screen = win32::gdi::get_dc(None)?;
    let hbm_buffer = win32::gdi::create_compatible_bitmap(&hdc_screen, width, height)?;
    let hdc_mem = win32::gdi::create_compatible_dc(Some(&hdc_screen))?;
    let prev_hbm_buffer = win32::gdi::select_object(
        &hdc_mem,
        windows::Win32::Graphics::Gdi::HGDIOBJ(hbm_buffer.0 .0),
    )?;

    // 2. Draw Background
    if let Some(hbm_dim) = &state.hbitmap_dim {
        let hdc_src = win32::gdi::create_compatible_dc(Some(&hdc_screen))?;
        let prev_hbm_src = win32::gdi::select_object(
            &hdc_src,
            windows::Win32::Graphics::Gdi::HGDIOBJ(hbm_dim.0 .0),
        )?;

        win32::gdi::bit_blt(
            &hdc_mem,
            0,
            0,
            width,
            height,
            &hdc_src,
            0,
            0,
            windows::Win32::Graphics::Gdi::SRCCOPY,
        )?;

        win32::gdi::select_object(&hdc_src, prev_hbm_src)?;
    }

    // 3. Highlight Selection (Cutout)
    if let Some(sel) = state.selection {
        let sw = sel.right - sel.left;
        let sh = sel.bottom - sel.top;
        if sw > 0 && sh > 0 {
            if let Some(hbm_bright) = &state.hbitmap_bright {
                let hdc_src = win32::gdi::create_compatible_dc(None)?;
                let prev = win32::gdi::select_object(
                    &hdc_src,
                    windows::Win32::Graphics::Gdi::HGDIOBJ(hbm_bright.0 .0),
                )?;
                win32::gdi::bit_blt(
                    &hdc_mem,
                    sel.left,
                    sel.top,
                    sw,
                    sh,
                    &hdc_src,
                    sel.left,
                    sel.top,
                    windows::Win32::Graphics::Gdi::SRCCOPY,
                )?;
                win32::gdi::select_object(&hdc_src, prev)?;
            }

            // Draw Selection Border and Handles
            selection::draw_selection_overlay(&hdc_mem, &sel, state)?;
        }
    }

    // 4. Draw Drawing Objects (NEW)
    drawing::draw_all_objects(&hdc_mem, state)?;

    // 5. Draw UI Elements
    toolbar.draw(&hdc_mem)?;

    // Draw magnifier logic:
    // 1. Always show when selecting or resizing (to help precision)
    // 2. Otherwise, show only if mouse is OUTSIDE the selection area
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
        true // No selection yet, show it immediately
    };

    let is_over_toolbar = state.mouse_x >= toolbar.rect.left
        && state.mouse_x < toolbar.rect.right
        && state.mouse_y >= toolbar.rect.top
        && state.mouse_y < toolbar.rect.bottom;

    if (is_adjusting || is_outside) && !is_over_toolbar {
        magnifier::draw_magnifier(&hdc_mem, state.mouse_x, state.mouse_y, state)?;
    }

    // 5. Update Layered Window
    win32::window::update_layered_window(
        hwnd,
        &hdc_mem,
        &POINT {
            x: state.x,
            y: state.y,
        },
        &SIZE {
            cx: width,
            cy: height,
        },
        255,
        0,
    )?;

    // Cleanup
    win32::gdi::select_object(&hdc_mem, prev_hbm_buffer)?;

    Ok(())
}
