pub mod tools;
pub mod traits;

use crate::service::native_overlay::state::{DrawingObject, DrawingTool, OverlayState};
use crate::service::win32::gdi::{self, SafeHDC, GdiCache};
use traits::DrawingToolRenderer;
use windows::Win32::Foundation::RECT;

struct ClippingGuard<'a> {
    hdc: &'a SafeHDC,
    saved_state: i32,
}

impl<'a> ClippingGuard<'a> {
    fn new(hdc: &'a SafeHDC, selection: Option<RECT>) -> Self {
        let saved_state = unsafe { windows::Win32::Graphics::Gdi::SaveDC(hdc.0) };
        if let Some(sel) = selection {
            unsafe {
                // IntersectClipRect uses logical coordinates.
                // Since draw_all_objects is called with a DC that has SetWindowOrgEx offset,
                // passing global coordinates here works perfectly as long as they are matched by GDI.
                let _ = windows::Win32::Graphics::Gdi::IntersectClipRect(
                    hdc.0, sel.left, sel.top, sel.right, sel.bottom,
                );
            }
        }
        Self { hdc, saved_state }
    }
}

impl<'a> Drop for ClippingGuard<'a> {
    fn drop(&mut self) {
        unsafe {
            let _ = windows::Win32::Graphics::Gdi::RestoreDC(self.hdc.0, self.saved_state);
        }
    }
}

pub fn draw_all_objects(
    hdc: &SafeHDC,
    graphics: Option<&crate::service::win32::gdiplus::GraphicsWrapper>,
    state: &mut OverlayState,
    cache: &mut GdiCache,
) -> anyhow::Result<()> {
    // 1. Setup Clipping to Selection (RAII)
    let _guard = ClippingGuard::new(hdc, state.selection);

    // 2. Prepare Source DC for sampling (used by Mosaic)
    let mut src_hdc_opt = None;
    if let Some(hbm) = &state.gdi.hbitmap_bright {
        if let Ok(sdc) = gdi::create_compatible_dc(Some(hdc)) {
            if let Ok(old) =
                gdi::select_object(&sdc, windows::Win32::Graphics::Gdi::HGDIOBJ(hbm.0 .0))
            {
                src_hdc_opt = Some((sdc, old));
            }
        }
    }

    // 3. Draw committed objects
    for obj in &state.objects {
        draw_object(
            hdc,
            graphics,
            src_hdc_opt.as_ref().map(|(s, _)| s),
            cache,
            obj,
        )?;
    }

    // 4. Draw current interaction (preview)
    if let Some(current) = &state.current_drawing {
        draw_object(
            hdc,
            graphics,
            src_hdc_opt.as_ref().map(|(s, _)| s),
            cache,
            current,
        )?;
    }

    // 5. Draw selection handles for the selected object
    if let Some(idx) = state.selected_object_index {
        if let Some(obj) = state.objects.get(idx) {
            let bounds = obj.get_bounds();
            crate::service::native_overlay::render::selection::draw_handles(
                hdc,
                &bounds,
                state,
                cache,
                state.monitor_rect.left,
                state.monitor_rect.top,
            )?;
        }
    }

    // Cleanup Source DC
    if let Some((sdc, old)) = src_hdc_opt {
        let _ = gdi::select_object(&sdc, old);
    }

    Ok(())
}

fn draw_object(
    hdc: &SafeHDC,
    graphics: Option<&crate::service::win32::gdiplus::GraphicsWrapper>,
    src_hdc: Option<&SafeHDC>,
    cache: &mut gdi::cache::GdiCache,
    obj: &DrawingObject,
) -> anyhow::Result<()> {
    // Smart Resource Selection based on Tool Type
    let (use_pen, use_brush) = match obj.tool {
        DrawingTool::Rect | DrawingTool::Ellipse => (true, obj.is_filled),
        DrawingTool::Line | DrawingTool::Brush | DrawingTool::Mosaic => (true, false), // Stroke only
        DrawingTool::Arrow => (true, true), // Hybrid (Stroke + Fill)
        DrawingTool::Text => (false, false), // Text handles its own
        DrawingTool::Number => (true, true), // Filled Circle (Brush) + Outline (Pen) or Null Pen
        _ => (true, false),
    };

    // 1. Setup Pen
    let old_pen = if use_pen {
        let pen = cache.get_pen(
            windows::Win32::Graphics::Gdi::PS_SOLID,
            obj.stroke_width as i32,
            obj.color,
        )?;
        gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(pen.0 .0))?
    } else {
        let null_pen = gdi::get_stock_object(windows::Win32::Graphics::Gdi::NULL_PEN)?;
        gdi::select_object(hdc, null_pen)?
    };

    // 2. Setup Brush
    let old_brush = if use_brush {
        let brush = cache.get_brush(obj.color)?;
        gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(brush.0 .0))?
    } else {
        let null_brush = gdi::get_stock_object(windows::Win32::Graphics::Gdi::NULL_BRUSH)?;
        gdi::select_object(hdc, null_brush)?
    };

    // Dispatch to specific renderer
    let result = match obj.tool {
        DrawingTool::Rect => tools::RectRenderer.render(hdc, graphics, src_hdc, cache, obj),
        DrawingTool::Ellipse => tools::EllipseRenderer.render(hdc, graphics, src_hdc, cache, obj),
        DrawingTool::Line => tools::LineRenderer.render(hdc, graphics, src_hdc, cache, obj),
        DrawingTool::Arrow => tools::ArrowRenderer.render(hdc, graphics, src_hdc, cache, obj),
        DrawingTool::Brush => tools::BrushRenderer.render(hdc, graphics, src_hdc, cache, obj),
        DrawingTool::Mosaic => tools::MosaicRenderer.render(hdc, graphics, src_hdc, cache, obj),
        DrawingTool::Text => tools::TextRenderer.render(hdc, graphics, src_hdc, cache, obj),
        DrawingTool::Number => tools::NumberRenderer.render(hdc, graphics, src_hdc, cache, obj),
        _ => Ok(()),
    };

    // Restore GDI state
    gdi::select_object(hdc, old_brush)?;
    gdi::select_object(hdc, old_pen)?;

    result
}
