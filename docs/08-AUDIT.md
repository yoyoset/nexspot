# 08 · 核查发现（2026-06-04）

本次对 `main`（`ce78ce1`）的全面核查记录。分两类：**文档漂移**（旧文档已与代码脱节，已通过重写本套文档解决）与 **代码漂移/隐患**（代码内部前后端不一致或未接线，需后续处理）。

---

## A. 文档漂移（已通过重写解决）

| 旧文档 | 问题 | 处置 |
|--------|------|------|
| `PROJECT_CONTEXT.md` | 文件索引全部失效（`state.rs`/`interaction.rs`/`render/mod.rs` 旧路径，且 `architecture_panorama.md` 已不在 docs 根） | 归档，由 [02](02-ARCHITECTURE.md) 取代 |
| `ARCHITECTURE_REFERENCE.md` | 概念正确但模块路径过时；File Map 与现结构不符 | 归档，由 [02](02-ARCHITECTURE.md)/[03](03-ENGINES.md) 取代 |
| `ENGINE_REFERENCE.md` | 技术结论仍成立，但 File Map 指向 `manager.rs`/`capture.rs`/`save.rs`/`mouse_move.rs`/`overlay_state.rs` 等已被拆成目录的旧单文件 | 归档，更新版见 [03](03-ENGINES.md) |
| `CAPTURE_PIPELINE_AUDIT.md` | 多数 P1/P2/P3 问题已修复；尤其 **Fullscreen/Window 已实现**，不再是占位符 | 归档（历史），现状见 [04](04-CAPTURE-MODES.md) |
| `TOOLS.md` | 是产品规格而非现状文档 | 归档，整理为 [06](06-TOOLBAR-TOOLS.md) |

---

## B. 代码漂移 / 隐患（待处理）

### B1 · 前端调用了未注册的 Tauri 命令（会运行时报错）

`src/hooks/useConfig.ts` 中存在两个 `invoke`，但对应命令**不在** `lib.rs` 的 `invoke_handler` 列表：

> **✅ B1 / B2 / B6 已于 Phase 1（2026-06-04）修复**，详见各条目状态。

### B1 · 前端调用了未注册的 Tauri 命令（会运行时报错）— ✅ 已修复

| 前端调用 | 原状态 | 处置 |
|----------|--------|------|
| `invoke("set_ocr_engine")` | ❌ 未注册 + 无控件调用（死管线） | **删除** `setOcrEngine` 及其 prop 传递链（useConfig / SettingsPanel / GeneralTab） |
| `invoke("set_indicator_color")` | ❌ 未注册 + `--color-indicator` 无消费者 + 与强调色重复 | **删除** 指示器颜色功能（StyleTab 区块 + useConfig + App.tsx + store 字段 + 文案键） |
| `invoke("get_startup_errors")` | ❌ 未注册（见 B6） | **后端补全并注册**（闭合启动竞态） |

### B2 · 前后端 AppConfig 字段不对齐 — ✅ 已修复

| 字段 | 处置 |
|------|------|
| `indicator_color` | 前端字段 + `--color-indicator` 写入已删除（功能两端皆为半成品，与强调色重复） |
| `ocr_engine` | 幽灵字段及其乐观写入已删除 |
| `quick_save` | Phase 0 提交已贯通前后端（GeneralTab 开关 + `set_quick_save`） |
| `language` | 经由 `set_language` 持久化（GeneralTab 切换调用），保留 |

### B6 · 启动错误补取命令缺失 — ✅ 已修复（Phase 1 新发现）

`TauriEventListener.tsx:41` 在挂载时 `invoke("get_startup_errors")`，但后端从未注册该命令 → **每次启动都抛错**（被 try/catch 吞掉，仅 console 报错）。
该调用是"挂载后补取启动错误"的兜底：后端在 `setup()` 中 `emit("shortcut-startup-error")` 可能早于前端注册监听器（启动竞态），此 fetch 用于补上漏掉的那次。
**处置**：在 `commands/config/mod.rs` 实现 `get_startup_errors`（返回 `ConfigState.last_registration_errors`）并在 `lib.rs` 注册。属"补全真机制"，非删除。

### B3 · AI 子系统未接线（💤）— ✅ 已移除（Phase 2，2026-06-04）

原状：`service/ai/{mod,openai}.rs` 与 `AgentCoreTab.tsx` 均存在但完全孤立（无命令注册、无任何 import、未挂入 SettingsPanel）。
**决策**：产品方向定为"专注截图、移除 AI"。已删除：
- 后端 `service/ai/`（mod.rs + openai.rs）及 `service/mod.rs` 的 `pub mod ai;`
- 前端孤儿组件 `AgentCoreTab.tsx`
- 中英 locale 的全部 AI 文案（settings.tabs.ai、顶层 `ai` 块、`tools.ai_directive`、`notification.ai_start_failed_title`）

遗留尾巴（低优先，下次顺手清）：`GlobalHUD` 的 `HUDType` 仍含未使用的 `'ai'` 分支；`Cargo.toml` 的 `reqwest`/`async-trait`/`futures` 若不再被 ocr/stitch 使用可一并精简（本次保守未动）。

### B4 · 未追踪文件

- `src/components/Settings/tabs/DonateTab.tsx`、`src/assets/`、`src/types/images.d.ts` 为新增、未纳入 git（`git status` 中 `??`）。
- 工作区有大量已修改但未提交的 `M` 文件（见会话起始 git status）。**重设计前建议先提交或明确取舍**，避免与重设计改动混淆。

### B5 · 遗留快照配置

`AppConfig.snapshot_enabled / snapshot_width / snapshot_height` 标注为 Legacy（应迁移到 workflow），但 `AdvancedTab` 仍在读写"默认快照尺寸"。需决定：完成迁移并移除，或正式保留为"新建快照工作流的默认值"。

---

## C. 建议的处理顺序

1. **重设计前先清账**：提交/取舍 B4 的工作区改动。
2. **修 B1/B2**：让前后端配置契约一致（这是重设计页面会直接踩到的坑）。
3. **定 B3**：AI 是做还是砍——影响 PIN 窗与设置页的信息架构。
4. **再做 UI 重设计**（见 `docs/DESIGN-PROMPT.md`）。
