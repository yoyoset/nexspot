# Handoff: NexSpot — UI 重设计（方向：Studio / 现代工作室）

## Overview
NexSpot 是一款 Windows「微信截图级」即时截图 + 原生标注工具（Tauri v2：Rust/Win32 内核 + React 18 / TS / Tailwind v4 / Zustand 的 Webview）。本包是它**全部界面**在 **Studio（现代工作室）** 设计方向下的高保真重设计：主窗口三页、三个悬浮辅助窗、以及原生标注工具栏的视觉规格。

设计方向定位：**Linear / Raycast 一脉的现代克制**——柔和圆角、以阴影与半透明玻璃制造层次（而非发丝线）、单一沉静的强调色。专业但不冷硬，呼应 Vello/Glass 渲染引擎的质感卖点。**本设计不含任何 AI 元素。**

## About the Design Files
本包内 `prototype/` 下的文件是用 **HTML + React(CDN) + 原生 CSS 变量** 写成的**设计参考原型**，用来表达预期的外观与交互，**不是用来直接拷进生产环境的代码**。任务是：**在目标代码库的现有环境里（React 18 + TypeScript + Tailwind v4 + Zustand）按这些设计重新实现**，沿用其既有组件与约定。原型里没用 Tailwind（为了在单文件里稳定预览），但所有视觉值都已在 `tokens.css` 里映射成 Tailwind v4 token，可直接落地。图标在原型里用 lucide UMD，生产环境改用 **lucide-react**（图标名一致，kebab→PascalCase，如 `layout-dashboard` → `<LayoutDashboard/>`）。克制的动效可用 framer-motion。

## Fidelity
**High-fidelity（hifi）。** 颜色、字号、间距、圆角、阴影、交互态都是最终值，请按 token 像素级还原。唯一的占位是图片内容区（截图缩略图 / 长图 / 源图），原型里用斜纹 placeholder 表示，生产环境填真实位图。

---

## Design Tokens
完整、可直接 import 的版本见 **`tokens.css`**（Tailwind v4 `@theme inline` + 运行时 CSS 变量）。要点：

- **主题**：`<html data-theme="dark|light">`，全部颜色走 CSS 变量；`system` 用 `matchMedia('(prefers-color-scheme: dark)')` 解析。
- **强调色**：用户可改，**只有一个源变量 `--accent`**；`--accent-press`、`--on-accent`（自动对比：accent 亮度 > 0.55 用深字，否则白字）在 JS 里派生（见 `stores.ts` 末尾 `applyTheme`）。预设 6 色：Periwinkle `#7a6ff2`(默认) / Blue `#4f8cff` / Teal `#16b8a6` / Amber `#f59e0b` / Rose `#f4517b` / Green `#46b86a`。

| 角色 | Dark | Light |
|---|---|---|
| bg0 (桌面/最底) | `#0c0d10` | `#f5f5f7` |
| bg1 (窗口/面板) | `#141519` | `#ffffff` |
| bg2 (卡片) | `#1b1c21` | `#f1f1f4` |
| bg3 (悬浮/输入) | `#23242b` | `#ffffff` |
| border / border2 | `rgba(255,255,255,.09 / .14)` | `rgba(0,0,0,.10 / .16)` |
| text / muted / faint | `#ececef` / `#9a9aa5` / `#63636d` | `#1b1c1f` / `#6c6c75` / `#a0a0a8` |
| ok (引擎就绪) | `#57d9a3` | `#1f9d6b` |
| warn (引擎未就绪) | `#f5c451` | `#bf8a14` |
| bad (热键冲突) | `#f76d6d` | `#dc4d4d` |
| glass (悬浮窗) | `rgba(20,21,25,.74)` | `rgba(255,255,255,.78)` |

