use super::VelloContext;
use vello::wgpu;

impl VelloContext {
    pub async fn render_to_image(
        &self,
        width: u32,
        height: u32,
        scene: &vello::Scene,
    ) -> anyhow::Result<vello::peniko::ImageData> {
        if width == 0 || height == 0 {
            return Err(anyhow::anyhow!("Invalid dimensions for offscreen render"));
        }

        let texture_format = wgpu::TextureFormat::Rgba8Unorm;
        let texture_desc = wgpu::TextureDescriptor {
            label: Some("Vello Offscreen Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: texture_format,
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = self.device.create_texture(&texture_desc);
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // 1. Render Scene to Texture
        {
            let mut renderer = self.renderer.lock().unwrap_or_else(|e| e.into_inner());
            renderer
                .render_to_texture(
                    &self.device,
                    &self.queue,
                    scene,
                    &texture_view,
                    &vello::RenderParams {
                        base_color: vello::peniko::Color::TRANSPARENT,
                        width,
                        height,
                        antialiasing_method: vello::AaConfig::Msaa8,
                    },
                )
                .map_err(|e| anyhow::anyhow!("Offscreen render failed: {:?}", e))?;
        }

        // 2. Readback from GPU
        let u32_size = std::mem::size_of::<u32>() as u32;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let unpadded_bytes_per_row = width * u32_size;
        let padding = (align - unpadded_bytes_per_row % align) % align;
        let padded_bytes_per_row = unpadded_bytes_per_row + padding;

        let buffer_size = (padded_bytes_per_row * height) as wgpu::BufferAddress;
        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vello Offscreen Staging Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Offscreen Readback Encoder"),
            });

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &staging_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(Some(encoder.finish()));

        // 3. Map and extract
        let (tx, rx) = std::sync::mpsc::channel();
        let buffer_slice = staging_buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| {
            let _ = tx.send(v);
        });
        let _ = self.device
            .poll(wgpu::PollType::wait_indefinitely());
        if let Ok(Ok(())) = rx.recv() {
            let data = buffer_slice.get_mapped_range();
            let mut result_pixels = Vec::with_capacity((width * height * 4) as usize);

            for row in 0..height {
                let start = (row * padded_bytes_per_row) as usize;
                let end = start + unpadded_bytes_per_row as usize;
                result_pixels.extend_from_slice(&data[start..end]);
            }

            drop(data);
            staging_buffer.unmap();

            Ok(vello::peniko::ImageData {
                data: vello::peniko::Blob::from(result_pixels),
                format: vello::peniko::ImageFormat::Rgba8,
                alpha_type: vello::peniko::ImageAlphaType::Alpha,
                width,
                height,
            })
        } else {
            Err(anyhow::anyhow!("Failed to map staging buffer"))
        }
    }
}
