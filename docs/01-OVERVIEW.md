# 01 · 概述

## 产品定位

NexSpot 是一款 **Windows 桌面截图与标注工具**，对标"微信截图级"的即时响应，并在其上叠加：

- **双渲染引擎**：GDI（极速、零延迟）与 Vello/WGC（GPU 加速、风格化效果）。
- **工作流（Workflow）驱动**：每个全局热键绑定一条可配置的采集流水线（模式 + 引擎 + 输出）。
- **原生覆盖层标注**：截图后直接在原生 Win32 覆盖层上绘制（矩形/椭圆/箭头/画笔/文字/序号等），不经 Webview。
- **PIN 贴图、滚动长截图、OCR** 等扩展能力。

## 技术栈

| 层 | 选型 | 说明 |
|----|------|------|
| 应用框架 | **Tauri v2** | Rust 主进程 + Webview 前端，IPC 通过 `invoke`/`emit` |
| 前端 | **React 18 + TypeScript + Vite** | 设置面板、仪表盘、活动中心、PIN/OCR/滚动等独立窗口 |
| 前端样式 | **Tailwind CSS v4** + CSS 变量令牌 | 工业风暗色主题，详见 [05-FRONTEND](05-FRONTEND.md) |
| 状态管理 | **Zustand** | `src/store/useAppStore.ts` |
| 后端核心 | **Rust + windows-rs (Win32)** | 直接操作 HWND/HDC/DXGI，规避 Webview 渲染延迟 |
| 采集 | **GDI BitBlt** / **Windows Graphics Capture (WGC)** | 双路采集 |
| 渲染 | **GDI/GDI+ UpdateLayeredWindow** / **Vello (wgpu/DirectX)** | 双路渲染 |
| 文本排版 | **Parley** (Vello 路径) | |
| 国际化 | **i18next** | `src/locales/{en,zh}.json` |

## 功能全清单与成熟度

> 图例：✅ 可用 · 🟡 部分可用/有已知缺陷 · 🧪 实验性 · 💤 代码存在但未接线 · 📝 规格已定未完整实现

| 功能 | 状态 | 入口 / 关键文件 |
|------|------|----------------|
| 区域选取截图 (Selection) | ✅ | `native_overlay/modes/selection.rs` |
| 全屏截图 (Fullscreen) | ✅ | `native_overlay/modes/fullscreen.rs` |
| 窗口捕获 (Window) | ✅ | `native_overlay/modes/window.rs` |
| 固定尺寸快照 (Snapshot) | ✅ | `native_overlay/modes/snapshot.rs` |
| GDI 引擎（采集+渲染） | ✅ | `native_overlay/capture/gdi.rs`, `render/mod.rs` |
| Vello/WGC 引擎（采集+渲染） | ✅ | `native_overlay/capture/wgc.rs`, `render/vello_engine/` |
| 工作流系统（热键→采集→输出） | ✅ | `service/workflow/mod.rs`, `service/config/` |
| 全局热键 + 冲突检测 | ✅ | `service/hotkey/`, `service/config/hotkey.rs` |
| 原生绘图工具栏 | ✅ | `render/toolbar/`, `render/drawing/` |
| 绘图工具：矩形/椭圆/直线/箭头/画笔/文字/序号/马赛克 | ✅ | `render/drawing/tools/`, `render/vello_engine/renderer/tools/` |
| 撤销 | ✅ | 工具栏 |
| 放大镜 (Magnifier) | ✅ | `native_overlay/magnifier.rs`, `render/magnifier.rs` |
| 保存到文件 / 复制到剪贴板 | ✅ | `native_overlay/save/{file,clipboard}.rs` |
| 命名模板 (`%Y%m%d` 等) | ✅ | `CaptureOutput.naming_template` |
| JPG 质量 / 默认导出格式 / 并发度 | ✅ | `AdvancedTab` + `service/config/` |
| Vello 风格化 (Default/Neon/PaperCut/Sketch/Glass) | 🟡 | `AestheticStyle` 已贯通配置与 `styles.rs`，各风格完整度不一 |
| PIN 贴图合集窗口 | ✅ | `service/pin.rs`, `src/components/Pin/` |
| 滚动长截图 (Scrolling) | 🧪 | `native_overlay/scrolling.rs`, `service/stitch.rs`, `ScrollingPreview.tsx` |
| OCR | 🧪 | `service/ocr.rs`, `OCRResultWindow.tsx` |
| 活动中心 (Activity Feed) | ✅ | `service/activity.rs`, `ActivityHub.tsx` |
| 系统托盘 | ✅ | `service/tray/mod.rs` |
| 多显示器 / 混合 DPI 处理 | ✅ | `win32/monitor.rs`，启动时检测并提示 |
| 主题 / 强调色 | ✅ | `set_theme` / `set_accent_color` |
| AI 截图宏（截图+提示词→AI→PIN 回流） | 💤 / 📝 | `service/ai/` + `AgentCoreTab.tsx` 存在，但**未注册为 Tauri 命令**，规格见 [06](06-TOOLBAR-TOOLS.md) |
| 捐赠页 | ✅ | `src/components/Settings/tabs/DonateTab.tsx`（未追踪，新增中） |

> 详细的"实现 vs 规格"差异、以及前后端字段漂移见 [08-AUDIT](08-AUDIT.md)。

## 性能基线（来自历史决策记录，待重新量测）

- 渲染延迟（指令→屏幕）：GDI 路径 **<5ms** 目标。
- UI 刷新：60FPS+（Win32 消息循环驱动）。
- 首帧采集：WGC 预热命中 ~0ms，未命中一次性捕获 ~100–200ms。
- 内存：空闲 ~30MB，截图态视分辨率（通常 <100MB）。

> 这些数值源自归档文档，未在本次核查中复测，作参考。