- **soft 变体**（状态底色）：`color-mix(in srgb, <status> 18%, transparent)`（light 略低）。`--accent-soft = mix(accent 16%)`，`--accent-line = mix(accent 38%)`。
- **圆角**：按钮 `9px` · 面板/卡片 `12px` · 窗口/大卡 `16px` · pill `999px` · 原生工具栏 `13px` / 工具按钮 `8px`。
- **间距刻度**（px）：`6 / 10 / 16 / 24 / 36`。
- **阴影**（暗色）：sm `0 2px 8px -2px rgba(0,0,0,.5)` · float `0 24px 60px -20px rgba(0,0,0,.78), inset 0 2px 0 rgba(255,255,255,.04)`。**用阴影 + 半透明制造层级，不要用发丝线堆叠。**
- **字体**：UI = **Manrope**（400/500/600/700/800）；技术元数据（热键、格式、引擎名、路径、尺寸、坐标）= **JetBrains Mono**。
- **字号/字重**：页标题 17/800 · 区块标题 15/800 · 正文 12.5–13.5/600 · 次要 11–12/400-600 · 元数据 mono 10–11.5。letter-spacing 标题 `-.02em`，mono 标签 `.04–.16em`。**禁止整屏大写+宽字距**（那是另一个方向）。

---

## Screens / Views

### 全局：主窗口外壳
- **无边框桌面窗**（非网页），紧凑（设计稿约 **960 × 620**，可缩放）。圆角 16，`--shadow-float`。
- **自定义标题栏** 高 38：左 16px 渐变 logo + `NexSpot`（12.5/700）+ mono 副标（如 `· 置顶`）；右 Windows 三键（最小化 `minus` / 最大化 `square` / 关闭 `x`，27×26，hover 底 bg2，关闭键 hover 变 `--bad` 白字）。整条 `-webkit-app-region: drag`。
- **活动栏 Rail**（左，宽 **48**，底色 bg0）：纯图标按钮 34×34，圆角 9；图标 18px。三页 `layout-dashboard / activity / settings`。**激活态**：底 `--accent-soft`、图标 `--accent`、左缘 3×18 圆角竖条 `--accent`。hover：bg2。每个按钮 `data-tip` 悬浮提示（右侧 42px，bg3 + border2 + shadow）。底部弹性留白后是**窗口置顶**开关（`pin` / `pin-off`，激活 = accent）。

### 1. Dashboard / 工作流列表（最核心页）
- **Header**（padding 16/22，下边框）：左 `工作流`(17/800) + 副标 `N 条 · 每个热键就是一条流水线`(11.5, muted)。紧跟**一眼扫描汇总 pill**：有冲突时红底 pill `⚠ N 个热键冲突`（`--bad-soft`），有未就绪时黄底 pill `● N 个引擎未就绪`（`--warn-soft`）。右侧：`预览空态`眼睛 icon-btn + **新建工作流**（accent 主按钮，`plus` 图标）。
- **Workflow Row**（卡片，bg2，border，圆角 12，padding 13/14，行距 9；**冲突行边框** = `mix(--bad 36%, --bd)`）。从左到右：
  1. **模式图标砖** 40×40，圆角 9，底 `--accent-soft`，图标 = 该模式 icon（region=`scan`/full=`monitor`/window=`app-window`/fixed=`crop`），accent 色，19px。
  2. **主体**：第一行 名称(13.5/600，超出省略) +（若系统预设）`系统预设` mono 小标签。第二行 spec：`ModeBadge`（mono 标签：引擎名 [vello 时 accent] · 模式名）+ **格式徽章**（`fmt`：PNG=accent 软底，JPG=青 `#22d3ee` 软底，mono 10/600）+ **文件夹快捷**（可点 mono 标签，`folder` 图标，点击打开目标文件夹）。
  3. **两个独立状态指示**（固定列宽 118，竖排）：① 引擎 = `● 引擎就绪`(ok 点) 或 `● 引擎未就绪`(warn 点 + warn 文字)；② 热键 = 冲突时 `⚠ 热键冲突`(bad，600)，否则 `⌨ 热键正常`(muted)。
  4. **热键 kbd**：mono 11/600，bg0，border2，下边框 2px，圆角 7；**冲突时** 文字/边框转 `--bad`。
  5. **操作**：`zap`(立即触发，accent)、`pencil`(编辑→跳 设置/工作流 表单)、`trash-2`(删除，hover 红) —— **系统预设行用 `lock` 禁用键替代删除**。icon-btn 30×30。
