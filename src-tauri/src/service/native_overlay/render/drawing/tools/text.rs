use super::super::traits::DrawingToolRenderer;
use crate::service::native_overlay::state::DrawingObject;
use crate::service::win32::gdi::{self, SafeHDC};

pub struct TextRenderer;
impl DrawingToolRenderer for TextRenderer {
    fn render(&self, hdc: &SafeHDC, obj: &DrawingObject) -> anyhow::Result<()> {
        if let Some(text) = &obj.text {
            if !text.is_empty() && !obj.points.is_empty() {
                gdi::set_text_color(hdc, obj.color);
                gdi::text_out(hdc, obj.points[0].0, obj.points[0].1, text)?;
            }
        }
        Ok(())
    }
}
