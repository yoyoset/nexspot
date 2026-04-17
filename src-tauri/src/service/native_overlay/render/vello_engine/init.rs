use super::VelloContext;
use crate::service::win32::send_sync::SendHWND;
use parley::FontContext;
use std::collections::HashMap;
use std::sync::Arc;
use vello::{Renderer, RendererOptions, Scene};

impl VelloContext {
    pub async fn new(initial_hwnd: Option<SendHWND>) -> anyhow::Result<Self> {
        let instance = vello::wgpu::Instance::new(&vello::wgpu::InstanceDescriptor {
            backends: vello::wgpu::Backends::all(),
            ..Default::default()
        });

        let mut compatible_surface = None;
        if let Some(send_hwnd) = initial_hwnd {
            let hwnd = send_hwnd.0;
            if hwnd.0.is_null() {
                return Err(anyhow::anyhow!(
                    "Invalid HWND (NULL) passed to VelloContext"
                ));
            }

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

        log::info!(
            "[Vello] Requesting Adapter (HighPerformance, Surface={})",
            compatible_surface.is_some()
        );

        let adapter = match instance
            .request_adapter(&vello::wgpu::RequestAdapterOptions {
                power_preference: vello::wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: compatible_surface.as_ref(),
            })
            .await
        {
            Ok(a) => a,
            Err(_) => {
                log::warn!(
                    "[Vello] request_adapter with surface failed, trying without surface..."
                );
                instance
                    .request_adapter(&vello::wgpu::RequestAdapterOptions {
                        power_preference: vello::wgpu::PowerPreference::HighPerformance,
                        force_fallback_adapter: false,
                        compatible_surface: None,
                    })
                    .await
                    .map_err(|e| {
                        anyhow::anyhow!("Failed to find any suitable wgpu adapter: {:?}", e)
                    })?
            }
        };

        let info = adapter.get_info();
        log::info!(
            "[Vello] Selected Adapter: {} ({:?}) on {:?}",
            info.name,
            info.device_type,
            info.backend
        );

        let required_features = vello::wgpu::Features::empty();
        let supported_features = adapter.features();
        if supported_features.contains(vello::wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY) {
            // required_features |= vello::wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY;
        }

        let (device, queue) = adapter
            .request_device(&vello::wgpu::DeviceDescriptor {
                label: Some("Nexspot Vello Device"),
                required_features,
                required_limits: vello::wgpu::Limits::default(),
                ..Default::default()
            })
            .await
            .map_err(|e| anyhow::anyhow!("Failed to request wgpu device: {:?}", e))?;

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

        let font_context = FontContext::new();

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
            monitor_backgrounds: std::sync::Mutex::new(HashMap::new()),
            font_context: std::sync::Mutex::new(font_context),
            layout_context: std::sync::Mutex::new(parley::LayoutContext::new()),
        })
    }
}