- **空态**（`workflows` 为空 / 预览空态）：居中，64px 圆角图标砖（accent-soft）+ `还没有工作流` + 说明 + accent 主按钮 `新建第一条工作流`。

### 2. Activity / 活动中心
- **Header**：`活动中心`(17/800，nowrap) + **LIVE** pill（mono 10.5，bad 描边，内含 `live-dot`：8px bad 圆点 `pulse` 动画 + 4px bad-soft 光环）。右侧说明文字(muted)。
- **Body**：两列 grid `1.5fr / 1fr`，gap 16，顶对齐。
  - **左 · 实时活动流**：区块标题 `实时活动流`(mono sect-title)。每条卡片：34px 类型图标砖（按类型着色：screenshot=accent、ocr=`#22d3ee`、scroll=`#f59e0b`，底为该色 16% 软）+ 名称(12.5/600) + mono 路径(10.5 muted 省略) + 右侧 mono 时间戳(faint) + `external-link` 打开键。
  - **右 · 存储池**：按 `folder` 分组的工作流。可点卡片：36px 文件夹图标砖(bg0 + border) + mono 路径(11/600) + 该池内工作流名(10.5 muted) + 右侧文件数 `fmt` 徽章。点击打开文件夹。

### 3. Settings / 设置
左**子 Tab 导航**（宽 158，右边框，bg1）：`通用 sliders-horizontal / 工作流 workflow / 高级 cpu / 外观 palette / 捐赠 heart`。项 9/10 padding，激活 = accent-soft 底 + accent 字。右侧内容区 padding 22/26，可滚动。通用行式布局用 `Row`（左 标题 13/600 + hint 11.5 muted，右 控件，行间下边框）。

- **通用**：默认保存路径（mono input + `folder-open` 浏览键）/ 标注字体（select）/ 界面语言（Segmented 中文·English）。
- **工作流**：列表（增删改）+ 顶部 `新建`。点 新建/编辑 进入 **WorkflowForm**（卡片 padding 22，max 620）：名称 input / 全局热键（**HotkeyRecorder**：点击进入录制，监听 keydown 拼 `Ctrl+Shift+X`，录制态虚线 accent 边）/ 输出格式（Segmented PNG·JPG）/ 采集模式（4 格选择卡，选中 accent-soft + accent 边/字）/ 渲染引擎（Segmented GDI·Vello；选 Vello 时右侧出现 **风格 select**：Default/Neon/PaperCut/Sketch/Glass）/ 保存位置（mono input + 浏览）/ 两个开关卡 `保存到文件`(hard-drive)、`复制到剪贴板`(clipboard)。底部 取消 / **保存工作流**(accent)。
- **高级**：分三组（mono sect-title）。**导出**：JPG 质量 slider(40–100，accent thumb，旁显数值) / 默认导出格式 Segmented / 并发度 **Stepper**(1–8) / 默认快照尺寸 mono input。**Vello 渲染引擎**：启用开关（Toggle）→ 开时显示 **风格 chips**（5 个，选中 = accent-soft + accent-line 边）+ 高级效果 Toggle。**日志维护**：查看 / 清空 按钮。
- **外观**：主题 Segmented（亮 sun / 暗 moon / 跟随系统 monitor，**实时生效**）+ 强调色 6 色板（40px 圆角块，选中 = 双层 ring）。
- **捐赠**：渐变卡（accent-soft 顶光）+ 60px accent 心形图标 + `请我喝杯咖啡` + 三档金额按钮(中间 accent) + GitHub Star / 收款码 小按钮。

