use crate::service::native_overlay::state::DrawingObject;
use vello::kurbo::Affine;

use super::{VelloRenderContext, VelloToolRenderer};

pub struct MosaicRenderer;

impl VelloToolRenderer for MosaicRenderer {
    fn render(&self, ctx: &mut VelloRenderContext, obj: &DrawingObject) {
        let scene = &mut ctx.scene;

        if obj.mosaic_blocks.is_empty() {
            return;
        }

        // The block_size stored in the object should match the one used during sampling.
        let block_size = match obj.stroke_width as i32 {
            0..=3 => 6.0,
            4..=7 => 10.0,
            _ => 16.0,
        };

        for ((gx, gy), color_u32) in &obj.mosaic_blocks {
            let x = *gx as f64 * block_size;
            let y = *gy as f64 * block_size;

            let rect = vello::kurbo::Rect::new(x, y, x + block_size, y + block_size);
            
            let color = vello::peniko::Color::from_rgba8(
                ((color_u32 >> 16) & 0xFF) as u8,
                ((color_u32 >> 8) & 0xFF) as u8,
                (color_u32 & 0xFF) as u8,
                ((color_u32 >> 24) & 0xFF) as u8,
            );

            scene.fill(
                vello::peniko::Fill::NonZero,
                Affine::IDENTITY,
                color,
                None,
                &rect,
            );
        }
    }
}
