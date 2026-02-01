---
name: tauri-patterns
version: 1.0.0
description: >
  Tauri 桌面应用开发模式，包括前后端通信、窗口管理、系统 API 调用。
---

# 🖥️ Tauri Patterns

## 1. 项目结构

```
project/
├── src/              # React 前端
├── src-tauri/        # Rust 后端
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands/ # Tauri commands
│   │   └── services/ # 业务逻辑
│   └── tauri.conf.json
```

---

## 2. 前后端通信

### Rust → JS (Event)

```rust
// Rust 端
window.emit("event-name", payload)?;

// JS 端
import { listen } from '@tauri-apps/api/event';
await listen('event-name', (event) => {
    console.log(event.payload);
});
```

### JS → Rust (Command)

```typescript
// JS 端
import { invoke } from '@tauri-apps/api/tauri';
const result = await invoke('command_name', { arg1: 'value' });

// Rust 端
#[tauri::command]
fn command_name(arg1: String) -> Result<String, String> {
    Ok(format!("Received: {}", arg1))
}
```

---

## 3. 窗口管理

### 创建新窗口

```rust
tauri::WindowBuilder::new(
    app,
    "label",
    tauri::WindowUrl::App("index.html".into())
)
.title("Window Title")
.inner_size(800.0, 600.0)
.build()?;
```

### 窗口属性

```rust
window.set_always_on_top(true)?;
window.set_decorations(false)?;
window.set_transparent(true)?;
```

---

## 4. 状态管理

```rust
// 定义状态
struct AppState {
    db: Mutex<Database>,
}

// 注册状态
fn main() {
    tauri::Builder::default()
        .manage(AppState { db: Mutex::new(db) })
        .run()?;
}

// 在 command 中使用
#[tauri::command]
fn query(state: State<'_, AppState>) -> Result<Vec<Data>, String> {
    let db = state.db.lock().unwrap();
    // ...
}
```

---

## 5. 系统 API

### 截图 (Windows Graphics Capture)

```rust
use windows::Graphics::Capture::*;

// 创建 capture session
let item = GraphicsCaptureItem::CreateFromMonitor(monitor)?;
let session = Direct3D11CaptureFramePool::Create(...)?;
```

### 全局快捷键

```rust
use tauri::GlobalShortcutManager;

app.global_shortcut_manager()
    .register("CmdOrCtrl+Shift+S", || {
        // 处理快捷键
    })?;
```

---

## 6. 构建优化

### tauri.conf.json

```json
{
  "build": {
    "beforeBuildCommand": "npm run build",
    "beforeDevCommand": "npm run dev"
  },
  "tauri": {
    "bundle": {
      "active": true,
      "targets": ["msi", "nsis"]
    }
  }
}
```
