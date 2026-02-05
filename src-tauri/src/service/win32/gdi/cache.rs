use super::resources::{create_pen, create_solid_brush, SafeBrush, SafePen};
use std::collections::HashMap;
use windows::Win32::Graphics::Gdi::PEN_STYLE;

/// Cache for GDI objects to avoid frequent system calls.
#[derive(Debug)]
pub struct GdiCache {
    pens: HashMap<(u32, i32, u32), SafePen>, // (style, width, color)
    brushes: HashMap<u32, SafeBrush>,        // color
}

impl GdiCache {
    pub fn new() -> Self {
        Self {
            pens: HashMap::new(),
            brushes: HashMap::new(),
        }
    }

    pub fn get_pen(
        &mut self,
        style: PEN_STYLE,
        width: i32,
        color: u32,
    ) -> anyhow::Result<&SafePen> {
        let key = (style.0 as u32, width, color);
        if !self.pens.contains_key(&key) {
            let pen = create_pen(style, width, color)?;
            self.pens.insert(key, pen);
        }
        Ok(self.pens.get(&key).unwrap())
    }

    pub fn get_brush(&mut self, color: u32) -> anyhow::Result<&SafeBrush> {
        if !self.brushes.contains_key(&color) {
            let brush = create_solid_brush(color)?;
            self.brushes.insert(color, brush);
        }
        Ok(self.brushes.get(&color).unwrap())
    }

    pub fn clear(&mut self) {
        self.pens.clear();
        self.brushes.clear();
    }
}
