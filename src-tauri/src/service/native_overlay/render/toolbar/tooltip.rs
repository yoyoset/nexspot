use super::types::ToolbarButton;

pub fn draw_tooltip(
    graphics: &crate::service::win32::gdiplus::GraphicsWrapper,
    btn: &ToolbarButton,
    orientation: crate::service::native_overlay::render::toolbar::builder::Orientation,
) -> anyhow::Result<()> {
    let text = &btn.tooltip;
    if text.is_empty() {
        return Ok(());
    }

    use crate::service::win32::gdiplus::{BrushWrapper, PenWrapper};

    // 1. Measure text using GDI+
    let font_family = "Segoe UI";
    let font_size = 14.0;
    let bounding_box =
        crate::service::win32::gdiplus::measure_text(graphics, text, font_family, font_size, None)?;

    let padding_h = 10.0;
    let padding_v = 6.0;
    let tw = bounding_box.Width + padding_h * 2.0;
    let th = bounding_box.Height + padding_v * 2.0;

    let mut tx = btn.rect.left as f32 + (btn.rect.right - btn.rect.left) as f32 / 2.0 - tw / 2.0;
    let mut ty = btn.rect.bottom as f32 + 12.0;

    if orientation
        == crate::service::native_overlay::render::toolbar::builder::Orientation::Vertical
    {
        tx = btn.rect.left as f32 - tw - 12.0;
        ty = btn.rect.top as f32 + (btn.rect.bottom - btn.rect.top) as f32 / 2.0 - th / 2.0;
    }

    // 2. Draw Background
    let bg_brush = BrushWrapper::new_solid(0xE61A1A1A)?; // Slightly transparent black
    let border_pen = PenWrapper::new(0xFF444444, 1.0)?;

    crate::service::win32::gdiplus::fill_rounded_rectangle(
        graphics,
        &bg_brush,
        (tx, ty, tw, th),
        6.0,
    )?;

    crate::service::win32::gdiplus::draw_rounded_rectangle(
        graphics,
        &border_pen,
        (tx, ty, tw, th),
        6.0,
    )?;

    // 3. Draw Text
    let text_brush = BrushWrapper::new_solid(0xFFFFFFFF)?;
    crate::service::win32::gdiplus::draw_text_centered(
        graphics,
        text,
        (tx + tw / 2.0, ty + th / 2.0),
        font_family,
        font_size,
        &text_brush,
        None,
    )?;

    Ok(())
}
