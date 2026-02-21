use global_hotkey::GlobalHotKeyManager;
use serde::{Deserialize, Serialize};

// GlobalHotKeyManager is Send + Sync on Windows when using the default features
// and it's intended to be used across threads as long as it's not dropped.
// We keep the wrapper but document its safety rationale.
// Note: In global-hotkey 0.6.0, it doesn't implement Send/Sync by default
// due to X11 types on Linux, but on Windows it's safe.
pub struct SafeGlobalHotKeyManager(pub GlobalHotKeyManager);
unsafe impl Send for SafeGlobalHotKeyManager {}
unsafe impl Sync for SafeGlobalHotKeyManager {}

#[derive(Debug, Clone)]
pub enum HotkeyAction {
    Workflow(CaptureWorkflow),
    AIShortcut(AIShortcut),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AIShortcut {
    pub id: String,
    pub name: String,
    pub prompt: String,
    pub shortcut: Option<String>,
}

// Duplicate AppConfig removed

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CaptureWorkflow {
    pub id: String,
    pub label: String,
    pub shortcut: String, // e.g. "Alt+A"
    pub action: CaptureAction,
    pub output: CaptureOutput,
    pub enabled: bool,
    #[serde(default)]
    pub is_system: bool, // If true, cannot be deleted
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CaptureOutput {
    pub save_to_file: bool,
    pub save_to_clipboard: bool,
    pub target_folder: Option<String>, // Override global save path
    pub naming_template: String,       // e.g., "capture_%Y%m%d_%H%M%S"
    #[serde(default = "default_format")]
    pub format: String, // "png", "jpg"
}

fn default_format() -> String {
    "png".to_string()
}

fn default_ai_api_url() -> String {
    "https://api.openai.com/v1/chat/completions".to_string()
}

fn default_ai_model() -> String {
    "gpt-3.5-turbo".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "config")]
pub enum CaptureAction {
    Selection {
        engine: String,
    }, // "gdi" or "vello"
    Fullscreen {
        engine: String,
    },
    Window {
        engine: String,
    },
    Snapshot {
        engine: String,
        width: i32,
        height: i32,
        #[serde(default)]
        allow_resize: bool,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct AppConfig {
    pub workflows: Vec<CaptureWorkflow>,

    pub save_path: String,
    pub language: String,
    pub font_family: String,
    pub vello_enabled: bool,
    pub vello_advanced_effects: bool,

    // Legacy Snapshot Config (Migrate to Workflow)
    pub snapshot_enabled: bool,
    pub snapshot_width: i32,
    pub snapshot_height: i32,

    pub selection_engine: String, // "gdi" or "vello"
    pub snapshot_engine: String,  // "gdi" or "vello"

    // AI Configuration
    #[serde(default = "default_ai_api_url")]
    pub ai_api_url: String,
    #[serde(default = "default_ai_model")]
    pub ai_model: String,
    #[serde(default)]
    pub ai_api_key: String,

    #[serde(default)]
    pub ai_shortcuts: Vec<AIShortcut>,

    // Appearance Configuration
    #[serde(default = "default_theme")]
    pub theme: String, // "light", "dark", "system"
    #[serde(default = "default_accent_color")]
    pub accent_color: String, // hex color

    // Performance & Quality
    #[serde(default = "default_jpg_quality")]
    pub jpg_quality: u8,
    #[serde(default = "default_concurrency")]
    pub concurrency: usize,

    #[serde(skip)]
    #[serde(default)]
    pub registration_errors: Vec<String>,
}

fn default_jpg_quality() -> u8 {
    90
}

fn default_concurrency() -> usize {
    4
}

fn default_theme() -> String {
    "system".to_string()
}

fn default_accent_color() -> String {
    "#3b82f6".to_string()
}

pub fn default_workflows() -> Vec<CaptureWorkflow> {
    vec![
        CaptureWorkflow {
            id: "capture_default".to_string(),
            label: "Capture Selection".to_string(),
            shortcut: "Alt+A".to_string(),
            action: CaptureAction::Selection {
                engine: "gdi".to_string(),
            },
            output: CaptureOutput {
                save_to_file: true,
                save_to_clipboard: true,
                target_folder: None,
                naming_template: "%Y-%m-%d_%H-%M-%S".to_string(),
                format: "png".to_string(),
            },
            enabled: true,
            is_system: true,
        },
        CaptureWorkflow {
            id: "snapshot_default".to_string(),
            label: "Snapshot".to_string(),
            shortcut: "Alt+S".to_string(),
            action: CaptureAction::Snapshot {
                engine: "gdi".to_string(),
                width: 800,
                height: 600,
                allow_resize: true,
            },
            output: CaptureOutput {
                save_to_file: true,
                save_to_clipboard: true,
                target_folder: None,
                naming_template: "snapshot_%Y-%m-%d_%H-%M-%S".to_string(),
                format: "png".to_string(),
            },
            enabled: false, // Disabled by default to avoid conflict expectation
            is_system: true,
        },
    ]
}
