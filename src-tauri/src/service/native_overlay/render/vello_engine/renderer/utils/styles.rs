use vello::kurbo::{Affine, Shape, Stroke};
use vello::peniko::{Brush, Fill};
use vello::Scene;
use crate::service::native_overlay::state::DrawingObject;
use crate::service::config::types::AestheticStyle;
use crate::service::native_overlay::render::vello_engine::renderer::utils::argb_to_vello;

/// Defines the rendering behavior for a single visual layer.
#[derive(Debug, Clone)]
struct RenderLayer {
    /// Overrides the primary color with a specific one (e.g. White for Neon core).
    color_override: Option<vello::peniko::Color>,
    /// Multiplies the primary color's alpha.
    alpha_mul: f32,
    /// Multiplies the stroke width.
    width_mul: f64,
    /// If Some, forces Fill or Stroke regardless of object state.
    force_fill: Option<bool>,
    /// Array of offsets for jitter/sketch effects. (0.0, 0.0) if empty.
    offsets: Vec<(f64, f64)>,
    /// Special effect marker (e.g. 1 for Glass Frosted)
    effect: u8,
}

impl Default for RenderLayer {
    fn default() -> Self {
        Self {
            color_override: None,
            alpha_mul: 1.0,
            width_mul: 1.0,
            force_fill: None,
            offsets: vec![(0.0, 0.0)],
            effect: 0,
        }
    }
}

/// A collection of layers that define a complete visual style.
#[derive(Debug, Clone)]
struct StyleSpec {
    layers: Vec<RenderLayer>,
}

/// Applies the standardized aesthetic style to a given shape using the Spec Engine.
pub fn apply_aesthetic_style<S: Shape>(scene: &mut Scene, shape: &S, obj: &DrawingObject) {
    let spec = get_style_spec(obj.style);
    
    for layer in &spec.layers {
        let base_color = layer.color_override.unwrap_or_else(|| argb_to_vello(obj.color));
        let alpha = if layer.color_override.is_some() { 1.0 } else { layer.alpha_mul };
        let rgba = base_color.to_rgba8();
        let color = if alpha < 1.0 {
             vello::peniko::Color::from_rgba8(rgba.r, rgba.g, rgba.b, (rgba.a as f32 * alpha) as u8)
        } else {
             base_color
        };
        
        let brush = Brush::Solid(color);
        let stroke_width = (obj.stroke_width as f64 * layer.width_mul).max(1.0);
        let mut stroke = Stroke::new(stroke_width);
        if obj.is_dashed {
            stroke = stroke.with_dashes(10.0, [10.0, 5.0]);
        }

        let is_filled = layer.force_fill.unwrap_or(obj.is_filled);

        for (dx, dy) in &layer.offsets {
            let affine = Affine::translate((*dx, *dy));
            
            // Handle special effects
            if layer.effect == 1 { // Glass Frosted BG
                let glass_brush = Brush::Solid(vello::peniko::Color::from_rgba8(255, 255, 255, 30));
                scene.fill(Fill::NonZero, affine, &glass_brush, None, shape);
                continue;
            } else if layer.effect == 2 { // Glass Highlight Edge
                let highlight_brush = Brush::Solid(vello::peniko::Color::from_rgba8(255, 255, 255, 80));
                scene.stroke(&Stroke::new(1.0), affine, &highlight_brush, None, shape);
                continue;
            }

            if is_filled {
                scene.fill(Fill::NonZero, affine, &brush, None, shape);
            } else {
                scene.stroke(&stroke, affine, &brush, None, shape);
            }
        }
    }
}

/// The Registry: Maps AestheticStyle to its parameter-based Specification.
fn get_style_spec(style: AestheticStyle) -> StyleSpec {
    match style {
        AestheticStyle::Default => StyleSpec {
            layers: vec![RenderLayer::default()],
        },
        AestheticStyle::Neon => StyleSpec {
            layers: vec![
                // 1. Glow
                RenderLayer {
                    alpha_mul: 0.4,
                    width_mul: 2.5,
                    force_fill: Some(false),
                    ..Default::default()
                },
                // 2. White Core
                RenderLayer {
                    color_override: Some(vello::peniko::Color::WHITE),
                    width_mul: 0.4,
                    ..Default::default()
                },
            ],
        },
        AestheticStyle::PaperCut => StyleSpec {
            layers: vec![
                // 1. BG Tint
                RenderLayer {
                    alpha_mul: 0.2,
                    force_fill: Some(true),
                    ..Default::default()
                },
                // 2. Main
                RenderLayer::default(),
            ],
        },
        AestheticStyle::Sketch => StyleSpec {
            layers: vec![
                // 1. First Pass: Light Draft Jitter
                RenderLayer {
                    alpha_mul: 0.35,
                    width_mul: 0.7,
                    force_fill: Some(false),
                    offsets: vec![(-2.2, 1.8), (2.8, -1.2)],
                    ..Default::default()
                },
                // 2. Second Pass: Inner Scratch
                RenderLayer {
                    alpha_mul: 0.45,
                    width_mul: 1.1,
                    force_fill: Some(false),
                    offsets: vec![(0.8, 1.2), (-1.2, -1.5)],
                    ..Default::default()
                },
                // 3. Third Pass: Main Pencil Line
                RenderLayer {
                    alpha_mul: 0.75,
                    width_mul: 0.9,
                    ..Default::default()
                },
            ],
        },
        AestheticStyle::Glass => StyleSpec {
            layers: vec![
                // 1. Frosted BG Effect
                RenderLayer { effect: 1, ..Default::default() },
                // 2. Highlight Edge Effect
                RenderLayer { effect: 2, ..Default::default() },
                // 3. Main/Boundary
                RenderLayer {
                    alpha_mul: 0.5, // Used only if filled in Glass mode typically
                    ..Default::default()
                },
            ],
        },
    }
}
