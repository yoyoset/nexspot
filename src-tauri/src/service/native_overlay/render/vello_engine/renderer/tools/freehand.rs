use crate::service::native_overlay::state::DrawingObject;
use vello::kurbo::BezPath;

use super::{VelloRenderContext, VelloToolRenderer};

pub struct BrushRenderer;

impl VelloToolRenderer for BrushRenderer {
    fn render(&self, ctx: &mut VelloRenderContext, obj: &DrawingObject) {
        if obj.points.len() < 2 {
            return;
        }

        let scene = &mut ctx.scene;
        let mut path = BezPath::new();
        let pts = &obj.points;
        path.move_to((pts[0].0 as f64, pts[0].1 as f64));

        if pts.len() == 2 {
            path.line_to((pts[1].0 as f64, pts[1].1 as f64));
        } else {
            for i in 1..pts.len() - 1 {
                let p_curr = (pts[i].0 as f64, pts[i].1 as f64);
                let p_next = (pts[i + 1].0 as f64, pts[i + 1].1 as f64);
                let mid = ((p_curr.0 + p_next.0) / 2.0, (p_curr.1 + p_next.1) / 2.0);
                path.quad_to(p_curr, mid);
            }
            // Last segment
            path.line_to((pts[pts.len() - 1].0 as f64, pts[pts.len() - 1].1 as f64));
        }
        crate::service::native_overlay::render::vello_engine::renderer::utils::styles::apply_aesthetic_style(scene, &path, obj);
    }
}
