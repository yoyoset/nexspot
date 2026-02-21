use super::types::AppConfig;

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            save_path: "captures".to_string(),
            language: "zh".to_string(),
            font_family: "Segoe UI".to_string(),
            vello_enabled: false,
            vello_advanced_effects: true,
            snapshot_enabled: false,
            snapshot_width: 800,
            snapshot_height: 600,
            selection_engine: "gdi".to_string(),
            snapshot_engine: "vello".to_string(),
            ai_api_url: "https://api.openai.com/v1/chat/completions".to_string(),
            ai_model: "gpt-3.5-turbo".to_string(),
            ai_api_key: String::new(),
            ai_shortcuts: Vec::new(),
            workflows: super::types::default_workflows(),
            theme: "system".to_string(),
            accent_color: "#3b82f6".to_string(),
            jpg_quality: 90,
            concurrency: 4,
            registration_errors: Vec::new(),
        }
    }
}
