use crate::service::native_overlay::render::vello_engine::renderer::utils::{
    points_to_rect, styles::apply_aesthetic_style,
};
use crate::service::native_overlay::state::{DrawingObject, DrawingTool};
use vello::kurbo::{Ellipse, Line};

use super::{VelloRenderContext, VelloToolRenderer};

pub struct ShapesRenderer;

impl VelloToolRenderer for ShapesRenderer {
    fn render(&self, ctx: &mut VelloRenderContext, obj: &DrawingObject) {
        let scene = &mut ctx.scene;
        match obj.tool {
            DrawingTool::Rect => {
                if obj.points.len() >= 2 {
                    let rect = points_to_rect(obj);
                    apply_aesthetic_style(scene, &rect, obj);
                }
            }
            DrawingTool::Ellipse => {
                if obj.points.len() >= 2 {
                    let rect = points_to_rect(obj);
                    let ellipse = Ellipse::from_rect(rect);
                    apply_aesthetic_style(scene, &ellipse, obj);
                }
            }
            DrawingTool::Line => {
                if obj.points.len() >= 2 {
                    let p1 = vello::kurbo::Point::new(obj.points[0].0 as f64, obj.points[0].1 as f64);
                    let p2 = vello::kurbo::Point::new(obj.points[1].0 as f64, obj.points[1].1 as f64);
                    let line = Line::new(p1, p2);
                    apply_aesthetic_style(scene, &line, obj);
                }
            }
            _ => {}
        }
    }
}

