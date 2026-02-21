use crate::service::win32::send_sync::SendHWND;
use std::collections::HashMap;
use std::sync::Arc;

// Use wgpu re-exported by vello for guaranteed compatibility
pub use vello::wgpu;
pub use wgpu::{
    Device, Extent3d, Instance, Origin3d, Queue, Surface, SurfaceConfiguration, Texture,
    TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor,
};
// Use the modern TexelCopy API types from vello::wgpu if available,
// otherwise fallback to what we know works.
pub use wgpu::TexelCopyTextureInfo;

use vello::{Renderer, RendererOptions, Scene};

pub mod renderer;

pub struct VelloContext {
    pub instance: Arc<Instance>,
    pub adapter: Arc<vello::wgpu::Adapter>, // Added to query capabilities
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub renderer: std::sync::Mutex<Renderer>,
    pub scene: std::sync::Mutex<Scene>,
    pub surfaces: std::sync::Mutex<HashMap<isize, Arc<Surface<'static>>>>,
    pub surface_configs: std::sync::Mutex<HashMap<isize, SurfaceConfiguration>>, // Cache configs
    pub surface_caps: std::sync::Mutex<HashMap<isize, wgpu::SurfaceCapabilities>>, // Cache capabilities
    pub proxy_textures: std::sync::Mutex<HashMap<isize, (wgpu::Texture, wgpu::TextureView)>>, // Proxy for compatibility
}

