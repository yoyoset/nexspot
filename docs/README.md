# NexSpot 文档中心

> **最后核查：** 2026-06-12（v0.3.0 状态固化，见 [09-STATUS](09-STATUS.md)）
> **状态：** 2026-06-04 全面核查重写本套文档（旧版归档于 [`archive/2026-06-04-pre-redesign/`](archive/2026-06-04-pre-redesign/)）；
> 其后完成 Studio UI 重设计与 OCR 子系统重建，增量收口于 09。

NexSpot 是一款面向 Windows 的高性能截图工具（Tauri v2 + Rust/Win32 后端 + React/TS 前端），核心目标是"零延迟、双引擎、工业级"的截图与标注体验。

---

## 文档导航

| # | 文档 | 内容 |
|---|------|------|
| 01 | [概述 OVERVIEW](01-OVERVIEW.md) | 产品定位、技术栈、功能全清单、各功能成熟度 |
| 02 | [架构 ARCHITECTURE](02-ARCHITECTURE.md) | 系统拓扑、真实模块树、IPC 命令表、数据流 |
| 03 | [引擎 ENGINES](03-ENGINES.md) | GDI / Vello(WGC) 双引擎、双 HWND 隔离、采集管道、DPI/坐标 |
| 04 | [采集模式 CAPTURE-MODES](04-CAPTURE-MODES.md) | Selection / Fullscreen / Window / Snapshot 四种独立模式 |
| 05 | [前端 FRONTEND](05-FRONTEND.md) | React 页面结构、状态仓库、设计令牌、多窗口路由 |
| 06 | [工具栏工具 TOOLBAR-TOOLS](06-TOOLBAR-TOOLS.md) | 原生工具栏每个工具的交互规格（绘图/选中/PIN/OCR/滚动） |
| 07 | [配置 CONFIG](07-CONFIG.md) | AppConfig / Workflow 数据模型与持久化、迁移 |
| 08 | [核查发现 AUDIT](08-AUDIT.md) | 2026-06-04 核查记录（行动项已闭环，保留为审计存档） |
| 09 | [状态固化 STATUS](09-STATUS.md) | **当前状态快照**：v0.3.0 已完成项、硬 bug 根因表、Phase 4 待办 |
| ★ | [设计交付 design/](design/2026-06-04-studio-redesign/) | Studio 方向 UI 重设计交付包（已落地实现的基准） |

---

## 阅读建议

- **接手项目 / 复刻架构** → 先读 01 → 02 → 03。
- **改前端 / 重设计页面** → 读 05 + 06，配合 08 的漂移清单。
- **改采集/渲染内核** → 读 03 + 04。
- **历史决策溯源** → 见 `archive/`。

---

## 约定

1. **真实优先**：本套文档只描述代码中**实际存在**的内容。计划中但未实现的功能在 01 的成熟度表中明确标注，不混入主体描述。
2. **路径可点击**：文中引用源文件均使用仓库相对路径。
3. **漂移单独成章**：发现的"文档曾经描述但代码已变"的问题统一收录于 08，不污染各专题文档。
