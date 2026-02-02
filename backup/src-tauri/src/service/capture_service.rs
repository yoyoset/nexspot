use image::RgbaImage;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::Instant;
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

use crate::service::logger;
use crate::service::monitor_info;

// Shared frame buffer for each monitor
// Stores: (Latest Image, Timestamp)
type FrameBuffer = Arc<Mutex<Option<(RgbaImage, Instant)>>>;

pub struct CaptureSession {
    pub frame_buffer: FrameBuffer,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub active: bool,
}

// Global sessions store
static GLOBAL_SESSIONS: OnceLock<Vec<CaptureSession>> = OnceLock::new();

struct CaptureFrameHandler {
    frame_buffer: FrameBuffer,
}

impl GraphicsCaptureApiHandler for CaptureFrameHandler {
    type Flags = FrameBuffer;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
        Ok(Self {
            frame_buffer: ctx.flags,
        })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        _capture_control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        // Just grab the latest frame and store it
        // Fix: buffer is immutable by default from frame.buffer()
        // Get dimensions BEFORE borrowing buffer mutably
        let width = frame.width();
        let height = frame.height();

        if let Ok(mut buffer) = frame.buffer() {
            if let Some(img) = RgbaImage::from_raw(width, height, buffer.as_raw_buffer().to_vec()) {
                if let Ok(mut lock) = self.frame_buffer.lock() {
                    *lock = Some((img, Instant::now()));
                }
            }
        }

        // Keep running! Do NOT stop capture_control
        Ok(())
    }

    fn on_closed(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub fn init_sessions() {
    thread::spawn(|| {
        GLOBAL_SESSIONS.get_or_init(|| {
            logger::log("CaptureService", "Initializing hot-standby sessions...");
            let monitors = Monitor::enumerate().unwrap_or_default();
            let mut sessions = Vec::new();
            let sys_monitors = monitor_info::enumerate_monitors();

            for (i, w_monitor) in monitors.into_iter().enumerate() {
                let frame_buffer: FrameBuffer = Arc::new(Mutex::new(None));
                let buf_clone = frame_buffer.clone();

                let sys_mon = if i < sys_monitors.len() {
                    &sys_monitors[i]
                } else {
                    &sys_monitors[0]
                };

                // Start capture thread for this monitor
                thread::spawn(move || {
                    let settings = Settings::new(
                        w_monitor,
                        CursorCaptureSettings::WithoutCursor,
                        DrawBorderSettings::WithoutBorder,
                        SecondaryWindowSettings::Default,
                        MinimumUpdateIntervalSettings::Default,
                        DirtyRegionSettings::Default,
                        ColorFormat::Rgba8,
                        buf_clone,
                    );

                    if let Err(e) = CaptureFrameHandler::start(settings) {
                        logger::log("CaptureService", &format!("Session {} error: {}", i, e));
                    }
                });

                sessions.push(CaptureSession {
                    frame_buffer,
                    width: sys_mon.width as u32,
                    height: sys_mon.height as u32,
                    x: sys_mon.x,
                    y: sys_mon.y,
                    active: true,
                });
            }

            logger::log("CaptureService", "Hot-standby sessions started.");
            sessions
        });
    });
}

pub fn get_snapshot_fast() -> Result<(Vec<u8>, u32, u32), String> {
    if let Some(sessions) = GLOBAL_SESSIONS.get() {
        let min_x = sessions.iter().map(|s| s.x).min().unwrap_or(0);
        let min_y = sessions.iter().map(|s| s.y).min().unwrap_or(0);
        let max_right = sessions
            .iter()
            .map(|s| s.x + s.width as i32)
            .max()
            .unwrap_or(1920);
        let max_bottom = sessions
            .iter()
            .map(|s| s.y + s.height as i32)
            .max()
            .unwrap_or(1080);

        let total_width = (max_right - min_x) as u32;
        let total_height = (max_bottom - min_y) as u32;

        let mut raw_buffer = vec![0u8; (total_width * total_height * 4) as usize];
        let stitch_start = Instant::now();

        for s in sessions {
            if let Ok(lock) = s.frame_buffer.lock() {
                if let Some((img, _)) = &*lock {
                    let x_offset = (s.x - min_x) as u32;
                    let y_offset = (s.y - min_y) as u32;
                    let img_width = img.width();
                    let img_height = img.height();

                    // Direct buffer copy (much faster than imageops::overlay)
                    for y in 0..img_height {
                        let src_idx = (y * img_width * 4) as usize;
                        let src_row = &img.as_raw()[src_idx..src_idx + (img_width * 4) as usize];

                        let dst_y = y + y_offset;
                        let dst_idx = (dst_y * total_width * 4 + x_offset * 4) as usize;

                        if dst_idx + src_row.len() <= raw_buffer.len() {
                            raw_buffer[dst_idx..dst_idx + src_row.len()].copy_from_slice(src_row);
                        }
                    }
                }
            }
        }

        /* REMOVED: Canvas and BMP encoding - We return raw pixels now for GDI
        // Create image from raw buffer
        let canvas = RgbaImage::from_raw(total_width, total_height, raw_buffer)
            .ok_or("Failed to create canvas from raw buffer")?;
        */

        logger::log(
            "CaptureService",
            &format!("Stitch took {:?}", stitch_start.elapsed()),
        );

        /* REMOVED: BMP Encoding is too slow (1.5s)
        // Encode to BMP
        let encode_start = Instant::now();
        let mut buffer = Vec::new();
        canvas
            .write_to(
                &mut std::io::Cursor::new(&mut buffer),
                image::ImageFormat::Bmp,
            )
            .map_err(|e| e.to_string())?;

        logger::log(
            "CaptureService",
            &format!("Encode took {:?}", encode_start.elapsed()),
        );
        */

        // Return raw RGBA buffer directly
        return Ok((raw_buffer, total_width, total_height));
    }

    Err("Capture service not initialized".to_string())
}
