use super::super::traits::DrawingToolRenderer;
use crate::service::native_overlay::state::DrawingObject;
use crate::service::win32::gdi::SafeHDC;
use crate::service::win32::gdiplus;

pub struct TextRenderer;
impl DrawingToolRenderer for TextRenderer {
    fn render(
        &self,
        hdc: &SafeHDC,
        graphics: Option<&crate::service::win32::gdiplus::GraphicsWrapper>,
        _src_hdc: Option<&SafeHDC>,
        cache: &mut crate::service::win32::gdi::GdiCache,
        obj: &DrawingObject,
    ) -> anyhow::Result<()> {
        if let Some(g) = graphics {
            self.render_gdiplus(g, cache, obj)
        } else {
            let g = crate::service::win32::gdiplus::GraphicsWrapper::new(hdc.0)?;
            self.render_gdiplus(&g, cache, obj)
        }
    }

    fn render_gdiplus(
        &self,
        graphics: &crate::service::win32::gdiplus::GraphicsWrapper,
        cache: &mut crate::service::win32::gdi::GdiCache,
        obj: &DrawingObject,
    ) -> anyhow::Result<()> {
        let mut display_text = obj.text.clone().unwrap_or_default();
        if obj.is_editing_text {
            // Blink cursor: |
            let show_cursor = (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
                / 500)
                % 2
                == 0;
            if show_cursor {
                display_text.push('|');
            } else {
                display_text.push(' '); // Keep width stable
            }
        }

        if !display_text.is_empty() && !obj.points.is_empty() {
            let argb = obj.color | 0xFF000000;
            let brush = cache.get_gdiplus_brush(argb)?;

            // Use Bold by default as requested
            let style = Some(windows::Win32::Graphics::GdiPlus::FontStyleBold);

            // If editing, draw a bright blue dashed orientation box
            if obj.is_editing_text {
                let actual_text = obj.text.as_deref().unwrap_or("");
                let text_for_measure = if actual_text.is_empty() {
                    " "
                } else {
                    actual_text
                };
                if let Ok(bounds) = gdiplus::measure_text(
                    graphics,
                    text_for_measure,
                    &obj.font_family,
                    obj.font_size,
                    style,
                ) {
                    let pen = cache.get_gdiplus_pen(
                        0xff00bfff,
                        1.0,
                        Some(windows::Win32::Graphics::GdiPlus::DashStyleDash),
                    )?; 

                    let box_x = obj.points[0].0 as f32 + bounds.X;
                    let box_y = obj.points[0].1 as f32 + bounds.Y;

                    gdiplus::draw_rectangle(
                        graphics,
                        &pen,
                        box_x - 4.0,
                        box_y - 4.0,
                        bounds.Width + 8.0,
                        bounds.Height + 8.0,
                    )?;
                }
            }

            gdiplus::draw_text(
                graphics,
                &display_text,
                (obj.points[0].0 as f32, obj.points[0].1 as f32),
                &obj.font_family,
                obj.font_size,
                &brush,
                style,
                None, // No layout rect
            )?;
        }
        Ok(())
    }
}
