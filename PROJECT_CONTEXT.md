# PROJECT_CONTEXT.md: HyperLens 长期记忆与决策白皮书

本项目旨在打造一款 **“WeChat-级”** 高性能 Windows 截图工具。本文件固化了项目至今的关键设计决策与技术细节，作为后续 AI 编程的持久化上下文。

---

## 1. 技术栈选型 (Tech Stack)

| 维度 | 选型 | 决策依据 |
| :--- | :--- | :--- |
| **基础框架** | **Tauri v2** | 利用 Rust 的物理性能，同时保持 Webview 编写 GUI 的灵活性。 |
| **GUI (前端)** | **React + Vite** | 成熟的组件化能力，用于设置面板与工具栏。 |
| **核心引擎 (后端)** | **Rust (Win32 API)** | 必须直接操作句柄以实现零延迟和多显示器精准控制。 |
| **捕获技术** | **DXGI / WGC** | 使用 `Windows.Graphics.Capture` (Windows 10+)，支持 GPU 加速且无需 UAC 权限。 |
| **渲染技术** | **GDI / GDI+** | 选用原生渲染而非 Webview Canvas，以消除 Webview 渲染管道带来的 ~16ms-32ms 延迟。 |

---

## 2. 核心架构决策 (Architecture Decisions)

### A. 零延迟原生覆盖层 (Zero-Latency Native Overlay)

为了实现按下快捷键后“瞬间”进入截图状态，我们抛弃了传统的“展示 Webview 窗口”方案：

* **决策**: 创建双层架构。底层使用 `WS_EX_LAYERED` 原生窗口配合 `UpdateLayeredWindow` 进行 GDI 渲染。
* **效果**: 渲染延迟从 >30ms 降低至 **<5ms** (指令到屏幕肉眼不可见)。

### B. 状态机 SSOT (OverlayState)

* **决策**: 维护一个受 `Arc<Mutex<OverlayState>>` 保护的核心状态。
* **流转**: `Capture` -> `Interaction` -> `Render`。
* **重构**: 将几何吸附计算 (`snapping`) 和输入处理 (`interaction`) 逻辑从主控制器拆分，确保模块职责单一。

### C. 智能磁吸 (Magnetic Snapping)

* **决策**: 调用 `DwmGetWindowAttribute` 获取包含 `DWMWA_EXTENDED_FRAME_BOUNDS` 的视觉包围盒。
* **解决问题**: 完美避开了 Windows 10/11 窗口透明阴影导致的所谓“幽灵边界”缩放问题。

---

## 3. 性能指标 (Performance Baseline)

* **捕获延迟 (Capture Latency)**: **~240ms** (全系统窗口枚举 + WGC 捕捉 + 亮度底图预生成)。
  * *优化方向*: 窗口拓扑异步增量更新，减少单次 `EnumWindows` 时间。
* **渲染帧率 (UI Refresh Rate)**: 稳定 **60FPS+** (由 Win32 消息循环驱动)。
* **内存占用 (Memory Footprint)**: 初始状态约 **30MB**，截图状态视显示器分辨率而定 (通常 <100MB)。

---

## 4. 关键文件索引 (Core File Index)

* [src-tauri/src/service/native_overlay/state.rs](file:///f:/my%20ai/HyperLens/src-tauri/src/service/native_overlay/state.rs) - **数据结构定义**。
* [src-tauri/src/service/native_overlay/interaction.rs](file:///f:/my%20ai/HyperLens/src-tauri/src/service/native_overlay/interaction.rs) - **状态机转换逻辑**。
* [src-tauri/src/service/native_overlay/render/mod.rs](file:///f:/my%20ai/HyperLens/src-tauri/src/service/native_overlay/render/mod.rs) - **原生 GDI 绘图管线**。
* [docs/architecture_panorama.md](file:///f:/my%20ai/HyperLens/docs/architecture_panorama.md) - **中英双语架构全景图**。

---

## 5. 项目约束 (Project Constraints)

1. **Strict Win32 Environment**: 所有的辅助计算必须考虑 Windows 的物理像素与 DPI 缩放。
2. **Zero-Placeholders**: 不允许使用占位符，所有生成的代码必须是可编译的生产级代码。
3. **Bilingual Compliance**: 所有技术文档保持中英双语，确保协作清晰。

---
*Last Verified: 2026-02-02 (Core Module Refactoring Complete)*
