# 07 · 配置模型与持久化

真相源：[`src-tauri/src/service/config/types.rs`](../src-tauri/src/service/config/types.rs)。
前端镜像：`src/store/useAppStore.ts` 的 `AppConfig` 接口（**有字段漂移，见 [08](08-AUDIT.md)**）。

## AppConfig（后端，serde，`#[serde(default)]`）

| 字段 | 类型 | 默认 | 说明 |
|------|------|------|------|
| `workflows` | `Vec<CaptureWorkflow>` | 2 条出厂工作流 | 见下 |
| `save_path` | `String` | — | 全局保存目录 |
| `language` | `String` | — | i18n |
| `font_family` | `String` | — | 标注字体 |
| `vello_enabled` | `bool` | `true` | 全局 Vello 开关 |
| `vello_advanced_effects` | `bool` | `true` | 高级效果 |
| `vello_aesthetic_style` | `AestheticStyle` | `Default` | Default/Neon/PaperCut/Sketch/Glass |
| `snapshot_enabled` | `bool` | — | 旧快照开关（**Legacy，迁移到 workflow**） |
| `snapshot_width/height` | `i32` | — | 旧快照尺寸（Legacy） |
| `selection_engine` | `String` | `"gdi"/"vello"` | 选区默认引擎（独立） |
| `snapshot_engine` | `String` | 同上 | 快照默认引擎（独立） |
| `theme` | `String` | `"system"` | light/dark/system |
| `accent_color` | `String` | `#3b82f6` | 强调色 |
| `jpg_quality` | `u8` | `90` | JPG 质量 |
| `concurrency` | `usize` | `4` | 并发度 |
| `default_export_format` | `String` | `"png"` | png/jpg |
| `quick_save` | `bool` | `false` | 快速保存 |
| `registration_errors` | `Vec<String>` | — | `skip_deserializing`，运行时回填热键冲突 |

## CaptureWorkflow

```rust
struct CaptureWorkflow {
    id: String,          // 系统项形如 "capture_default"；用户项 "user_<ts>"
    label: String,
    shortcut: String,    // "Alt+A"
    action: CaptureAction,
    output: CaptureOutput,
    enabled: bool,
    is_system: bool,     // true 不可删除，仅可编辑
}
```

### CaptureAction（`#[serde(tag="type", content="config")]`）

```rust
enum CaptureAction {
    Selection  { engine },
    Fullscreen { engine },
    Window     { engine },
    Snapshot   { engine, width, height, allow_resize },
}
```
`engine` = `"gdi"` | `"vello"`。模式行为见 [04-CAPTURE-MODES](04-CAPTURE-MODES.md)。

### CaptureOutput

```rust
struct CaptureOutput {
    save_to_file: bool,
    save_to_clipboard: bool,
    target_folder: Option<String>,  // 覆盖全局 save_path
    naming_template: String,         // chrono 格式，如 "%Y-%m-%d_%H-%M-%S"
    format: String,                  // "png" | "jpg"
}
```

## 持久化与迁移

- 加载/保存在 `service/config/io.rs` 与 `manager.rs`；任一改动（含用户手改 JSON）经 `io::load` 触发自愈迁移。
- 热键注册在 `service/config/hotkey.rs`；`register_all()` 返回 hotkey_map，冲突回填到 `registration_errors`。
- `snapshot_*` 为遗留字段，规划中迁移为标准 workflow（目前仍被 `AdvancedTab` 的"默认快照尺寸"读写）。

## 设置命令映射

每个可写字段对应一个 `set_*` 命令（见 [02-ARCHITECTURE](02-ARCHITECTURE.md) IPC 表）。`get_config` 返回完整 `AppConfig`，前端启动与每次设置变更后刷新到 store。
