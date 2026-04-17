use super::VelloContext;
use std::collections::VecDeque;
use std::sync::Arc;
use vello::wgpu;

impl VelloContext {
    pub fn load_fonts(&self, fonts_dir: &std::path::Path) -> anyhow::Result<()> {
        let mut font_context = self
            .font_context
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        if let Ok(entries) = std::fs::read_dir(fonts_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if ext_str == "ttf" || ext_str == "otf" {
                        if let Ok(data) = std::fs::read(&path) {
                            log::info!("[Vello] Loading font into Parley: {:?}", path);
                            font_context.collection.register_fonts(data.into(), None);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn render(
        &self,
        hwnd: windows::Win32::Foundation::HWND,
        _monitor_id: &str,
        width: u32,
        height: u32,
        scene: &vello::Scene,
    ) -> anyhow::Result<()> {
        if width == 0 || height == 0 {
            return Ok(());
        }

        let mut surfaces = self.surfaces.lock().unwrap_or_else(|e| e.into_inner());
        let surface = if let Some(s) = surfaces.get(&(hwnd.0 as isize)) {
            s.clone()
        } else {
            let s = self.create_surface(hwnd)?;
            surfaces.insert(hwnd.0 as isize, s.clone());
            s
        };

        let mut configs = self.surface_configs.lock().unwrap_or_else(|e| e.into_inner());
        let mut caps_cache = self.surface_caps.lock().unwrap_or_else(|e| e.into_inner());

        let key = hwnd.0 as isize;
        if !caps_cache.contains_key(&key) {
            let c = surface.get_capabilities(&self.adapter);
            caps_cache.insert(key, c);
        }
        let caps = caps_cache.get(&key).unwrap_or_else(|| {
            unreachable!("Cache insertion failed unexpectedly")
        });

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

        let needs_reconfig = if let Some(current) = configs.get(&(hwnd.0 as isize)) {
            current.width != width
                || current.height != height
                || current.alpha_mode != alpha_mode
                || current.format != target_config.format
        } else {
            true
        };

        if needs_reconfig {
            surface.configure(&self.device, &target_config);
            configs.insert(hwnd.0 as isize, target_config);

            let mut proxies = self.proxy_textures.lock().unwrap_or_else(|e| e.into_inner());
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
        }

        let proxies = self.proxy_textures.lock().unwrap_or_else(|e| e.into_inner());
        let (proxy_tex, proxy_view) = proxies
            .get(&(hwnd.0 as isize))
            .ok_or_else(|| anyhow::anyhow!("Proxy texture missing"))?;

        let surface_texture = match surface.get_current_texture() {
            Ok(t) => t,
            Err(wgpu::SurfaceError::Lost) | Err(wgpu::SurfaceError::Outdated) => {
                let mut configs = self.surface_configs.lock().unwrap_or_else(|e| e.into_inner());
                configs.remove(&(hwnd.0 as isize));
                return Ok(());
            }
            Err(e) => return Err(anyhow::anyhow!("Failed to get surface: {:?}", e)),
        };

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Vello Composite Encoder"),
            });

        // --- Vello UI & Background Rendering (v0.2.3) ---

        // --- Pass 2 - Vello UI Rendering ---
        {
            let mut renderer = self.renderer.lock().unwrap_or_else(|e| e.into_inner());

            renderer
                .render_to_texture(
                    &self.device,
                    &self.queue,
                    scene,
                    proxy_view,
                    &vello::RenderParams {
                        base_color: vello::peniko::Color::TRANSPARENT,
                        width,
                        height,
                        antialiasing_method: vello::AaConfig::Msaa8,
                    },
                )
                .map_err(|e| anyhow::anyhow!("Render fail: {:?}", e))?;
        }

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

    pub fn update_background(&self, monitor_id: &str, data: &[u8], width: u32, height: u32) {
        if width == 0 || height == 0 { return; }

        let mut backgrounds = self.monitor_backgrounds.lock().unwrap_or_else(|e| e.into_inner());
        
        // 1. Get or Create Resource
        let resource = backgrounds.entry(monitor_id.to_string()).or_insert_with(|| {
            self.create_monitor_resource(width, height)
        });

        // 2. Handle Resolution Changes (Strict Bound Re-init)
        if resource.width != width || resource.height != height {
            log::info!("[Vello] Resolution changed for {}: {}x{} -> {}x{}", monitor_id, resource.width, resource.height, width, height);
            *resource = self.create_monitor_resource(width, height);
        }

        // 3. 5-Frame Rolling Cache Logic
        // If we have less than 5, create a new one. Otherwise, rotate and reuse the oldest.
        let texture = if resource.textures.len() < 5 {
            let new_tex = Arc::new(self.create_one_background_texture(width, height));
            let new_view = new_tex.create_view(&wgpu::TextureViewDescriptor::default());
            resource.textures.push_back(new_tex.clone());
            resource.views.push_back(new_view);
            new_tex
        } else {
            // Rotate: move front to back for reuse
            let tex = resource.textures.pop_front().unwrap();
            let view = resource.views.pop_front().unwrap();
            resource.textures.push_back(tex.clone());
            resource.views.push_back(view);
            tex
        };

        // 4. Write to the Back buffer (the one we just pushed/reused)
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        // FORCE GARBAGE COLLECTION:
        // WGPU buffers `write_texture` commands in a staging pool. 
        // If the overlay is hidden (pre-heating), `render` is never called, so `queue.submit` is never called.
        // This causes staging buffers to pile up infinitely (100MB/s leak).
        // Submitting an empty command forces WGPU to process the writes and reclaim CPU memory immediately.
        self.queue.submit(None);
    }

    fn create_monitor_resource(&self, width: u32, height: u32) -> super::MonitorResource {
        // Initialize with a single texture to start the rolling cache
        let mut textures = VecDeque::new();
        let mut views = VecDeque::new();
        
        let tex = Arc::new(self.create_one_background_texture(width, height));
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        
        textures.push_back(tex);
        views.push_back(view);

        super::MonitorResource {
            textures,
            views,
            width,
            height,
        }
    }

    fn create_one_background_texture(&self, width: u32, height: u32) -> wgpu::Texture {
        self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Vello Monitor Background Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        })
    }

    pub fn purge_surfaces(&self) {
        let mut surfaces = self.surfaces.lock().unwrap_or_else(|e| e.into_inner());
        let mut configs = self.surface_configs.lock().unwrap_or_else(|e| e.into_inner());
        let mut caps = self.surface_caps.lock().unwrap_or_else(|e| e.into_inner());
        let mut proxies = self.proxy_textures.lock().unwrap_or_else(|e| e.into_inner());
        let mut backgrounds = self.monitor_backgrounds.lock().unwrap_or_else(|e| e.into_inner());

        surfaces.clear();
        configs.clear();
        caps.clear();
        proxies.clear();
        backgrounds.clear();
    }
}