### B. 三个悬浮辅助窗（无边框 / 半透明玻璃）
统一外壳 `.float-win`：`--glass` 底 + `backdrop-filter: blur(22px) saturate(1.3)`，border2，圆角 16，`--shadow-float`，右下 `grip` 缩放手柄。标题栏 `.fw-bar` 高 36：左 6 点 grip（可拖动）+ 图标(accent)+ 标题(12/700 nowrap)+ mono 计数 pill + 右侧操作 + 关闭。

1. **PIN 合集窗**（约 580×470）：标题 `PIN 合集` + 计数 `N / 24`(最大数量) + 全部保存键。Body 2 列卡片网格(gap 12)。**PinCard**：bg1，圆角 12，`cursor: grab`；自带 30px 小标题栏（`grip-vertical` + 标题 + hover 显现的 保存/复制/删除 微钮）+ 截图缩略图（占位）+ hover 右下 `move 拖出粘贴` 玻璃提示。底栏：`可视化的临时剪贴板合集` + **翻页点**（多卡时）。每卡可独立 保存/复制/删除、可拖出粘贴。空态：`pin-off` + 提示。
2. **滚动长截图预览窗**（约 430×540）：标题 + mono 尺寸 `1080 × 5240` + 缩放 ±。Body 居中长图（多段拼接，段间 `接缝` 标记 + accent 虚线），按 zoom% 缩放。底栏 mono 缩放值 + 保存 / **复制**(accent)。
3. **OCR 结果窗**（约 460×420）：标题 + mono `412 字 · 98%` + 复制全部。次栏：源图缩略 + `识别自 xxx.png` + `置信度 98%` chip(on)。Body 可选中文本(13/1.85)。底栏 导出 .txt / **复制全部**(accent)。

### C. 原生截图工具栏（视觉规格，Rust 原生层还原）
见 `prototype/`「工具栏规格」视图——左侧实样 + 右侧规格说明（编号 1–5 对应）。
- **主工具栏**：高 44 / 按钮 32 / 圆角 13 / 工具图标 16；**阴影替代边框**。顺序：矩形`square` 椭圆`circle` 直线`minus` 箭头`move-up-right` 画笔`pen-line` 文字`type` 序号`hash` 马赛克`grid-3x3` ｜ 撤销`undo-2` PIN`pin` ｜ 保存`download` 复制`copy` 关闭`x`(bad)。选中工具 = accent 底 + on-accent 图标。**位置逻辑**：浮于选区下方 8px；选区贴底时翻转到选区上方。
- **二级属性条**：选中绘图工具时在工具栏下方 6px 弹出。含 大小（3 档圆点，选中 accent 描边）/ 颜色（swatch 行，选中双 ring）/ **透明度（仅 Vello）** mono 滑条 / **填充（矩形·椭圆）** Toggle。
- **选区 + 手柄**：1.5px accent 描边，8 个 9px 白描边手柄，外部 42% 黑遮罩，顶部 mono 尺寸读数。
- **选中对象手柄**：8 向方向手柄（10px，bg1 + 2px accent，圆角 2）+ 顶部连接线 + 圆形旋转手柄（含 `rotate-cw`）。
- **放大镜**：96px 圆形，像素网格 + accent 十字准星 + 中心取色框；下方 mono 读数卡（色块 + HEX / RGB / XY）。用于取色与精确定位。

---

## Interactions & Behavior
- **导航**：Rail 切三页；Settings 内左子 Tab 切换；Dashboard `编辑`/`新建` → 跳转 Settings·工作流 并带入 `editTarget`（`"new"` 或 workflow id）。
- **置顶**：Rail 底部开关写 `alwaysOnTop`，标题栏显示 `· 置顶`，并 toast。
- **立即触发**：`triggerWorkflow` → 模拟采集 → `pushActivity`（按 toFile/toClip 决定 path）→ toast。
- **删除**：用户工作流可删；`preset` 拒删（显示 lock）。
- **热键录制**：录制态监听全局 keydown，组合 `Ctrl/Shift/Alt + Key`，松开非修饰键即写入并退出。
- **主题/强调色**：写 store → effect 即时应用到 `<html>`（见 `stores.ts`）。无刷新。
- **Toast**：底部居中，bg3 + border2 + shadow-float，入场 `translateY(8px)→0`，2.2s 自动消失。
- **动效（克制）**：列表/卡片入场仅 `translateY(7px)→0`（**不要用 opacity 0→1 作为入场门，避免后台节流时停在不可见首帧**）；LIVE 点 `pulse`；控件 hover/active 13–16ms transition。reduced-motion 下应直接显示终态。
- **状态色语义（务必区分两个独立信号）**：引擎就绪/未就绪 = ok/warn；热键正常/冲突 = 默认/bad。两者互不耦合，可同时各自亮。

