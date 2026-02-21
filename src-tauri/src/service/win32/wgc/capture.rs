use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use windows::core::Interface;
use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Direct3D11::{
    ID3D11Device, ID3D11Texture2D, D3D11_BIND_SHADER_RESOURCE, D3D11_TEXTURE2D_DESC,
    D3D11_USAGE_DEFAULT,
};
use windows::Win32::System::WinRT::Direct3D11::IDirect3DDxgiInterfaceAccess;
use windows_capture::{
    capture::{Context, GraphicsCaptureApiHandler},
    frame::Frame,
    graphics_capture_api::InternalCaptureControl,
    monitor::Monitor,
    settings::{
        ColorFormat, CursorCaptureSettings, DirtyRegionSettings, DrawBorderSettings,
        MinimumUpdateIntervalSettings, SecondaryWindowSettings, Settings,
    },
};

pub struct OneShotState {
    texture: Option<ID3D11Texture2D>,
    image: Option<vello::peniko::ImageData>,
    size: (u32, u32),
    captured: bool,
    _error: Option<String>,
}

pub struct OneShotHandler {
    state: Arc<Mutex<OneShotState>>,
}

impl GraphicsCaptureApiHandler for OneShotHandler {
    type Flags = Arc<Mutex<OneShotState>>;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
        Ok(Self { state: ctx.flags })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        capture_control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        let mut state = self.state.lock().unwrap();

        if state.captured {
            capture_control.stop();
            return Ok(());
        }

        // Texture Copy Logic (Optional for D2D, but we keep it if needed for others)
        let surface = unsafe { frame.as_raw_surface() };
        let access: IDirect3DDxgiInterfaceAccess = surface.cast()?;
        let texture_src: ID3D11Texture2D = unsafe { access.GetInterface()? };

        let device: ID3D11Device = unsafe { texture_src.GetDevice()? };

        let mut desc = D3D11_TEXTURE2D_DESC::default();
        unsafe {
            texture_src.GetDesc(&mut desc);
        }

        desc.BindFlags = D3D11_BIND_SHADER_RESOURCE.0 as u32;
        desc.MiscFlags = 0;
        desc.CPUAccessFlags = 0;
        desc.Usage = D3D11_USAGE_DEFAULT;

        let texture_dst = unsafe {
            let mut tex_out: Option<ID3D11Texture2D> = None;
            device.CreateTexture2D(&desc, None, Some(&mut tex_out))?;
            tex_out.ok_or("Failed to create dst texture")?
        };

        let context = unsafe { device.GetImmediateContext()? };

        unsafe {
            context.CopyResource(&texture_dst, &texture_src);
        }

        // Vello Image Logic: Extract buffer for cross-platform rendering
        let (width, height) = (frame.width(), frame.height());
        let mut frame_buffer = frame.buffer()?;
        let data = vello::peniko::Blob::from(frame_buffer.as_raw_buffer().to_vec());
        let image = vello::peniko::ImageData {
            data,
            format: vello::peniko::ImageFormat::Rgba8,
            alpha_type: vello::peniko::ImageAlphaType::Alpha,
            width,
            height,
        };

        state.texture = Some(texture_dst);
        state.image = Some(image);
        state.size = (width, height);
        state.captured = true;

        capture_control.stop();
        Ok(())
    }

    fn on_closed(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub struct StreamState {
    pub image: Option<vello::peniko::ImageData>,
    pub size: (u32, u32),
    pub stop: bool,
    pub is_alive: bool,
}

pub struct WgcStreamHandler {
    state: Arc<Mutex<StreamState>>,
}

impl GraphicsCaptureApiHandler for WgcStreamHandler {
    type Flags = Arc<Mutex<StreamState>>;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
        Ok(Self { state: ctx.flags })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        _capture_control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        let (width, height) = (frame.width(), frame.height());
        let mut frame_buffer = frame.buffer()?;
        let data = vello::peniko::Blob::from(frame_buffer.as_raw_buffer().to_vec());

        let image = vello::peniko::ImageData {
            data,
            format: vello::peniko::ImageFormat::Rgba8,
            alpha_type: vello::peniko::ImageAlphaType::Alpha,
            width,
            height,
        };

        let mut state = self.state.lock().unwrap();
        if state.stop {
            _capture_control.stop();
            return Ok(());
        }
        state.image = Some(image);
        state.size = (width, height);

        Ok(())
    }

    fn on_closed(&mut self) -> Result<(), Self::Error> {
        if let Ok(mut state) = self.state.lock() {
            state.is_alive = false;
        }
        log::warn!("WGC Stream explicitly unexpectedly closed. State marked dead.");
        Ok(())
    }
}

pub struct WgcStreamManager {
    pub states: Arc<Mutex<HashMap<usize, Arc<Mutex<StreamState>>>>>,
    _handles: Arc<Mutex<Vec<std::thread::JoinHandle<()>>>>,
}

