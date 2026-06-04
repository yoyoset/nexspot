# 05 · 前端

> React 18 + TypeScript + Vite + Tailwind v4 + Zustand + framer-motion + i18next。
> 重设计页面时这是主要改造对象，配合 [08-AUDIT](08-AUDIT.md) 的字段漂移清单一起看。

## 窗口与路由

前端是**单 HTML 多窗口**架构：同一个 `index.html` 被 Tauri 以不同 `window.location.hash` 加载成不同窗口。`App.tsx` 顶部按 hash 分流：

| hash | 渲染 | 组件 |
|------|------|------|
| （无） | 主窗口 | `Navigator` + `Dashboard`/`ActivityHub`/`SettingsPanel` |
| `#pin-collection` | PIN 合集窗 | `PinCollectionWindow` |
| `#scrolling-preview` | 滚动长截图预览 | `ScrollingPreview` |
| `#ocr-result` | OCR 结果 | `OCRResultWindow` |

所有窗口都叠了全局 `GlobalHUD`（toast 反馈）。主窗口额外有 `StartupErrorToast`、`EngineErrorModal`、`WorkflowModal`、`TauriEventListener`。

## 主窗口三大页（Tab 由本地 state 切换）

| Tab | 组件 | 职责 |
|-----|------|------|
| `dashboard` | `Dashboard.tsx` | 工作流列表（工业控制台风：每行=标识/规格/格式/状态灯/目录/热键/操作），ZAP 手动触发、编辑、删除、新建 |
| `activity` | `ActivityHub.tsx` | 左：实时活动 feed（截图/OCR/滚动）；右：各工作流的"存储池"快捷入口。监听 `activity://updated` |
| `settings` | `SettingsPanel.tsx` | 侧栏 5 个子 Tab（见下） |

`Navigator.tsx`：40px 宽垂直活动栏（仅图标 + hover tooltip），底部一个"PIN 置顶"锁开关（`toggle_pin_always_on_top`）。

### 设置子 Tab

| id | 组件 | 内容 |
|----|------|------|
| `general` | `GeneralTab` | 保存路径、字体、语言、OCR 引擎等基础项 |
| `workflows` | `WorkflowsTab` | 工作流增删改（与 `WorkflowModal`/`WorkflowForm` 联动） |
| `advanced` | `AdvancedTab` | 性能与质量（JPG 质量/默认格式/并发/快照尺寸）+ Vello 开关与 5 种风格 + 高级效果 + 日志维护 |
| `style` | `StyleTab` | 主题/强调色等外观 |
| `donate` | `DonateTab` | 捐赠页（新增，未追踪入 git） |

## 状态仓库（`src/store/useAppStore.ts`）

Zustand store，核心切片：
- `config: AppConfig | null` —— 镜像后端配置（结构见 [07-CONFIG](07-CONFIG.md)）。
- `velloStatus: 'pending'|'ready'|'failed'` + `velloError` + `velloErrorModal`。
- `activity: ActivityEntry[]` + `fetchActivity()`（invoke `get_activity`）。
- `hud`（消息/类型/可见）+ `showHUD/hideHUD`。
- `settingsNavigation`（深链到某 Tab/工作流）、`workflowEditing`（模态）、`dashboardCollapsible`（localStorage 持久化）、`startupErrors`。

配置读写封装在 `src/hooks/useConfig.ts`（各 `set_*` 命令 + 乐观更新）。

## 设计令牌（`src/index.css`）

当前为**工业暗色主题**，CSS 变量 + Tailwind v4 `@theme` 映射：

```
强调色   --color-accent           #3b82f6（用户可改）
背景     --color-bg-main          #09090b (zinc-950)
         --color-bg-card          #121214
         --color-bg-subtle        #18181b (zinc-900)
         --color-bg-sidebar       #09090b
边框     --color-border-subtle    #27272a (zinc-800)
         --color-border-hover     #3f3f46 (zinc-700)
文字     --color-text-main        #e2e2e7 (zinc-200)
         --color-text-muted       #a1a1aa (zinc-400)
圆角     --radius-industrial      2px      --radius-button 4px
间距     xs/sm/md/lg = 4/8/12/16px
```

- 亮色主题：`[data-theme='light']` 覆盖同名变量。主题切换逻辑在 `App.tsx`（含 `system` 跟随）。
- 字体：正文 `Inter`，技术信息用 `.tech-text`（等宽，`letter-spacing:-0.02em`）。
- 标志性样式：`.industrial-panel`、`.tech-badge`、`.noise-bg`（2% 噪点）、`.animate-breathing`（呼吸点）、3–4px 细滚动条。
- 现风格关键词：高密度、小字号（8–12px 居多）、大量 `uppercase` + `tracking`、极小圆角、细边框替代阴影、状态用色点/色条。

> 重设计时若要换风格，主要改 `index.css` 令牌 + 各组件 Tailwind 类。令牌已集中，换肤成本可控。

## 国际化

`src/i18n.ts` + `src/locales/{en,zh}.json`。组件内全部走 `t('key')`。新增文案需同步两份 locale。
