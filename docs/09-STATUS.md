# 09 · 项目状态固化（2026-06-12 · v0.3.0）

> 本文是 2026-06-04 全面核查（[08-AUDIT](08-AUDIT.md)）之后一轮密集演进的**收口快照**。
> 涵盖：Studio UI 重设计落地、OCR 子系统重建、一批运行时硬 bug 修复。
> 代码为唯一真相，本文记录"为什么是现在这样"。

## 版本与发布

- 版本：**0.3.0**（package.json / tauri.conf.json / Cargo.toml / locale 四处同步）
- 产物：`npm run tauri build` → NSIS setup + MSI + 独立 exe（`src-tauri/target/release/`）
- ⚠ 长期教训：**测试只认当前构建**。多次"功能没生效"误报均因运行了旧 exe / 旧安装版。

## 已完成的大块

### 1. Studio UI 重设计（Phase 3，全部落地）
设计基准：`docs/design/2026-06-04-studio-redesign/`（网页版 Claude 产出，Linear/Raycast 方向）。

- **令牌系统**：`src/index.css` 单一真相（bg0-3 / ink / muted / accent periwinkle `#7a6ff2` / ok·warn·bad + soft），
  Tailwind v4 `@theme inline`，亮暗主题 `<html data-theme>`，accent 由 JS 派生 press/on-accent（App.tsx）。
  旧工业风工具类与兼容别名**已全部移除**。
- **外壳**：无边框窗（decorations:false，Win11 DWM 自动圆角投影，**刻意不开 transparent** 避黑角）、
  自定义 TitleBar（38px，三键，关闭 hover 红）、48px Rail（accent-soft 激活 + 左缘竖条）。
- **页面**：Dashboard（工作流卡片 + 引擎/热键**双独立状态指示** + 汇总 pill）、
  Activity（双列，缩略图/列表切换，asset 协议加载真实缩略图）、
  Settings（158px 子 Tab + 共享原子 `Settings/atoms.tsx`：Row/Toggle/Segmented/Stepper/TextField）、
  三个悬浮窗玻璃化（`.float-win`）。
- **原生层**：GDI/Vello 工具栏、选区、二级属性条全部对齐 Studio（constants.rs 集中调色；
  选中工具 = accent 整键填充 + 白图标；图标 lucide 几何重绘 + remixicon 字形经 fontTools cmap 校验精确换码点）。
- 文案全面去"工程冷硬腔"；捐赠页 → "赞助 Token"。

### 2. OCR 子系统（重建，双引擎）
原状态：工具栏按钮 emit 无监听者的事件 = **完全死链**。现状：

- **触发**：工具栏按钮 → 后端直接调用（`native_overlay/commands.rs`）。失败弹系统通知。
- **流程**（`service/ocr.rs::execute_ocr`）：渲染选区 → **立即开结果窗**（"识别中"动画，即时反馈）
  → **关闭截图覆盖层**（解决 Vello DXGI topmost 压窗的 z-order 终局方案）→ 识别 → 填充。
- **引擎双轨**：
  - `winrt`：零依赖。语言三级选择（设置指定 → auto 中文优先 zh-Hans → 用户档案）。
  - `paddle`：**PaddleOCR-json 组件**（`service/paddle_ocr.rs`）。常驻子进程 + JSON stdio（image_base64，BMP 编码）；
    组件目录 `{app_data}/ocr/PaddleOCR-json/`；**应用内一键下载/安装/更新**（GitHub release + 进度事件 `paddle://progress`）；
    语言 = `models/config_*.txt` 枚举，换语言重启进程；**预热**（启动/切引擎/切语言时后台加载模型，点击即识别）。
- **坐标链**：识别坐标（选区物理像素）→ ÷upscale（仅 WinRT 小图 2x）→ ÷DPI scale → 结果窗逻辑像素。
- **文字层**：始终可见（深底白字块），字号按**中位高度归一**（检测框高度天然抖动 ±30%，
  逐词取高会大小字乱跳；>1.5× 中位的真标题保留大字）。
- 滚动长截图同轮接通：工具栏按钮切换语义（开始/结束），结束弹预览窗。

### 3. 修复的硬 bug（根因记录，防回归）
| Bug | 根因 | 防回归要点 |
|---|---|---|
| 双击截图崩溃（多屏） | Wgc 保存路径用世界坐标索引画布局部缓冲，负坐标 `as usize` 回绕 → 乘法溢出 abort | 选区坐标必须先减画布原点（render.rs） |
| OCR/滚动点击 panic | `render_snapshot` 内 `block_on`，tokio worker 上嵌套 runtime | 已改 spawn+通道；**禁止在可能跑在 worker 的代码里 block_on** |
| 一次点击拖垮全部截图 | `start_scrolling_session` 持 manager 锁调 `render_snapshot`，后者再锁同一 manager → 自死锁 | **绝不持 overlay_manager 锁调用 render_snapshot**（其 Wgc 分支内部要锁它取 vello_ctx） |
| Vello 复制粘贴失败 | Wgc 产物是 DIB section，直接当 CF_BITMAP 多数应用解码不了 | clipboard.rs 统一 BitBlt 转 DDB 再上剪贴板 |
| OCR 识别全黑/全盲 | ①GetDIBits alpha 恒 0，直构 SoftwareBitmap 按预乘解释≈全黑 ②用户档案英文 → 英文引擎中文全盲 | BGRA 转换强制 alpha=255；语言显式选择 |
| OCR/滚动预览窗关不掉 | `ocr-result`/`scrolling-preview` 不在 capability 窗口白名单 | 新窗口 label 必须进 `capabilities/default.json` |
| Vello 色块选中环不可见 | kurbo `inset` 负值=缩小，环画进色块内被盖 | — |
| 右下手柄带动上边界 | resize BottomRight 分支复制粘贴错改 `top` | — |
| 强调色迁移不生效 | 迁移写在 `ConfigManager::load()`，但启动走 `ConfigState::new()` | 启动路径才是迁移挂载点 |

### 4. 基础设施
- **全局 panic 钩子**：崩溃位置写 `%TEMP%\nexspot_panic.log`（非展开 panic 也能拿到 文件:行），多次直接定位根因，长期保留。
- 截图保存成功记入活动流（`log_activity("screenshot")`，此前只有 OCR/滚动记录）。
- IPC 契约经全量交叉核对（前端 invoke ⊆ lib.rs 注册表）；locale zh/en key 集合一致。

## 已知未尽事项（Phase 4 候选，按价值排序）
1. **原生工具栏 accent 跟随用户自定义色**（现为常量 periwinkle；需把 config 接入 overlay state）。
2. PIN 窗：移除死的 `ai-stream://*` 监听；深度交互（拖出粘贴、翻页点）。
3. 滚动/OCR 结果窗按设计稿补功能件（缩放±、接缝标记、置信度 chip、导出 .txt）。
4. 设计稿中 Vello 5 风格（Neon/PaperCut/Sketch/Glass）的视觉成熟度。
5. Clippy 约 80 条风格 lint（非错误）。
6. 双引擎 × 4 模式 8 格验证矩阵未做系统性回归（修复后只做了重点路径）。

## 验证现状（用户实机确认过）
- ✅ Studio UI 全套观感、强调色、文案
- ✅ GDI/Vello 工具栏与选区样式、图标
- ✅ Vello 复制→粘贴、双击保存（多屏）
- ✅ GDI OCR（PaddleOCR 引擎识别质量良好）、组件一键下载
- ⏳ 待复测：Vello OCR（覆盖层关闭方案）、即时反馈、字号归一、滚动长截图全流程
