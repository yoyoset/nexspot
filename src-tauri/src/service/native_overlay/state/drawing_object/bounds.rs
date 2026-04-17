use super::DrawingObject;
use windows::Win32::Foundation::RECT;
use crate::service::native_overlay::state::types::DrawingTool;

impl DrawingObject {
    pub fn get_bounds(&self) -> RECT {
        if self.points.is_empty() {
            return RECT::default();
        }

        if matches!(self.tool, DrawingTool::Text) {
            let p = self.points[0];
            let font_size = self.font_size;
            let text_content = self.text.as_deref().unwrap_or("");
            // Rough estimation of text width: average char width is about 0.6 * font_size for Segoe UI
            let width = (text_content.len().max(1) as f32 * font_size * 0.6) as i32;
            let height = font_size as i32;

            return RECT {
                left: p.0,
                top: p.1,
                right: p.0 + width,
                bottom: p.1 + height,
            };
        }

        if self.tool == DrawingTool::Mosaic && !self.mosaic_blocks.is_empty() {
            let block_size = match self.stroke_width as i32 {
                0..=3 => 6,
                4..=7 => 10,
                _ => 16,
            };
            let mut min_gx = i32::MAX;
            let mut min_gy = i32::MAX;
            let mut max_gx = i32::MIN;
            let mut max_gy = i32::MIN;
            for (gx, gy) in self.mosaic_blocks.keys() {
                min_gx = min_gx.min(*gx);
                min_gy = min_gy.min(*gy);
                max_gx = max_gx.max(*gx);
                max_gy = max_gy.max(*gy);
            }
            return RECT {
                left: min_gx * block_size,
                top: min_gy * block_size,
                right: (max_gx + 1) * block_size,
                bottom: (max_gy + 1) * block_size,
            };
        }

        if self.tool == DrawingTool::Number && !self.points.is_empty() {
            let p = self.points[0];
            let radius = 18; // Standard hit-area for Number circle
            return RECT {
                left: p.0 - radius,
                top: p.1 - radius,
                right: p.0 + radius,
                bottom: p.1 + radius,
            };
        }

        let mut min_x = self.points[0].0;
        let mut min_y = self.points[0].1;
        let mut max_x = self.points[0].0;
        let mut max_y = self.points[0].1;

        for p in &self.points {
            min_x = min_x.min(p.0);
            min_y = min_y.min(p.1);
            max_x = max_x.max(p.0);
            max_y = max_y.max(p.1);
        }

        // Add stroke margin for Brush or small tools
        let margin = if self.tool == DrawingTool::Brush {
            (self.stroke_width / 2.0).max(1.0) as i32 + 2
        } else {
            2
        };

        RECT {
            left: min_x - margin,
            top: min_y - margin,
            right: max_x + margin,
            bottom: max_y + margin,
        }
    }

    pub fn get_raw_bounds(&self) -> RECT {
        if self.points.is_empty() {
            return RECT::default();
        }

        let mut min_x = self.points[0].0;
        let mut min_y = self.points[0].1;
        let mut max_x = self.points[0].0;
        let mut max_y = self.points[0].1;

        for p in &self.points {
            min_x = min_x.min(p.0);
            min_y = min_y.min(p.1);
            max_x = max_x.max(p.0);
            max_y = max_y.max(p.1);
        }

        RECT {
            left: min_x,
            top: min_y,
            right: max_x,
            bottom: max_y,
        }
    }
}
