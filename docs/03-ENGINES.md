# 03 · 双引擎与采集渲染管道

> 本文是对归档版 `ENGINE_REFERENCE.md`（Frozen v0.2.3）的核查更新版：技术结论仍然成立，文件路径已按 `main` 当前模块结构修正。需要逐字历史细节时查 [`archive/2026-06-04-pre-redesign/ENGINE_REFERENCE.md`](archive/2026-06-04-pre-redesign/ENGINE_REFERENCE.md)。

## 1. 为什么是两个独立引擎

NexSpot 支持两套互相独立的"采集 + 渲染"引擎，**各自拥有专属 HWND，绝不共用**：

| 维度 | GDI 引擎 | Vello (WGC) 引擎 |
|------|----------|------------------|
| 专属 HWND | `gdi_hwnd`（类名 `HyperLensOverlayGDI`） | `vello_hwnd`（类名 `HyperLensOverlayVello`） |
| 采集 API | Win32 `BitBlt`（整个虚拟桌面） | Windows Graphics Capture（按显示器） |
| 渲染 API | GDI `UpdateLayeredWindow` | wgpu/DirectX via Vello |
| 窗口样式 | `WS_EX_LAYERED` | DWM 合成（非分层） |
| 背景存储 | `state.gdi.hbitmap_dim/bright`（HBITMAP） | `state.vello.background`（RGBA8 ImageData） |
| 坐标系 | 全局虚拟桌面，偏移 `state.x/y` | 目标显示器本地，经 `global_transform` |
| 多屏 | 整个虚拟桌面单张合成位图 | 每屏独立 WGC 流或一次性捕获 |
| 风格化 | 不支持 | 支持 `AestheticStyle`（Neon/Glass…） |

### 双 HWND 隔离的根因（关键设计约束）

DWM 把 **DXGI 交换链** 与 **分层窗口（Layered Window）** 当作两个独立合成层，DXGI 优先级更高。若两引擎共用同一 HWND，DXGI 的残留帧会污染 GDI 采集，且任何 GDI 级 flush 或 DWM API 都无法清除。**结论：物理隔离两个 HWND**（v0.2.2 起）。曾失败的方案：GDI flush、禁用 DWM、重建 HWND、wgpu 透明 present。

实现位置：`native_overlay/manager/`（`creation.rs` 建双窗、`engine.rs` 引擎切换、`lifecycle.rs` 显示/隐藏/重置）。关键方法：`active_hwnd(engine)`、`show_overlay_at()`（显示当前引擎窗口、隐藏另一个）、`close_and_reset()`（隐藏两窗）、`set_user_data()`（两窗都注册 WindowEventHandler）。

## 2. 引擎选择流程

配置来源：
- `config.vello_enabled`：全局开关（持久化）。
- `workflow.action` 中每条工作流的 `engine` 字符串（`"gdi"` / `"vello"`）。
- `config.selection_engine` / `config.snapshot_engine`：模式级默认引擎（**选区与快照引擎独立配置**）。

分发链（`workflow/mod.rs` → `capture/` → `manager/`）：
1. `workflow/mod.rs`：由 `target_engine` 决定 `CaptureEngine::{Gdi|Wgc}`，写入 `state.capture_engine`，调用 `perform_capture`。
2. `capture/mod.rs`：按 `capture_engine` 分发到 `gdi.rs`（BitBlt 全桌面）或 `wgc.rs`（流帧/一次性）。
3. `manager/`：`show_overlay_at` 从 active_workflow 重新推导引擎（**神圣，不被覆盖**），`SetWindowPos` → `render_frame`。

优先级规则：① 有 active_workflow → 用其 `engine`；② 否则用 `state.capture_engine`；③ 仅当无 active_workflow 且 `vello_pref==true` 时自动升级到 Vello。

## 3. GDI 引擎

- **采集**（`capture/gdi.rs`）：对整个虚拟桌面 `union_rect` 做 `BitBlt`，输出两张 HBITMAP——`hbitmap_bright`（原图，用于选区高亮抠图）与 `hbitmap_dim`（经 `AlphaBlend` 叠 ~45% 黑得到的变暗图）。
  - ⚠ 变暗遮罩必须用**全尺寸**黑位图做 `AlphaBlend`；把 1×1 黑像素拉伸到 4K 在部分 WDDM 驱动上会失败。
- **渲染**（`render/mod.rs` GDI 路径）：`WS_EX_LAYERED=true`；`hbitmap_dim` → 后台缓冲；选区区域用 `hbitmap_bright` 抠出；GDI 绘图模块画标注；`UpdateLayeredWindow` 输出。
- **坐标**：全局虚拟桌面空间；设备坐标 = 全局坐标 − `state.x/y`。
  - ⚠ 内存 DC 上**禁用** `SetWindowOrgEx`（会静默污染 `UpdateLayeredWindow` 的 `pptSrc`，且在多屏下致 bug）。

