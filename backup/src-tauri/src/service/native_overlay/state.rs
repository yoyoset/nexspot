use std::sync::Mutex;
use windows::Win32::Foundation::POINT;
use windows::Win32::Graphics::Gdi::HBITMAP;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InteractionState {
    Idle,            // Initial state
    Creating,        // Dragging to create new selection
    Selected,        // Selection exists, displaying anchors
    Moving,          // Dragging the entire selection
    Resizing(usize), // Dragging an anchor (0-7 index)
}

use super::gdi::AutoGdiObject;
use super::toolbar::Toolbar;

pub struct OverlayState {
    pub width: i32,
    pub height: i32,
    // GDI Objects (Wrapped in RAII AutoGdiObject)
    pub hbitmap_bright: AutoGdiObject,
    pub hbitmap_dim: AutoGdiObject,
    // Selection Logic
    pub start_pos: POINT,
    pub current_pos: POINT,
    pub interaction_state: InteractionState, // Replaces is_dragging/selection_active
    // We persist the finalized rect to support "Selected" state logic
    pub final_rect: Option<windows::Win32::Foundation::RECT>,
    pub toolbar: Toolbar, // New Toolbar Component
}

// Safety: HBITMAP and POINT are Send
unsafe impl Send for OverlayState {}

// Global State
pub static STATE: Mutex<Option<OverlayState>> = Mutex::new(None);

// Event Callback Mechanism
pub type EventCallback = Box<dyn Fn(String, String) + Send + Sync>; // event_name, payload
pub static CALLBACK: Mutex<Option<EventCallback>> = Mutex::new(None);

pub fn set_event_callback<F>(callback: F)
where
    F: Fn(String, String) + Send + Sync + 'static,
{
    let mut guard = CALLBACK.lock().unwrap();
    *guard = Some(Box::new(callback));
}

pub fn emit_event(name: &str, payload: &str) {
    if let Ok(guard) = CALLBACK.lock() {
        if let Some(cb) = guard.as_ref() {
            cb(name.to_string(), payload.to_string());
        }
    }
}
