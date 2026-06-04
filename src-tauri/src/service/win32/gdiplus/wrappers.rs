use windows::Win32::Graphics::GdiPlus::*;
use windows::Win32::Graphics::Gdi::*;
use std::ops::Deref;

#[derive(Debug)]
pub struct GraphicsWrapper(pub *mut GpGraphics);
impl GraphicsWrapper {
    pub fn new(hdc: HDC) -> anyhow::Result<Self> {
        let mut graphics = std::ptr::null_mut();
        unsafe {
            if GdipCreateFromHDC(hdc, &mut graphics) != windows::Win32::Graphics::GdiPlus::Ok {
                return Err(anyhow::anyhow!("GdipCreateFromHDC failed"));
            }
            // Enable anti-aliasing by default
            let _ = GdipSetSmoothingMode(graphics, SmoothingModeAntiAlias);
            let _ = GdipSetTextRenderingHint(graphics, TextRenderingHintAntiAliasGridFit);
        }
        std::result::Result::Ok(Self(graphics))
    }
    
    pub fn get_hdc(&self) -> anyhow::Result<HDC> {
        let mut hdc = HDC::default();
        unsafe {
            if GdipGetDC(self.0, &mut hdc) != windows::Win32::Graphics::GdiPlus::Ok {
                return Err(anyhow::anyhow!("GdipGetDC failed"));
            }
        }
        std::result::Result::Ok(hdc)
    }

    pub fn release_hdc(&self, hdc: HDC) {
        unsafe {
            let _ = GdipReleaseDC(self.0, hdc);
        }
    }

    pub fn translate(&self, dx: f32, dy: f32) -> anyhow::Result<()> {
        unsafe {
            if GdipTranslateWorldTransform(self.0, dx, dy, MatrixOrderPrepend) != windows::Win32::Graphics::GdiPlus::Ok {
                return Err(anyhow::anyhow!("GdipTranslateWorldTransform failed"));
            }
        }
        std::result::Result::Ok(())
    }

    pub fn reset_transform(&self) -> anyhow::Result<()> {
        unsafe {
            if GdipResetWorldTransform(self.0) != windows::Win32::Graphics::GdiPlus::Ok {
                return Err(anyhow::anyhow!("GdipResetWorldTransform failed"));
            }
        }
        std::result::Result::Ok(())
    }

    pub fn set_smoothing_mode(&self, mode: SmoothingMode) -> anyhow::Result<()> {
        unsafe {
            if GdipSetSmoothingMode(self.0, mode) != windows::Win32::Graphics::GdiPlus::Ok {
                return Err(anyhow::anyhow!("GdipSetSmoothingMode failed"));
            }
        }
        std::result::Result::Ok(())
    }

    pub fn set_text_rendering_hint(&self, hint: TextRenderingHint) -> anyhow::Result<()> {
        unsafe {
            if GdipSetTextRenderingHint(self.0, hint) != windows::Win32::Graphics::GdiPlus::Ok {
                return Err(anyhow::anyhow!("GdipSetTextRenderingHint failed"));
            }
        }
        std::result::Result::Ok(())
    }
}

impl Drop for GraphicsWrapper {
    fn drop(&mut self) {
        unsafe {
            let _ = GdipDeleteGraphics(self.0);
        }
    }
}

unsafe impl Send for GraphicsWrapper {}
unsafe impl Sync for GraphicsWrapper {}

// --- POD Handles (Copyable, no Drop) ---

#[derive(Debug, Clone, Copy)]
pub struct PenWrapper(pub *mut GpPen);
impl PenWrapper {
    pub fn new(color: u32, width: f32) -> anyhow::Result<SafePenWrapper> {
        SafePenWrapper::actual_new(color, width)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BrushWrapper(pub *mut GpBrush);
impl BrushWrapper {
    pub fn new_solid(color: u32) -> anyhow::Result<SafeBrushWrapper> {
        SafeBrushWrapper::actual_new_solid(color)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BitmapWrapper(pub *mut GpBitmap);
impl BitmapWrapper {
    pub fn from_hbitmap(hbitmap: HBITMAP) -> anyhow::Result<SafeBitmapWrapper> {
        SafeBitmapWrapper::actual_from_hbitmap(hbitmap)
    }
}

unsafe impl Send for PenWrapper {}
unsafe impl Sync for PenWrapper {}
unsafe impl Send for BrushWrapper {}
unsafe impl Sync for BrushWrapper {}
unsafe impl Send for BitmapWrapper {}
unsafe impl Sync for BitmapWrapper {}

// --- RAII Wrappers (Not Copyable, handles Drop) ---

#[derive(Debug)]
pub struct SafePenWrapper(pub PenWrapper);
impl SafePenWrapper {
    fn actual_new(color: u32, width: f32) -> anyhow::Result<Self> {
        let mut pen = std::ptr::null_mut();
        unsafe {
            if GdipCreatePen1(color, width, UnitPixel, &mut pen) != windows::Win32::Graphics::GdiPlus::Ok {
                return Err(anyhow::anyhow!("GdipCreatePen1 failed"));
            }
        }
        std::result::Result::Ok(Self(PenWrapper(pen)))
    }
}
impl Deref for SafePenWrapper {
    type Target = PenWrapper;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Drop for SafePenWrapper {
    fn drop(&mut self) {
        unsafe {
            let _ = GdipDeletePen((self.0).0);
        }
    }
}

#[derive(Debug)]
pub struct SafeBrushWrapper(pub BrushWrapper);
impl SafeBrushWrapper {
    fn actual_new_solid(color: u32) -> anyhow::Result<Self> {
        let mut solid_brush = std::ptr::null_mut();
        unsafe {
            if GdipCreateSolidFill(color, &mut solid_brush) != windows::Win32::Graphics::GdiPlus::Ok {
                return Err(anyhow::anyhow!("GdipCreateSolidFill failed"));
            }
        }
        std::result::Result::Ok(Self(BrushWrapper(solid_brush as *mut GpBrush)))
    }
}
impl Deref for SafeBrushWrapper {
    type Target = BrushWrapper;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Drop for SafeBrushWrapper {
    fn drop(&mut self) {
        unsafe {
            let _ = GdipDeleteBrush((self.0).0);
        }
    }
}

#[derive(Debug)]
pub struct SafeBitmapWrapper(pub BitmapWrapper);
impl SafeBitmapWrapper {
    fn actual_from_hbitmap(hbitmap: HBITMAP) -> anyhow::Result<Self> {
        let mut bitmap = std::ptr::null_mut();
        unsafe {
            if GdipCreateBitmapFromHBITMAP(hbitmap, HPALETTE::default(), &mut bitmap) != windows::Win32::Graphics::GdiPlus::Ok {
                return Err(anyhow::anyhow!("GdipCreateBitmapFromHBITMAP failed"));
            }
        }
        std::result::Result::Ok(Self(BitmapWrapper(bitmap)))
    }
}
impl Deref for SafeBitmapWrapper {
    type Target = BitmapWrapper;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Drop for SafeBitmapWrapper {
    fn drop(&mut self) {
        unsafe {
            let _ = GdipDisposeImage((self.0).0 as *mut GpImage);
        }
    }
}
