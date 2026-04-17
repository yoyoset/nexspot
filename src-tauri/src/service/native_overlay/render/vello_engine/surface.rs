use super::VelloContext;
use std::sync::Arc;
use vello::wgpu::{self, Surface};

impl VelloContext {
    pub fn create_surface(
        &self,
        hwnd: windows::Win32::Foundation::HWND,
    ) -> anyhow::Result<Arc<Surface<'static>>> {
        let handle = unsafe {
            raw_window_handle::Win32WindowHandle::new(std::num::NonZeroIsize::new_unchecked(
                hwnd.0 as isize,
            ))
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

    pub fn cleanup_surface(&self, hwnd: windows::Win32::Foundation::HWND) {
        let mut surfaces = self.surfaces.lock().unwrap_or_else(|e| e.into_inner());
        let mut configs = self.surface_configs.lock().unwrap_or_else(|e| e.into_inner());
        let mut caps = self.surface_caps.lock().unwrap_or_else(|e| e.into_inner());
        let mut proxies = self.proxy_textures.lock().unwrap_or_else(|e| e.into_inner());

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

    pub fn present_clear_frame(&self) {
        let surfaces = self.surfaces.lock().unwrap_or_else(|e| e.into_inner());
        let configs = self.surface_configs.lock().unwrap_or_else(|e| e.into_inner());

        for (key, surface) in surfaces.iter() {
            if configs.get(key).is_none() {
                continue;
            }

            let texture = match surface.get_current_texture() {
                Ok(t) => t,
                Err(_) => continue,
            };

            let view = texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Clear Frame Encoder"),
                });

            {
                let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Clear Frame Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
            }

            self.queue.submit(Some(encoder.finish()));
            texture.present();

            log::info!(
                "[Vello] Presented transparent clear frame for HWND key {}",
                key
            );
        }
    }
}
