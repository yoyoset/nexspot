# 04 · 四种采集模式（独立）

采集模式与引擎**正交**：任意模式都可在 GDI 或 Vello 引擎下运行（由 workflow 的 `engine` 决定）。模式之间通过统一的 `CaptureModeHandler` trait 隔离，互不影响。

## 抽象：CaptureModeHandler

```rust
// native_overlay/modes/mod.rs
pub trait CaptureModeHandler {
    fn prepare_selection(
        &self, x, y, w, h,        // 采集区域（虚拟桌面/显示器）
        mouse_x, mouse_y,         // 触发时光标位置
        window_rects: &[RECT],    // 当前可见窗口包围盒列表
    ) -> Option<RECT>;            // 返回预设选区；None = 由用户手动拖拽
}
```

每种模式只负责"采集后如何初始化选区"，后续交互/渲染/保存路径完全共用。

## 模式行为

| 模式 | 枚举 | `prepare_selection` 行为 | 文件 |
|------|------|--------------------------|------|
| **区域选取** | `CaptureAction::Selection { engine }` | 返回 `None` —— 不预设，用户手动拖拽框选 | `modes/selection.rs` |
| **全屏** | `CaptureAction::Fullscreen { engine }` | 返回整个采集区域 `(x,y,x+w,y+h)`，跳过拖拽 | `modes/fullscreen.rs` |
| **窗口捕获** | `CaptureAction::Window { engine }` | 在 `window_rects` 中找**包含光标且面积最小**（>100px²）的窗口作为选区；找不到回退全屏 | `modes/window.rs` |
| **固定快照** | `CaptureAction::Snapshot { engine, width, height, allow_resize }` | 以光标（或屏幕中心）为中心放置 `width×height` 矩形，并夹紧到采集区域内 | `modes/snapshot.rs` |

> ⚠ 历史漂移：归档的采集审计曾把 Fullscreen / Window 标为"占位符，行为等同区域选取"。**本次核查确认两者已实现**（见上表），审计文档已过时。

## 模式 × 引擎 默认配置

- `config.selection_engine`：区域选取的默认引擎。
- `config.snapshot_engine`：快照的默认引擎。
- 这两项**相互独立**——可以让选区走 Vello（风格化），快照走 GDI（极速），或任意组合。
- 单条工作流的 `action.engine` 覆盖全局默认。

## 窗口检测的数据来源

`window_rects` 由 `win32/window/enumeration.rs` 枚举可见顶层窗口产生，并经 `snapping.rs` 用 `DwmGetWindowAttribute(DWMWA_EXTENDED_FRAME_BOUNDS)` 取**视觉包围盒**，规避 Win10/11 透明阴影导致的"幽灵边界"。窗口模式与磁吸吸附复用同一套包围盒数据。

## 默认工作流（出厂）

来自 `config/types.rs::default_workflows()`：

| id | 标签 | 热键 | 模式/引擎 | 输出 |
|----|------|------|-----------|------|
| `capture_default` | Capture Selection | `Alt+A` | Selection / gdi | 存文件+剪贴板, png |
| `snapshot_default` | Snapshot | `Alt+S` | Snapshot 800×600 (可缩放) / gdi | 存文件+剪贴板, png |

两条均为 `is_system=true`（不可删除，仅可编辑）。
