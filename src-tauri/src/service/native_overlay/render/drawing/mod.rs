pub mod tools;
pub mod traits;

use crate::service::native_overlay::state::{DrawingObject, DrawingTool, OverlayState};
use crate::service::win32::gdi::{self, SafeHDC};
use traits::DrawingToolRenderer;

pub fn draw_all_objects(hdc: &SafeHDC, state: &mut OverlayState) -> anyhow::Result<()> {
    // 1. Draw committed objects
    for obj in &state.objects {
        draw_object(hdc, &mut state.gdi_cache, obj)?;
    }

    // 2. Draw current interaction (preview)
    if let Some(current) = &state.current_drawing {
        draw_object(hdc, &mut state.gdi_cache, current)?;
    }

    Ok(())
}

fn draw_object(
    hdc: &SafeHDC,
    cache: &mut gdi::cache::GdiCache,
    obj: &DrawingObject,
) -> anyhow::Result<()> {
    // Setup shared GDI resources (Pen & Stock Brush) from Cache
    let pen = cache.get_pen(
        windows::Win32::Graphics::Gdi::PS_SOLID,
        obj.stroke_width as i32,
        obj.color,
    )?;
    let old_pen = gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(pen.0 .0))?;

    let null_brush = unsafe {
        windows::Win32::Graphics::Gdi::GetStockObject(windows::Win32::Graphics::Gdi::NULL_BRUSH)
    };
    let old_brush = gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(null_brush.0))?;

    // Dispatch to specific renderer
    let result = match obj.tool {
        DrawingTool::Rect => tools::RectRenderer.render(hdc, obj),
        DrawingTool::Ellipse => tools::EllipseRenderer.render(hdc, obj),
        DrawingTool::Line => tools::LineRenderer.render(hdc, obj),
        DrawingTool::Arrow => tools::ArrowRenderer.render(hdc, obj),
        DrawingTool::Brush => tools::BrushRenderer.render(hdc, obj),
        DrawingTool::Mosaic => tools::MosaicRenderer.render(hdc, obj),
        DrawingTool::Text => tools::TextRenderer.render(hdc, obj),
        _ => Ok(()),
    };

    // Restore GDI state
    gdi::select_object(hdc, old_brush)?;
    gdi::select_object(hdc, old_pen)?;

    result
}
