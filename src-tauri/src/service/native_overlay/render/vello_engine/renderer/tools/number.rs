use crate::service::native_overlay::state::DrawingObject;
use vello::kurbo::Circle;

use super::{VelloRenderContext, VelloToolRenderer};

pub struct NumberRenderer;

impl VelloToolRenderer for NumberRenderer {
    fn render(&self, ctx: &mut VelloRenderContext, obj: &DrawingObject) {
        if obj.points.is_empty() {
            return;
        }

        let scene = &mut ctx.scene;
        let center = obj.points[0];
        let circle = Circle::new((center.0 as f64, center.1 as f64), 14.0);

        // 1. Draw the Background Circle/Bubble (Forced Solid)
        let mut filled_obj = obj.clone();
        filled_obj.is_filled = true;
        crate::service::native_overlay::render::vello_engine::renderer::utils::styles::apply_aesthetic_style(scene, &circle, &filled_obj);

        // 2. Render the Number Text
        if let Some(text) = &obj.text {
            let font_size = 16.0;
            
            let mut builder = ctx.layout_context.ranged_builder(ctx.font_context, text, 1.0, false);
            builder.push_default(parley::style::StyleProperty::FontSize(font_size));
            builder.push_default(parley::style::StyleProperty::Brush([255, 255, 255, 255]));

            let layout: parley::Layout<[u8; 4]> = {
                let mut layout = builder.build(text);
                layout.break_all_lines(None);
                layout
            };

            let text_width = layout.width();
            let text_height = layout.height();

            // Center the text in the circle
            let tx = center.0 as f64 - (text_width as f64 / 2.0);
            let ty = center.1 as f64 - (text_height as f64 / 2.0);

            crate::service::native_overlay::render::vello_engine::renderer::utils::text::draw_layout_to_scene(
                scene,
                &layout,
                vello::kurbo::Affine::translate((tx, ty)),
                &vello::peniko::Brush::Solid(vello::peniko::Color::WHITE),
            );
        }
    }
}