## 4. Vello (WGC) 引擎

### 4.1 VelloContext 生命周期（`render/vello_engine/`）

- 启动（`vello_enabled=true`）：异步 `VelloContext::new(hwnd)` → wgpu Instance → 高性能 adapter → device → Renderer，存入 `OverlayManager.vello_ctx`。
- ESC `close_and_reset()`：只清 surfaces/configs/caps/proxy_textures（HashMap::clear）；**VelloContext（device/adapter/renderer）保持存活**。
  - ⚠ 切勿把 `vello_ctx` 置 None——GPU 重新初始化耗时 ~5–10s。
- 退出（Drop）：`stop_pre_heat()` → 释放 WGC 流；VelloContext 随 OverlayManager 自然析构。

### 4.2 WGC 采集管道（`win32/wgc/capture.rs`）

- `WgcStreamManager` 后台线程预热：对每个 `Monitor::enumerate()` 起一个 `WgcStreamHandler`，`on_frame_arrived` 把帧写入 `StreamState.image`，`on_closed` 置 `is_alive=false`。
- 采集优先级：① 命中流缓存（`is_alive && image.is_some()`，~0ms）；② 否则一次性 `capture_monitor_to_vello`（~100–200ms）。
- 显示器键用 **usize 枚举索引**（不用名称——相同型号显示器名称相同会哈希碰撞）。
- ⚠ WGC 流线程会在 DRM/分辨率变更/休眠唤醒时静默死亡，由 `is_alive` 标记，死流自动回退到一次性采集。

### 4.3 Vello 渲染（`render/vello_engine/renderer/mod.rs`）

`render_state_to_scene`：① 画背景（按窗口/背景尺寸非均匀缩放，补偿 DPI/采集尺寸差）；② 计算 `global_transform = translate(-state.x, -state.y)`；③ 在**全局坐标**下构建 `inner_scene`（选区四块变暗遮罩+抠图+手柄、绘图对象含发光/阴影/透明图层、放大镜、工具栏、tooltip(Parley)、工具预览）；④ `scene.append(&inner_scene, Some(global_transform))` 把全局→本地变换施加到每个元素。
- ⚠ Vello 的 `push_layer` 的 transform **只作用于裁剪形状**，不变换子几何。坐标平移**必须**用 `scene.append(inner, Some(transform))`（O(N)）。
- 裁剪矩形固定 `[-1e6, 1e6]`，故意超大以免裁掉多屏内容。

### 4.4 跨屏选区约束

| 引擎 | 约束 |
|------|------|
| WGC(Vello) | **始终**限制在采集显示器内（`restrict_to_monitor = Some(rect)`） |
| GDI | 仅当混合 DPI 时限制，否则允许跨屏 |

约束在 `interaction/mouse_move/` 的 Selecting/Moving/Resizing 中执行。

## 5. 状态清理（`close_and_reset`，ESC 触发）

置位/清空：`is_visible→false`、`selection→None`、`interaction_mode→None`、`objects` 清空、`vello.background→None`、`gdi.hbitmap_dim/bright→None`、`active_workflow→None`。
保留：`vello_ctx`（只清 surfaces）、`wgc_stream`（继续运行）。
两个 HWND：KillTimer + 隐藏。

> 历史教训：早期 `close_and_reset` 不清 `is_snapshot_mode`/`capture_engine`/`active_workflow` 等导致跨模式切换状态泄漏（旧 P1 缺陷，现已修复并模块化到 `state/overlay_state/logic.rs`）。

## 6. 已知约束清单（务必遵守）

1. `push_layer` 不变换几何 → 用 `scene.append`。
2. 两套 API 的显示器索引顺序可能不同（`Monitor::enumerate` vs `EnumDisplayMonitors`）。
3. WGC 流线程会静默死亡 → `is_alive` + 一次性回退。
4. `AlphaBlend` 不可拉伸 1×1 → 用全尺寸源位图。
5. 内存 DC 禁用 `SetWindowOrgEx`。
6. VelloContext GPU 初始化 ~5–10s，绝不重建，只清 surface。
7. DWM DXGI/Layered 隔离 → 双 HWND（已解决）。
8. Parley 用 `new_for_non_complex_scripts()`，CJK 分词需上游修复，改 Cargo feature 无效。
9. `global_hotkey::AlreadyRegistered` = 被其他程序占用，必须报错而非静默成功。
10. 高频日志（WM_MOUSEMOVE / 每帧 / 选区）一律用 `log::trace!`，否则日志爆炸阻塞渲染循环。
