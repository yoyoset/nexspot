use crate::service::native_overlay::render::vello_engine::renderer::utils::{
    argb_to_vello, points_to_rect,
};
use crate::service::native_overlay::state::{DrawingObject, DrawingTool};
use vello::kurbo::{Affine, Ellipse, Line, Stroke};
use vello::peniko::{Brush, Fill};
use vello::Scene;

pub fn render_shape(scene: &mut Scene, obj: &DrawingObject) {
    let color = argb_to_vello(obj.color);
    let brush = Brush::Solid(color);
    let mut stroke = Stroke::new(obj.stroke_width as f64);
    if obj.is_dashed {
        stroke = stroke.with_dashes(10.0, [10.0, 5.0]);
    }

    match obj.tool {
        DrawingTool::Rect => {
            if obj.points.len() >= 2 {
                let rect = points_to_rect(obj.points[0], obj.points[1]);
                if obj.is_filled {
                    scene.fill(Fill::NonZero, Affine::IDENTITY, &brush, None, &rect);
                } else {
                    scene.stroke(&stroke, Affine::IDENTITY, &brush, None, &rect);
                }
            }
        }
        DrawingTool::Ellipse => {
            if obj.points.len() >= 2 {
                let rect = points_to_rect(obj.points[0], obj.points[1]);
                let ellipse = Ellipse::from_rect(rect);
                if obj.is_filled {
                    scene.fill(Fill::NonZero, Affine::IDENTITY, &brush, None, &ellipse);
                } else {
                    scene.stroke(&stroke, Affine::IDENTITY, &brush, None, &ellipse);
                }
            }
        }
        DrawingTool::Line => {
            if obj.points.len() >= 2 {
                let p1 = vello::kurbo::Point::new(obj.points[0].0 as f64, obj.points[0].1 as f64);
                let p2 = vello::kurbo::Point::new(obj.points[1].0 as f64, obj.points[1].1 as f64);
                let line = Line::new(p1, p2);
                scene.stroke(&stroke, Affine::IDENTITY, &brush, None, &line);
            }
        }
        _ => {}
    }
}
