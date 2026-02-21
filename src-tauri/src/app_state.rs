use crate::service::config::{types::HotkeyAction, ConfigState};
use crate::service::logger::LoggerState;
use crate::service::native_overlay::OverlayManager;
use std::collections::HashMap;
use std::sync::{Mutex, RwLock};

pub struct AppState {
    pub overlay_manager: Mutex<OverlayManager>,
    pub config_state: Mutex<ConfigState>,
    pub logger_state: LoggerState,
    pub hotkey_map: RwLock<HashMap<u32, HotkeyAction>>,
}