impl VelloContext {
    pub async fn new(initial_hwnd: Option<SendHWND>) -> anyhow::Result<Self> {
        let instance = vello::wgpu::Instance::default();

        let mut compatible_surface = None;
        if let Some(send_hwnd) = initial_hwnd {
            let hwnd = send_hwnd.0;
            if hwnd.0.is_null() {
                return Err(anyhow::anyhow!(
                    "Invalid HWND (NULL) passed to VelloContext"
                ));
            }

            // Create a temporary surface to ensure adapter compatibility
            let handle = unsafe {
                raw_window_handle::Win32WindowHandle::new(std::num::NonZeroIsize::new_unchecked(
                    hwnd.0 as isize,
                ))
            };
            let display = raw_window_handle::WindowsDisplayHandle::new();
            let window_handle = raw_window_handle::RawWindowHandle::Win32(handle);
            let display_handle = raw_window_handle::RawDisplayHandle::Windows(display);

            unsafe {
                if let Ok(surface) =
                    instance.create_surface_unsafe(vello::wgpu::SurfaceTargetUnsafe::RawHandle {
                        raw_window_handle: window_handle,
                        raw_display_handle: display_handle,
                    })
                {
                    compatible_surface = Some(surface);
                }
            }
        }

        let adapter = instance
            .request_adapter(&vello::wgpu::RequestAdapterOptions {
                power_preference: vello::wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: compatible_surface.as_ref(),
            })
            .await
            .map_err(|e| anyhow::anyhow!("Failed to find a suitable wgpu adapter: {:?}", e))?;

        let (device, queue) = adapter
            .request_device(&vello::wgpu::DeviceDescriptor {
                label: Some("Vello Device"),
                ..Default::default()
            })
            .await?;

        let adapter = Arc::new(adapter);
        let instance = Arc::new(instance);
        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let renderer = Renderer::new(
            &*device,
            RendererOptions {
                use_cpu: false,
                antialiasing_support: vello::AaSupport::all(),
                num_init_threads: None,
                pipeline_cache: None,
            },
        )
        .map_err(|e| anyhow::anyhow!("Failed to create vello renderer: {:?}", e))?;

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            renderer: std::sync::Mutex::new(renderer),
            scene: std::sync::Mutex::new(Scene::new()),
            surfaces: std::sync::Mutex::new(HashMap::new()),
            surface_configs: std::sync::Mutex::new(HashMap::new()),
            surface_caps: std::sync::Mutex::new(HashMap::new()),
            proxy_textures: std::sync::Mutex::new(HashMap::new()),
        })
    }

    pub fn create_surface(
        &self,
        hwnd: windows::Win32::Foundation::HWND,
    ) -> anyhow::Result<Arc<Surface<'static>>> {
        // vello::wgpu should be used consistently
        use vello::wgpu;

        let handle = unsafe {
            let handle = raw_window_handle::Win32WindowHandle::new(
                std::num::NonZeroIsize::new_unchecked(hwnd.0 as isize),
            );
            handle
        };

        let display = raw_window_handle::WindowsDisplayHandle::new();

        let window_handle = raw_window_handle::RawWindowHandle::Win32(handle);
        let display_handle = raw_window_handle::RawDisplayHandle::Windows(display);

        unsafe {
            let surface = self
                .instance
                .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                    raw_window_handle: window_handle,
                    raw_display_handle: display_handle,
                })
                .map_err(|e| anyhow::anyhow!("Failed to create wgpu surface: {:?}", e))?;
            Ok(Arc::new(surface))
        }
    }

    pub fn render(
        &self,
        hwnd: windows::Win32::Foundation::HWND,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        if width == 0 || height == 0 {
            return Ok(());
        }

        let mut surfaces = match self.surfaces.lock() {
            Ok(s) => s,
            Err(_) => return Err(anyhow::anyhow!("Surfaces mutex poisoned")),
        };
        let surface = if let Some(s) = surfaces.get(&(hwnd.0 as isize)) {
            s.clone()
        } else {
            let s = self.create_surface(hwnd)?;
            surfaces.insert(hwnd.0 as isize, s.clone());
            s
        };
        // The user's provided snippet for `(_mode, _zone)` was syntactically incorrect and
        // referenced `state_arc` which is not defined in this context.
        // Assuming it was meant to be placed outside the `if/else` block and
        // was a placeholder for some future logic, it's omitted to maintain
        // syntactic correctness and avoid compilation errors.

        let mut configs = match self.surface_configs.lock() {
            Ok(c) => c,
            Err(_) => return Err(anyhow::anyhow!("SurfaceConfigs mutex poisoned")),
        };

        let mut caps_cache = match self.surface_caps.lock() {
            Ok(c) => c,
            Err(_) => return Err(anyhow::anyhow!("SurfaceCaps mutex poisoned")),
        };

        // 1. Get or cached capabilities
        let key = hwnd.0 as isize;
        let caps = if let Some(c) = caps_cache.get(&key) {
            c
        } else {
            let c = surface.get_capabilities(&self.adapter);
            caps_cache.insert(key, c);
            caps_cache.get(&key).unwrap()
        };

        // 2. Determine Alpha Mode
        let alpha_mode = if caps
            .alpha_modes
            .contains(&wgpu::CompositeAlphaMode::PostMultiplied)
        {
            wgpu::CompositeAlphaMode::PostMultiplied
        } else if caps
            .alpha_modes
            .contains(&wgpu::CompositeAlphaMode::PreMultiplied)
        {
            wgpu::CompositeAlphaMode::PreMultiplied
        } else {
            wgpu::CompositeAlphaMode::Opaque
        };

        // 3. Determine Surface Format (Prefer Rgba8Unorm or Bgra8Unorm)
        let surface_format = if caps.formats.contains(&wgpu::TextureFormat::Rgba8Unorm) {
            wgpu::TextureFormat::Rgba8Unorm
        } else if caps.formats.contains(&wgpu::TextureFormat::Bgra8Unorm) {
            wgpu::TextureFormat::Bgra8Unorm
        } else {
            caps.formats
                .get(0)
                .copied()
                .unwrap_or(wgpu::TextureFormat::Bgra8Unorm)
        };

        let target_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST,
            format: surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        // Check if configuration actually changed
        let needs_reconfig = if let Some(current) = configs.get(&(hwnd.0 as isize)) {
            current.width != width
                || current.height != height
                || current.alpha_mode != alpha_mode
                || current.format != target_config.format
        } else {
            true
        };

        if needs_reconfig {
            log::info!(
                "[Vello] Configuring surface for HWND {:?}: {}x{}, Alpha: {:?}",
                hwnd,
                width,
                height,
                alpha_mode
            );
            surface.configure(&self.device, &target_config);
            configs.insert(hwnd.0 as isize, target_config);

            // Create or Resize Proxy Texture
            let mut proxies = self.proxy_textures.lock().unwrap();
            let proxy_format = wgpu::TextureFormat::Rgba8Unorm;
            let proxy = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Vello Proxy Texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: proxy_format,
                usage: wgpu::TextureUsages::STORAGE_BINDING
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let view = proxy.create_view(&wgpu::TextureViewDescriptor {
                label: Some("Vello Proxy View"),
                format: Some(proxy_format),
                ..Default::default()
            });
            proxies.insert(hwnd.0 as isize, (proxy, view));
            log::info!(
                "[Vello] Created proxy texture ({:?}) and view for HWND {:?}",
                proxy_format,
                hwnd
            );
        }

        let proxies = self.proxy_textures.lock().unwrap();
        let (_proxy_tex, proxy_view) = proxies
            .get(&(hwnd.0 as isize))
            .ok_or_else(|| anyhow::anyhow!("Proxy texture missing for HWND {:?}", hwnd))?;

        let surface_texture = match surface.get_current_texture() {
            Ok(t) => t,
            Err(wgpu::SurfaceError::Lost) | Err(wgpu::SurfaceError::Outdated) => {
                // Surface is lost or outdated, force a re-config next time
                let mut configs = self.surface_configs.lock().unwrap();
                configs.remove(&(hwnd.0 as isize));
                log::warn!(
                    "[Vello] Surface lost or outdated for HWND {:?}, retrying next frame",
                    hwnd
                );
                return Ok(());
            }
            Err(e) => return Err(anyhow::anyhow!("Failed to get surface texture: {:?}", e)),
        };

        let scene = match self.scene.lock() {
            Ok(s) => s,
            Err(_) => return Err(anyhow::anyhow!("Scene mutex poisoned")),
        };
        let mut renderer = match self.renderer.lock() {
            Ok(s) => s,
            Err(_) => return Err(anyhow::anyhow!("Renderer mutex poisoned")),
        };

        // 1. Render to PROXY (using the format Vello expects)
        let render_res = renderer.render_to_texture(
            &self.device,
            &self.queue,
            &scene,
            proxy_view,
            &vello::RenderParams {
                base_color: vello::peniko::Color::TRANSPARENT,
                width,
                height,
                antialiasing_method: vello::AaConfig::Msaa8,
            },
        );

        if let Err(e) = render_res {
            log::error!("Vello render error: {:?}", e);
            return Err(anyhow::anyhow!("Vello render error: {:?}", e));
        }

        // 2. Copy PROXY to SURFACE (let wgpu handle the format/colorspace conversion)
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Proxy Copy Encoder"),
            });

        let (proxy_tex, _) = proxies.get(&(hwnd.0 as isize)).unwrap();

        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: proxy_tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: &surface_texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(Some(encoder.finish()));

        surface_texture.present();
        Ok(())
    }

    pub fn cleanup_surface(&self, hwnd: windows::Win32::Foundation::HWND) {
        let mut surfaces = match self.surfaces.lock() {
            Ok(s) => s,
            Err(_) => return,
        };
        let mut configs = match self.surface_configs.lock() {
            Ok(c) => c,
            Err(_) => return,
        };
        let mut caps = match self.surface_caps.lock() {
            Ok(c) => c,
            Err(_) => return,
        };
        let mut proxies = match self.proxy_textures.lock() {
            Ok(p) => p,
            Err(_) => return,
        };
        let key = hwnd.0 as isize;
        surfaces.remove(&key);
        configs.remove(&key);
        caps.remove(&key);
        proxies.remove(&key);
        log::debug!(
            "[Vello] Cleaned up surface, proxy, and config for HWND {:?}",
            hwnd
        );
    }
}
