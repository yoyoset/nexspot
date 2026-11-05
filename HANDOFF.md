# HANDOFF：写给接手 NexSpot 的下一个 AI

你好。这封交接信是 2026-11-05 写的（Claude Opus 5.5），把 2026-06 以来几轮会话的记忆、提交记录和 `docs/` 里的状态文档汇到一起。
信里讲这几件事：项目是什么、现在走到哪了、怎么构建、为什么这样设计、踩过哪些坑、还有什么没做、哪些事不要做。
**`docs/09-STATUS.md` 是状态的单一真相，`docs/02-ARCHITECTURE.md` 里的 IPC 命令表是契约的真相**，动手前先读这两份。

> 用户的全局工作原则，照做：
> - 先实测再下结论；实测和早先的假设矛盾时，如实写"以为是 A，实测是 B"。
> - 绝不能出现"后台失败但用户以为成功"。
> - 只按路径 `git add`，小步原子提交；**这个项目的提交信息用中文，并写清根因**。
> - 构建和测试全绿才提交。
> - 机器是 Windows，主 shell 是 PowerShell，git bash 在 `D:\PortableGit\bin\bash.exe`。

---

## 1. 项目是什么，现在什么状态

NexSpot 是 Windows 截图工具，技术栈是 Tauri v2 + Rust/Win32 + React 19/TS/Tailwind v4。
功能有：截图（GDI 和 Vello/WGC 两套引擎）、标注、OCR（WinRT 和 PaddleOCR-json 双引擎）、贴图（PIN）、滚动长截图。
最初由另一个 AI 工具 antigravity 写成，2026-06 做过全面核查（`docs/08-AUDIT.md`），之后完成了 Studio UI 重设计和 OCR 子系统重建，当前版本 **v0.3.0**。

**分支状态，必须先搞清楚：**
- `main` = `origin/main` = `cca630f`（06-12，"v0.3.0 状态固化"）。
- 本地分支 **`codex/v0.3.0-rc1-recovery`** 比 main 多 3 个提交（07-10，145 个文件、+9625 行），**只在本地，没推到 GitHub**：
  - `18bb20b`：稳定 OCR 和滚动截图
  - `4e20ceb`：发布和 QA 门禁脚本（`scripts/*.mjs`、`npm run release:*`、`qa:*`）
  - `b7d07c0`：冻结 RC1 发布证据（`docs/10`–`21`）
- 这个分支名说明它是 Codex 做的。要不要合进 main、要不要推送，**先问用户**。
- 这封交接信放在 main 上。如果你在那个分支上干活，用 `git show main:HANDOFF.md` 查看。

**仓库与发布：**
- 仓库 `yoyoset/nexspot` 是 public，**还没有 GitHub Release**。
- 本地有安装包 `src-tauri/target/release/bundle/nsis/NexSpot_0.3.0_x64-setup.exe`（07-10，8.9M），**没有签名**。

**发布评审结论（`docs/21-COMMERCIAL-CLOSURE-PLAN.md`，07-10）：**

| 级别 | 结论 |
|---|---|
| 内部试用 | Go |
| 小范围试点 | Go，但要附"未签名"说明 |
| 公开稳定版 | No-Go |

公开稳定版 No-Go 的原因：4 个 P0、23 行必测项没测（失败数 0，全部是"待测"，不是"测挂了"）。4 个 P0 是：
- QA-001：Vello OCR
- QA-002：滚动长截图
- QA-003：多屏/DPI
- QA-004：分发信任（签名）

**用户 11-02 的决定（记录在 SUBLOG 排期表）：** 滚动长截图和截视频当初需求不大，可以不要，不要这两样就可以发。
核对结果：代码里**没有截视频**，只有滚动长截图。拿掉滚动长截图后 QA-002 就不存在了，剩下的处理方式：
- Vello OCR：用户实机确认过的是 GDI 引擎下的 OCR，所以**把默认引擎定成 GDI**。
- 多屏/DPI：用户实机确认过多屏双击保存；其他 DPI 组合没测，如实写明。
- 签名：下载页写明"Windows 会提示未知发布者"，并附 SHA256。

**要做的改动：**
1. 藏掉滚动长截图的所有入口：GDI 和 Vello 两套工具栏、首页工作流、设置页。
2. 重新打包。
3. 跑一遍冒烟测试。
4. 请用户实机点一遍截图和 OCR。
5. 上架 squareuncle.com 下载页（这一步在 SUBLOG 仓库做）。

排期在 `F:\my_ai\SUBLOG\video\weekly\2026-11-博客与产品排期.md`。用户是否最终同意上架还没表态，以排期表为准。

## 2. 命令

```powershell
npm install
npm run tauri dev          # 开发运行（用户测试时必须确认跑的是这个，见坑 8）
npm run build              # 前端 tsc + vite
cd src-tauri; cargo check  # 每改一步都要跑
npm run tauri build        # → src-tauri/target/release/：NSIS setup + MSI + 独立 exe
# 下面这些只在 codex/v0.3.0-rc1-recovery 分支上有：
npm run release:smoke; npm run release:installer:smoke; npm run release:verify; npm run qa:manual:gate
```

