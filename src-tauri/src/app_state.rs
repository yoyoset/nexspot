use crate::service::config::ConfigState;
use crate::service::logger::LoggerState;
use crate::service::native_overlay::OverlayManager;
use std::sync::Mutex;

pub struct AppState {
    pub overlay_manager: Mutex<OverlayManager>,
    pub config_state: Mutex<ConfigState>,
    pub logger_state: LoggerState,
}
