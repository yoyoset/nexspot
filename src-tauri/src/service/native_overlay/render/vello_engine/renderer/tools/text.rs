use crate::service::native_overlay::state::DrawingObject;

use super::{VelloRenderContext, VelloToolRenderer};

pub struct TextRenderer;

impl VelloToolRenderer for TextRenderer {
    fn render(&self, _ctx: &mut VelloRenderContext, obj: &DrawingObject) {
        if obj.points.is_empty() {
            return;
        }
        let text = obj.text.as_deref().unwrap_or("");
        if text.is_empty() && !obj.is_editing_text {
            return;
        }

        let scene = &mut _ctx.scene;
        let pos = obj.points[0];
        let font_size = obj.font_size;
        
        // Use a space for measurement if text is empty but we're editing
        let layout_text = if text.is_empty() { " " } else { text };
        
        let mut builder = _ctx.layout_context.ranged_builder(_ctx.font_context, layout_text, 1.0, false);
        builder.push_default(parley::style::StyleProperty::FontSize(font_size));
        
        // Convert u32 ARGB to parley brush [r, g, b, a]
        let a = ((obj.color >> 24) & 0xff) as u8;
        let r = ((obj.color >> 16) & 0xff) as u8;
        let g = ((obj.color >> 8) & 0xff) as u8;
        let b = (obj.color & 0xff) as u8;
        builder.push_default(parley::style::StyleProperty::Brush([r, g, b, a]));

        let layout: parley::Layout<[u8; 4]> = {
            let mut layout = builder.build(layout_text);
            layout.break_all_lines(None);
            layout
        };

        // 1. Draw Actual Text (if not empty)
        if !text.is_empty() {
            crate::service::native_overlay::render::vello_engine::renderer::utils::text::draw_layout_to_scene(
                scene,
                &layout,
                vello::kurbo::Affine::translate((pos.0 as f64, pos.1 as f64)),
                &vello::peniko::Brush::Solid(vello::peniko::Color::from_rgba8(r, g, b, a)),
            );
        }

        // 2. Draw Editor Decorations (Cursor & Box)
        if obj.is_editing_text {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();
            let show_cursor = (now / 500) % 2 == 0;

            if show_cursor {
                if let Some(last_line) = layout.lines().last() {
                    let mut cursor_x = layout.width() as f64;
                    // If text was empty (placeholder space used), cursor_x should be near 0
                    if text.is_empty() {
                        cursor_x = 0.0;
                    }

                    let cursor_y = last_line.metrics().baseline as f64;
                    
                    let p0 = vello::kurbo::Point::new(
                        pos.0 as f64 + cursor_x,
                        pos.1 as f64 + cursor_y - last_line.metrics().ascent as f64,
                    );
                    let p1 = vello::kurbo::Point::new(
                        pos.0 as f64 + cursor_x,
                        pos.1 as f64 + cursor_y + last_line.metrics().descent as f64,
                    );
                    scene.stroke(
                        &vello::kurbo::Stroke::new(1.5),
                        vello::kurbo::Affine::IDENTITY,
                        &vello::peniko::Brush::Solid(vello::peniko::Color::from_rgba8(r, g, b, a)),
                        None,
                        &vello::kurbo::Line::new(p0, p1),
                    );
                }
            }

            // Draw dashed box
            let padding = 4.0;
            let box_w = if text.is_empty() { 12.0 } else { layout.width() as f64 };
            let box_h = layout.height() as f64;
            
            let rect = vello::kurbo::Rect::new(
                pos.0 as f64 - padding,
                pos.1 as f64 - padding,
                pos.0 as f64 + box_w + padding,
                pos.1 as f64 + box_h + padding,
            );
            
            let dash_pattern = [4.0, 4.0];
            let stroke = vello::kurbo::Stroke::new(1.0).with_dashes(0.0, dash_pattern);
            
            scene.stroke(
                &stroke,
                vello::kurbo::Affine::IDENTITY,
                &vello::peniko::Brush::Solid(vello::peniko::Color::from_rgba8(0, 191, 255, 255)), // DeepSkyBlue
                None,
                &rect,
            );
        }
    }
}