impl WgcStreamManager {
    pub fn new() -> Self {
        Self {
            states: Arc::new(Mutex::new(HashMap::new())),
            _handles: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        let monitors = match Monitor::enumerate() {
            Ok(m) => m,
            Err(e) => {
                log::error!("WGC Monitor enumeration failed: {:?}", e);
                return Err(anyhow::anyhow!("Monitor enumeration failed: {:?}", e));
            }
        };

        for (index, monitor) in monitors.into_iter().enumerate() {
            let monitor_name = match monitor.name() {
                Ok(n) => n,
                Err(e) => {
                    log::error!("Failed to get monitor name: {:?}", e);
                    continue;
                }
            };
            log::info!(
                "[WGC Manager] Enumerated monitor {}: {}",
                index,
                monitor_name
            );
            let state = Arc::new(Mutex::new(StreamState {
                image: None,
                size: (0, 0),
                stop: false,
                is_alive: true,
            }));

            // Store in our states map keyed by Index
            {
                let mut states = self.states.lock().unwrap();
                states.insert(index, state.clone());
            }

            log::info!(
                "Starting WGC Pre-heat Stream for monitor idx {}: {}",
                index,
                monitor_name
            );

            let handle = std::thread::spawn(move || {
                let state_clone = state.clone();
                let settings = Settings::new(
                    monitor,
                    CursorCaptureSettings::Default,
                    DrawBorderSettings::WithoutBorder,
                    SecondaryWindowSettings::Default,
                    MinimumUpdateIntervalSettings::Default,
                    DirtyRegionSettings::Default,
                    ColorFormat::Rgba8,
                    state_clone,
                );

                if let Err(e) = WgcStreamHandler::start(settings) {
                    log::error!("WGC Stream failed for monitor {}: {:?}", monitor_name, e);
                    if let Ok(mut s) = state.lock() {
                        s.is_alive = false;
                    }
                }
            });

            self._handles.lock().unwrap().push(handle);
        }

        Ok(())
    }

    pub fn get_states(&self) -> Arc<Mutex<HashMap<usize, Arc<Mutex<StreamState>>>>> {
        self.states.clone()
    }

    pub fn grab_latest_frame(
        &self,
        monitor_index: usize,
    ) -> Option<(vello::peniko::ImageData, (u32, u32))> {
        let states = self.states.lock().ok()?;
        let state_arc = states.get(&monitor_index)?;
        let lock = state_arc.lock().ok()?;

        if let (Some(img), size, true) = (lock.image.clone(), lock.size, lock.is_alive) {
            Some((img, size))
        } else {
            None
        }
    }
}

impl Drop for WgcStreamManager {
    fn drop(&mut self) {
        log::info!("Stopping WGC Stream Manager (multi-monitor)...");
        if let Ok(states) = self.states.lock() {
            for (idx, state_arc) in states.iter() {
                if let Ok(mut state) = state_arc.lock() {
                    log::debug!("Stopping WGC stream for monitor idx {}", idx);
                    state.stop = true;
                }
            }
        }
        // Handles will terminate as their capture loops stop
    }
}

pub fn capture_monitor_to_vello(
    monitor_index: usize,
    monitor_name: &str,
    friendly_name: &str,
    _target_rect: Option<RECT>,
) -> anyhow::Result<(vello::peniko::ImageData, (u32, u32))> {
    // Keep legacy fallback for now if pre-heat is off
    let state = Arc::new(Mutex::new(OneShotState {
        texture: None,
        image: None,
        size: (0, 0),
        captured: false,
        _error: None,
    }));

    let monitors = Monitor::enumerate()?;
    let monitor = monitors
        .into_iter()
        .enumerate()
        .find(|(idx, _)| *idx == monitor_index)
        .map(|(_, m)| m)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Monitor not found for one-shot at index {}: {} (Friendly: {})",
                monitor_index,
                monitor_name,
                friendly_name
            )
        })?;

    let settings = Settings::new(
        monitor.clone(),
        CursorCaptureSettings::Default,
        DrawBorderSettings::WithoutBorder,
        SecondaryWindowSettings::Default,
        MinimumUpdateIntervalSettings::Default,
        DirtyRegionSettings::Default,
        ColorFormat::Rgba8,
        state.clone(),
    );

    OneShotHandler::start(settings).map_err(|e| anyhow::anyhow!("Capture failed: {:?}", e))?;

    let lock = match state.lock() {
        Ok(s) => s,
        Err(_) => return Err(anyhow::anyhow!("Mutex poisoned")),
    };
    if let (Some(img), size) = (lock.image.clone(), lock.size) {
        Ok((img, size))
    } else {
        Err(anyhow::anyhow!("Capture finished without image"))
    }
}
