use super::resources::{create_pen, create_solid_brush, SafeBrush, SafePen};
use std::collections::HashMap;
use windows::Win32::Graphics::Gdi::PEN_STYLE;
use crate::service::win32::gdiplus::{SafePenWrapper, SafeBrushWrapper, PenWrapper, BrushWrapper};

/// Cache for GDI objects to avoid frequent system calls.
#[derive(Debug, Default)]
pub struct GdiCache {
    pens: HashMap<(u32, i32, u32), SafePen>, // (style, width, color)
    brushes: HashMap<u32, SafeBrush>,        // color

    // GDI+ Resources (Owned RAII wrappers)
    gdiplus_pens: HashMap<(u32, u32, i32), SafePenWrapper>, // (argb, width_bits, dash_style)
    gdiplus_brushes: HashMap<u32, SafeBrushWrapper>,   // argb
}

impl GdiCache {
    pub fn new() -> Self {
        Self {
            pens: HashMap::new(),
            brushes: HashMap::new(),
            gdiplus_pens: HashMap::new(),
            gdiplus_brushes: HashMap::new(),
        }
    }

    pub fn get_pen(
        &mut self,
        style: PEN_STYLE,
        width: i32,
        color: u32,
    ) -> anyhow::Result<&SafePen> {
        self.get_gdi_pen(style.0 as u32, width, color)
    }

    pub fn get_gdi_pen(
        &mut self,
        style_raw: u32,
        width: i32,
        color: u32,
    ) -> anyhow::Result<&SafePen> {
        let key = (style_raw, width, color);
        if !self.pens.contains_key(&key) {
            let pen = create_pen(PEN_STYLE(style_raw as _), width, color)?;
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

    // --- GDI+ Cache Methods ---

    pub fn get_gdiplus_pen(
        &mut self,
        argb: u32,
        width: f32,
        dash_style: Option<windows::Win32::Graphics::GdiPlus::DashStyle>,
    ) -> anyhow::Result<PenWrapper> {
        let dash_val = dash_style.unwrap_or(windows::Win32::Graphics::GdiPlus::DashStyleSolid).0;
        let key = (argb, width.to_bits(), dash_val);
        if !self.gdiplus_pens.contains_key(&key) {
            let pen = PenWrapper::new(argb, width)?;
            if let Some(ds) = dash_style {
                unsafe {
                    windows::Win32::Graphics::GdiPlus::GdipSetPenDashStyle((pen.0).0, ds);
                }
            }
            self.gdiplus_pens.insert(key, pen);
        }
        Ok(self.gdiplus_pens.get(&key).unwrap().0)
    }

    pub fn get_gdiplus_brush(
        &mut self,
        argb: u32,
    ) -> anyhow::Result<BrushWrapper> {
        if !self.gdiplus_brushes.contains_key(&argb) {
            let brush = BrushWrapper::new_solid(argb)?;
            self.gdiplus_brushes.insert(argb, brush);
        }
        Ok(self.gdiplus_brushes.get(&argb).unwrap().0)
    }

    pub fn clear(&mut self) {
        // GDI objects are automatically dropped via SafePen/SafeBrush
        self.pens.clear();
        self.brushes.clear();

        // GDI+ objects are now also automatically dropped via SafePenWrapper/SafeBrushWrapper
        self.gdiplus_pens.clear();
        self.gdiplus_brushes.clear();
    }
}

impl Drop for GdiCache {
    fn drop(&mut self) {
        self.clear();
    }
}
