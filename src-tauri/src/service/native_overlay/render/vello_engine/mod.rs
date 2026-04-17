use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use vello::{Renderer, Scene};

// Use wgpu re-exported by vello for guaranteed compatibility
pub use vello::wgpu;
pub use wgpu::{Device, Instance, Queue, Surface, SurfaceConfiguration};

mod init;
mod offscreen;
pub mod renderer;
mod rendering;
mod surface;

pub struct VelloContext {
    pub instance: Arc<Instance>,
    pub adapter: Arc<wgpu::Adapter>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub renderer: std::sync::Mutex<Renderer>,
    pub scene: std::sync::Mutex<Scene>,
    pub surfaces: std::sync::Mutex<HashMap<isize, Arc<Surface<'static>>>>,
    pub surface_configs: std::sync::Mutex<HashMap<isize, SurfaceConfiguration>>,
    pub surface_caps: std::sync::Mutex<HashMap<isize, wgpu::SurfaceCapabilities>>,
    pub proxy_textures: std::sync::Mutex<HashMap<isize, (wgpu::Texture, wgpu::TextureView)>>,
    pub font_context: std::sync::Mutex<parley::FontContext>,
    pub layout_context: std::sync::Mutex<parley::LayoutContext<[u8; 4]>>,
    pub monitor_backgrounds: std::sync::Mutex<HashMap<String, MonitorResource>>,
}

pub struct MonitorResource {
    pub textures: VecDeque<Arc<wgpu::Texture>>,
    pub views: VecDeque<wgpu::TextureView>,
    pub width: u32,
    pub height: u32,
}

impl VelloContext {
    // Methods are now in submodules (init.rs, surface.rs, rendering.rs)
}
