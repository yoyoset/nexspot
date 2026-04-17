use crate::service::win32::gdi::SafeHBITMAP;
use vello::peniko::ImageData;

#[derive(Debug)]
pub struct GdiData {
    pub hbitmap_dim: Option<SafeHBITMAP>,
    pub hbitmap_bright: Option<SafeHBITMAP>,
    pub bright_pixels: Option<std::sync::Arc<Vec<u32>>>, // Cached pixels for O(1) sampling
    pub gdiplus_bitmap_dim: Option<crate::service::win32::gdiplus::SafeBitmapWrapper>,
    pub gdiplus_bitmap_bright: Option<crate::service::win32::gdiplus::SafeBitmapWrapper>,
    pub style_initialized: bool,

    // Performance Caches (Global)
    pub snap_x_cache: Vec<i32>,
    pub snap_y_cache: Vec<i32>,
}

impl Default for GdiData {
    fn default() -> Self {
        Self {
            hbitmap_dim: None,
            hbitmap_bright: None,
            bright_pixels: None,
            gdiplus_bitmap_dim: None,
            gdiplus_bitmap_bright: None,
            style_initialized: false,
            snap_x_cache: Vec::new(),
            snap_y_cache: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct VelloData {
    pub background: Option<ImageData>,
    pub background_tex: Option<std::sync::Arc<vello::wgpu::Texture>>,
    pub scene: Option<VelloSceneWrapper>,
    pub d3d_texture: Option<windows::Win32::Graphics::Direct3D11::ID3D11Texture2D>,
}

impl Default for VelloData {
    fn default() -> Self {
        Self {
            background: None,
            background_tex: None,
            scene: None,
            d3d_texture: None,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ToolRegistry {}

#[derive(Clone)]
pub struct VelloSceneWrapper(pub std::sync::Arc<std::sync::Mutex<vello::Scene>>);
impl std::fmt::Debug for VelloSceneWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VelloScene").finish()
    }
}
