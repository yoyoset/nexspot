pub mod tools;
pub mod traits;

use crate::service::native_overlay::state::{DrawingObject, DrawingTool, OverlayState};
use crate::service::win32::gdi::{self, SafeHDC};
use traits::DrawingToolRenderer;
use windows::Win32::Foundation::RECT;

struct ClippingGuard<'a> {
    hdc: &'a SafeHDC,
    rgn: Option<windows::Win32::Graphics::Gdi::HRGN>,
}

impl<'a> ClippingGuard<'a> {
    fn new(hdc: &'a SafeHDC, selection: Option<RECT>) -> Self {
        if let Some(sel) = selection {
            let rgn = unsafe {
                windows::Win32::Graphics::Gdi::CreateRectRgn(
                    sel.left, sel.top, sel.right, sel.bottom,
                )
            };
            if !rgn.is_invalid() {
                unsafe {
                    let _ = windows::Win32::Graphics::Gdi::SelectClipRgn(hdc.0, Some(rgn));
                }
                return Self {
                    hdc,
                    rgn: Some(rgn),
                };
            }
        }
        Self { hdc, rgn: None }
    }
}

impl<'a> Drop for ClippingGuard<'a> {
    fn drop(&mut self) {
        if let Some(rgn) = self.rgn {
            unsafe {
                let _ = windows::Win32::Graphics::Gdi::SelectClipRgn(self.hdc.0, None);
                let _ = windows::Win32::Graphics::Gdi::DeleteObject(
                    windows::Win32::Graphics::Gdi::HGDIOBJ(rgn.0),
                );
            }
        }
    }
}

pub fn draw_all_objects(hdc: &SafeHDC, state: &mut OverlayState) -> anyhow::Result<()> {
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
            src_hdc_opt.as_ref().map(|(s, _)| s),
            &mut state.gdi.cache,
            obj,
        )?;
    }

    // 4. Draw current interaction (preview)
    if let Some(current) = &state.current_drawing {
        draw_object(
            hdc,
            src_hdc_opt.as_ref().map(|(s, _)| s),
            &mut state.gdi.cache,
            current,
        )?;
    }

    // 5. Draw selection handles for the selected object
    if let Some(idx) = state.selected_object_index {
        if let Some(obj) = state.objects.get(idx) {
            let bounds = obj.get_bounds();
            crate::service::native_overlay::render::selection::draw_handles(hdc, &bounds, state)?;
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
        let null_pen = gdi::create_pen(windows::Win32::Graphics::Gdi::PS_NULL, 0, 0)?;
        gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(null_pen.0 .0))?
    };

    // 2. Setup Brush
    let old_brush = if use_brush {
        let brush = cache.get_brush(obj.color)?;
        gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(brush.0 .0))?
    } else {
        let null_brush = unsafe {
            windows::Win32::Graphics::Gdi::GetStockObject(windows::Win32::Graphics::Gdi::NULL_BRUSH)
        };
        gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(null_brush.0))?
    };

    // Dispatch to specific renderer
    let result = match obj.tool {
        DrawingTool::Rect => tools::RectRenderer.render(hdc, src_hdc, obj),
        DrawingTool::Ellipse => tools::EllipseRenderer.render(hdc, src_hdc, obj),
        DrawingTool::Line => tools::LineRenderer.render(hdc, src_hdc, obj),
        DrawingTool::Arrow => tools::ArrowRenderer.render(hdc, src_hdc, obj),
        DrawingTool::Brush => tools::BrushRenderer.render(hdc, src_hdc, obj),
        DrawingTool::Mosaic => tools::MosaicRenderer.render(hdc, src_hdc, obj),
        DrawingTool::Text => tools::TextRenderer.render(hdc, src_hdc, obj),
        DrawingTool::Number => tools::NumberRenderer.render(hdc, src_hdc, obj),
        _ => Ok(()),
    };

    // Restore GDI state
    gdi::select_object(hdc, old_brush)?;
    gdi::select_object(hdc, old_pen)?;

    result
}
