---
name: rust-expert
version: 1.0.0
description: >
  Rust 开发最佳实践，包括错误处理、内存安全、模块化设计。
  针对 Tauri 桌面应用场景优化。
---

# 🦀 Rust Expert

## 1. 代码组织

### 模块结构

```rust
src/
├── main.rs          // 入口，只做初始化
├── lib.rs           // 公共导出
├── commands/        // Tauri commands
├── services/        // 业务逻辑
├── models/          // 数据结构
└── utils/           // 工具函数
```

### 规则

- ✅ 每个模块职责单一
- ✅ 公共 API 通过 `lib.rs` 导出
- ❌ 禁止在 `main.rs` 写业务逻辑

---

## 2. 错误处理

### 使用 `thiserror` + `anyhow`

```rust
// 定义错误类型
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Capture failed: {0}")]
    Capture(String),
}

// 在函数中使用
fn capture() -> Result<(), AppError> {
    // ...
}
```

### 规则

- ✅ 永远使用 `Result<T, E>`，不用 `unwrap()`
- ✅ 只在测试或确定安全时用 `expect()`
- ❌ 禁止 `panic!` 在生产代码

---

## 3. 异步处理

### Tokio 最佳实践

```rust
#[tokio::main]
async fn main() {
    // 主线程只做调度
}

// CPU 密集任务用 spawn_blocking
tokio::task::spawn_blocking(|| {
    heavy_computation();
}).await?;
```

### 规则

- ✅ IO 操作用 `async`
- ✅ CPU 密集用 `spawn_blocking`
- ❌ 禁止在 async 函数中阻塞

---

## 4. 内存安全

### 所有权检查清单

- [ ] 避免不必要的 `clone()`
- [ ] 优先使用 `&str` 而非 `String`
- [ ] 大结构体传引用 `&T`，小结构体传值
- [ ] 使用 `Arc<Mutex<T>>` 跨线程共享

---

## 5. Tauri 特定

### Command 定义

```rust
#[tauri::command]
async fn my_command(state: State<'_, AppState>) -> Result<String, String> {
    // 业务逻辑不写这里，调用 service
    services::do_something(&state).await
        .map_err(|e| e.to_string())
}
```

### 规则

- ✅ Command 只做参数解析和调用 service
- ✅ 错误转换为 `String` 返回前端
- ❌ 禁止在 command 里写复杂逻辑
