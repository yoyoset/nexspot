use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_notification::NotificationExt;

pub fn notify_warn(app: &AppHandle, title: &str, body: &str) {
    let _ = app.notification().builder().title(title).body(body).show();
}

pub fn notify_error(app: &AppHandle, title: &str, body: &str) {
    let _ = app.notification().builder().title(title).body(body).show();
}

pub fn notify_error_blocking(app: &AppHandle, title: &str, body: &str) {
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