- 版本号有四处要同步：`package.json`、`tauri.conf.json`、`Cargo.toml`、locale。
- 改 locale 时用 Python 的 json 读写（UTF-8），改完核对 zh 和 en 的 key 集合是否一致。
- 发布到 GitHub 用 `& "C:\Program Files\GitHub CLI\gh.exe" release create ...`（gh 要写全路径；账号 yoyoset 已登录）。上传后把文件下载回来，核对 SHA256。

## 3. 关键设计和为什么

1. **OCR 结果窗先弹出、再识别**：先显示"识别中"窗口，再关掉截图覆盖层，最后识别。这样能立即给反馈，也绕开了 Vello 覆盖层一直置顶、压住结果窗的问题。用户说过："点了没反应"不可接受。
2. **PaddleOCR 做成应用内一键下载、安装、更新**：用户不接受手动解压组件。组件目录是 `{app_data}/ocr/PaddleOCR-json/`。它是常驻子进程，用 JSON 走 stdio 通信；启动、切换引擎、切换语言时会在后台预热。
3. **OCR 文字层的字号按中位高度归一**：检测框的高度天然会抖动 ±30%，如果每个词按自己的高度取字号，大小字会乱跳。高度超过中位数 1.5 倍的真标题保留大字。
4. **设计令牌只有一处**：`src/index.css`。原生层的颜色集中在 `constants.rs`。
5. **无边框窗口，刻意不开 transparent**：开了会出现黑角，Win11 的 DWM 会自动加圆角和投影。
6. **全局 panic 钩子**：崩溃位置会写到 `%TEMP%\nexspot_panic.log`（文件:行号），要长期保留。

## 4. 代码雷区（每条都导致过崩溃、死锁或"没反应"；完整根因表见 09-STATUS §3）

1. **绝不能在持有 overlay_manager 锁的时候调用 `render_snapshot`**：它的 Wgc 分支会再锁同一个 manager，同一线程重入 std::Mutex 就是永久死锁，而且会静默拖垮之后所有截图操作，包括 GDI 热键。正确做法是先克隆 Arc，再释放锁。
2. **可能跑在 tokio worker 上的代码里禁止 `block_on`**：嵌套 runtime 会直接 abort。要等结果就用 spawn 加 mpsc 通道。
3. **坐标系**：选区用世界坐标（多屏时可以是负数），画布缓冲用局部坐标。索引前必须先减去捕获原点，否则负数 `as usize` 会回绕，乘法溢出导致崩溃。OCR 的坐标链是：物理像素 ÷ upscale ÷ DPI scale → 逻辑像素。
4. **剪贴板的 CF_BITMAP 只接受 DDB**：DIB section 直接放上去，多数应用解码会失败。要先用 BitBlt 转成 DDB。
5. **GetDIBits 拿到的 alpha 恒为 0**：在构造 SoftwareBitmap 之前，强制把 alpha 设成 255，否则 OCR 看到的是一张全黑图。
6. **新建 webview 窗口，必须把它的 label 加进 `capabilities/default.json`**，否则前端调用 `close()` 之类会被静默拒绝。
7. **诊断顺序**：崩溃了先读 `%TEMP%\nexspot_panic.log`。没有 panic 却"没反应"，先怀疑死锁或者某条静默的错误路径。
8. **测试假象**：用户说"没变化""坏了"时，先确认他跑的是 `npm run tauri dev` 的当前源码，而不是 target/release 里的旧 exe 或者旧安装版。这个坑反复出现过。
9. **Vello 覆盖窗一直置顶，别跟它抢 z-order**（`set_focus` 不可靠）：要弹结果窗时，直接 `close_and_reset()` 关掉覆盖层。
10. **kurbo 的 `Rect::inset` 正值是向外扩**：选中环要画在色块外侧。
11. **强调色迁移写在 `ConfigManager::load()` 里不会生效**，因为启动走的是 `ConfigState::new()`。迁移逻辑要挂在真正的启动路径上。

## 5. 已知但还没处理的问题

- 上面 4 个 P0、23 行必测项（见 `docs/12-MANUAL-QA-MATRIX.md`、`docs/16-COMMERCIAL-QA-ISSUE-LOG.md`）。
- Phase 4 候选（按价值排序）：
  1. 原生工具栏的强调色跟随用户自定义色
  2. PIN 窗的深度交互
  3. OCR 和滚动结果窗按设计稿补功能件
  4. Vello 五种风格的视觉成熟度
- 大约 80 条 clippy 风格 lint。
- "双引擎 × 4 模式"的 8 格验证矩阵没有系统回归过。
- 没有代码签名。

## 6. 不要做的事

- 不要 `git add -A` / `git add .`。
- 没有用户同意，不要把 `codex/v0.3.0-rc1-recovery` 推送或合并到 main。
- 不要留摆设控件：坏掉或无效的控件要么修通，要么删掉（用户的标准是"成熟的商业级软件"）。
- 不要宣称修好了用户能看见的问题，除非用户在当前构建上实机确认过。你没法看到画面时，要明确告诉他"请看哪几点"，并给出 10 秒内能判断的标准。
- 不要凭"设计稿条文"否掉用户的实际观感：设计稿是基准，但用户的观感优先。
