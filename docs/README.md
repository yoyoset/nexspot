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
| 10 | [商业化交付审查 COMMERCIAL-RELEASE-REVIEW](10-COMMERCIAL-RELEASE-REVIEW.md) | 商业试用结论、发版产物、自动/人工验收门禁、剩余阻断项 |
| 11 | [发布证据 RELEASE-EVIDENCE](11-RELEASE-EVIDENCE.json) | 机器可读发版证据：产物路径、SHA256、签名状态、chunk 与剩余人工门禁 |
| 12 | [人工 QA 矩阵 MANUAL-QA-MATRIX](12-MANUAL-QA-MATRIX.md) | 商业人工验收矩阵：Vello OCR、滚动截图、多屏/DPI、安装器 smoke 与未签名 pilot 说明 |
| 13 | [启动 Smoke 证据 SMOKE-EVIDENCE](13-SMOKE-EVIDENCE.json) | 机器可读 EXE 启动 smoke 证据：启动时长、当前 EXE SHA256、进程结果 |
| 14 | [安装器 Smoke 证据 INSTALLER-SMOKE-EVIDENCE](14-INSTALLER-SMOKE-EVIDENCE.json) | NSIS 安装、覆盖安装、已安装应用启动、卸载和清理证据 |
| 15 | [未签名试点分发说明 UNSIGNED-PILOT-DISTRIBUTION-NOTICE](15-UNSIGNED-PILOT-DISTRIBUTION-NOTICE.md) | 未签名 EXE/安装包的试点分发口径、SHA256 与 public stable No-Go 条件 |
| 16 | [商业 QA 缺陷日志 COMMERCIAL-QA-ISSUE-LOG](16-COMMERCIAL-QA-ISSUE-LOG.md) | Public stable 阻断项、试点观察项、关闭条件与已闭环证据 |
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