## State Management
见 **`stores.ts`** —— 完整 TypeScript 接口与 actions 即为目标 Zustand store 形状：`workflows / activity / pins / alwaysOnTop` + `Settings`（theme/accent/lang/savePath/annFont/jpgQuality/defaultFmt/concurrency/defaultSize/velloOn/velloStyle/advEffects）。Actions：`toggleTop / addWorkflow / updateWorkflow / deleteWorkflow(拒预设) / triggerWorkflow / pushActivity / removePin / set(settings)`。建议按 slice 拆（workflowSlice / activitySlice / pinSlice / settingsSlice），持久化 settings + workflows（Tauri store 或 localStorage）。

## Assets
- **图标**：lucide（生产用 `lucide-react`）。本设计用到的名字见各 Screen 段内反引号。
- **字体**：Manrope + JetBrains Mono（Google Fonts；生产建议自托管 woff2）。
- **图片**：无品牌素材；所有缩略图/长图/源图为 placeholder，需替换为真实截图位图。
- **Logo**：占位渐变方块（accent → 青），可替换为正式 logo。

## Files
- `prototype/index.html` — 入口（CSS 变量 token 层 + 脚本装配）。
- `prototype/store.jsx` — mock 数据 + 迷你 store + lucide `Icon` 封装。
- `prototype/ui.jsx` — 通用原子（Toggle / Segmented / Stepper / Field / Row / ModeBadge / Toast / EmptyState / Ph）。
- `prototype/main.jsx` — 主窗口外壳 + Rail + Dashboard + Activity。
- `prototype/settings.jsx` — 设置五个子 Tab + 工作流表单 + 热键录制。
- `prototype/floating.jsx` — PIN / 滚动 / OCR 三个悬浮窗。
- `prototype/toolbar.jsx` — 原生工具栏视觉规格（工具栏 + 属性条 + 手柄 + 放大镜）。
- `prototype/app.jsx` — 演示外壳（视图切换 + 主题/强调色实时应用）+ 挂载。
- `tokens.css` — Tailwind v4 token（可直接 import）。
- `stores.ts` — Zustand store 契约（TS 接口 + actions + applyTheme 片段）。

## 从当前实现迁移的注意点
- **新增圆角 / 阴影 / 玻璃 token**：旧版以发丝线为主、几乎无阴影。新增 `--radius-panel:12px`、`--shadow-float`、悬浮窗 `backdrop-filter`。Card / Row / Toolbar 的视觉需重写（间距加大、边框换柔光 + 阴影），**逻辑不变**。
- **变量映射**：旧 `--bg-main→--bg0`、`--bg-card→--bg1`、`--bg-subtle→--bg2`、`--text-main→--tx`、`--text-muted→--mut`、`--border→--bd`。强调色由 `#3b82f6` 调为 periwinkle `#7a6ff2`，且**只保留单一 `--accent`**，press/on-accent 改为派生。
- **去工程冷硬**：把「整屏大写 + 宽字距」收敛为仅 mono 元数据；正文改 Manrope 常规字重。
- **状态升级**：原单一色点 → **双独立指示**（引擎 + 热键），Dashboard 顶部加「一眼扫描」汇总 pill。
- **图标**：lucide 名一致，注意 kebab→PascalCase 组件名。
