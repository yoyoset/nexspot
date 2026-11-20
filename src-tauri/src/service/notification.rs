use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_notification::NotificationExt;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureErrorEvent<'a> {
    pub area: &'a str,
    pub code: &'a str,
    pub title: &'a str,
    pub message: &'a str,
}

pub fn notify_warn(app: &AppHandle, title: &str, body: &str) {
    let _ = app.notification().builder().title(title).body(body).show();
}

pub fn notify_error(app: &AppHandle, title: &str, body: &str) {
    let _ = app.notification().builder().title(title).body(body).show();
}

pub fn notify_capture_error(app: &AppHandle, area: &str, code: &str, title: &str, body: &str) {
    let _ = app.notification().builder().title(title).body(body).show();
    emit_capture_error(app, area, code, title, body);
}

pub fn emit_capture_error(app: &AppHandle, area: &str, code: &str, title: &str, body: &str) {
    let _ = app.emit(
        "capture://error",
        CaptureErrorEvent {
            area,
            code,
            title,
            message: body,
        },
    );
}

pub fn notify_error_blocking(app: &AppHandle, title: &str, body: &str) {
    emit_capture_error(app, "general", "blocking_error", title, body);
    app.dialog()
        .message(body)
        .title(title)
        .kind(tauri_plugin_dialog::MessageDialogKind::Error)
        .blocking_show();
}

pub async fn confirm_async(app: &AppHandle, title: String, body: String) -> bool {
    let app_handle = app.clone();
    let (tx, rx) = tokio::sync::oneshot::channel();

    tauri::async_runtime::spawn(async move {
        let answer = app_handle
            .dialog()
            .message(body)
            .title(title)
            .kind(tauri_plugin_dialog::MessageDialogKind::Warning)
            .blocking_show();
        let _ = tx.send(answer);
    });

    rx.await.unwrap_or(false)
}
