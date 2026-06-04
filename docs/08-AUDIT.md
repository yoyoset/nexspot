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

| 前端调用 | 位置 | 后端状态 |
|----------|------|----------|
| `invoke("set_ocr_engine", { engine })` | `useConfig.ts:62`（`setOcrEngine`，由 `GeneralTab` 使用） | ❌ 未注册 → 调用必抛错 |
| `invoke("set_indicator_color", { color })` | `useConfig.ts:192`（`setIndicatorColor`） | ❌ 未注册 → 调用必抛错 |

**建议**：要么在后端实现并注册这两个命令（并在 `AppConfig` 增字段），要么从前端移除相关 setter/调用。

### B2 · 前后端 AppConfig 字段不对齐

| 字段 | 后端 `types.rs` | 前端 `useAppStore.ts` | 备注 |
|------|----------------|----------------------|------|
| `indicator_color` | ❌ 无 | ✅ 有（`App.tsx` 还会 `setProperty('--color-indicator')`） | 前端读了一个后端不下发的字段 → 永远 undefined |
| `ocr_engine` | ❌ 无 | ❌ 接口无，但 `setOcrEngine` 乐观写入该键 | 幽灵字段 |
| `quick_save` | ✅ 有 + `set_quick_save` 命令 | ❌ 接口未声明，无对应 setter | 后端能力前端未暴露 |
| `language` | ✅ 有 + `set_language` 命令 | ✅ 有，但 `useConfig` 无 `setLanguage` | 切换语言入口在别处（i18n），确认是否真正持久化 |

**建议**：以后端 `types.rs` 为准，统一前端 `AppConfig` 接口；删除 `indicator_color`/`ocr_engine` 或在后端补齐。

### B3 · AI 子系统未接线（💤）

`service/ai/{mod,openai}.rs` 与前端 `src/components/Settings/tabs/AgentCoreTab.tsx` 均存在，但：
- 无任何 AI 相关命令注册到 `invoke_handler`。
- `SettingsPanel` 的 Tab 列表当前为 general/workflows/advanced/style/donate，**未挂 AgentCoreTab**。

**结论**：AI 截图宏（[06](06-TOOLBAR-TOOLS.md)）目前不可用。需决策：补全接线，或移除死代码。注：归档的前端重设计 spec（`docs/superpowers/specs/2026-04-11-frontend-compact-redesign.md`）原本主张**删除全部 AI 组件**——与 `TOOLS.md` 中"AI 回流到 PIN"的产品愿景**相互冲突**，需产品决策定调。

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
