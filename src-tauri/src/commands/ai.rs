use crate::AppState;
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub async fn stream_ai_response(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    prompt: String,
    pin_id: String,
) -> Result<(), String> {
    let (api_url, api_key, model): (String, String, String) = {
        let config_state = state.config_state.lock().map_err(|e| e.to_string())?;
        (
            config_state.config.ai_api_url.clone(),
            config_state.config.ai_api_key.clone(),
            config_state.config.ai_model.clone(),
        )
    };

    if api_key.is_empty() {
        return Err("AI API Key is missing. Please configure it in Settings.".to_string());
    }

    let provider = crate::service::ai::openai::OpenAIProvider::new(api_url, api_key, model);

    use crate::service::ai::LLMProvider;
    use futures::StreamExt;

    let stream_result = provider.stream(&prompt).await.map_err(|e| e.to_string())?;

    // Spawn a task to handle the stream so the command returns immediately (or we could await it)
    // Actually, usually commands await completion. But for streaming, we want to emit events.
    // However, if we await here, the frontend promise won't resolve until done.
    // If we want real-time, we usually just await the whole thing while emitting events.

    let mut stream = stream_result;
    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(content) => {
                let payload: serde_json::Value =
                    serde_json::json!({ "id": pin_id, "content": content });
                app.emit("ai-stream://chunk", payload)
                    .map_err(|e| e.to_string())?;
            }
            Err(e) => {
                let payload: serde_json::Value =
                    serde_json::json!({ "id": pin_id, "error": e.to_string() });
                app.emit("ai-stream://error", payload)
                    .map_err(|e| e.to_string())?;
            }
        }
    }

    let payload: serde_json::Value = serde_json::json!({ "id": pin_id });
    app.emit("ai-stream://done", payload)
        .map_err(|e| e.to_string())?;

    Ok(())
}
