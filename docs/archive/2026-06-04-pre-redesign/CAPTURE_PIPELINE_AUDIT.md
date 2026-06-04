# 采集管道审计报告

> 审计日期：2026-02-21  
> 审计范围：GDI / Vello(WGC) 双引擎全流程 — 采集 → 渲染 → 保存 → 状态管理

---

## 一、管道架构总览

```
触发 (hotkey/tray/command)
  ↓
execute_capture_workflow    ← 设置 OverlayState
  ↓
perform_capture             ← GDI BitBlt / WGC 帧捕获
  ↓
show_overlay_at             ← 显示覆盖层窗口
  ↓
render_frame                ← GDI UpdateLayeredWindow / Vello DX11 SwapChain
  ↓
save_selection / copy_to_clipboard  ← 裁剪 + 编码 + 输出
  ↓
close_and_reset             ← 重置状态
```

### 关键文件

| 模块 | 文件 |
|:---|:---|
| 工作流执行 | `service/workflow/mod.rs` |
| 屏幕采集 | `service/native_overlay/capture.rs` |
| WGC 流管理 | `service/win32/wgc/capture.rs` |
| 覆盖层管理 | `service/native_overlay/manager.rs` |
| 渲染主函数 | `service/native_overlay/render/mod.rs` |
| Vello 渲染器 | `service/native_overlay/render/vello_engine/` |
| 保存/复制 | `service/native_overlay/save.rs` |
| 位图编码 | `service/win32/bitmap.rs` |
| 状态定义 | `service/native_overlay/state/overlay_state.rs` |
| 配置类型 | `service/config/types.rs` |

---

## 二、发现的问题

### P1 — 严重（功能性缺陷）

#### 2.1 Vello 模式副屏黑屏

**位置**：`service/win32/wgc/capture.rs` 第 184 行  
**现象**：Vello 模式在副屏触发截图时显示黑屏  
**根因**：  

- `WgcStreamManager::start()` 只为 `monitors.first()`（主屏）建立预热流
- 副屏走 `capture_monitor_to_vello()` 一次性捕获，失败后不回退
- 注释写明 `"Strict separation: Do NOT fallback to GDI"`
- 失败后 `vello_background = None` → 黑屏

```rust
// wgc/capture.rs:184
if let Some(monitor) = monitors.first() {  // ← 只取主屏
```

**修复**：为所有屏幕各建独立预热流。

---

#### 2.2 Vello 模式复制到剪贴板不工作

**位置**：`save.rs` 第 309 行  
**现象**：Vello 模式下按复制无反应  
**根因**：`copy_to_clipboard` 只读取 `hbitmap_bright`，Vello 模式下此字段为 `None`

```rust
// save.rs:309
if let Some(hbm_bright) = &state.hbitmap_bright {  // ← Vello模式下为 None
```

而 `save_selection`（第 77-99 行）有 `vello_background` 回退逻辑，`copy_to_clipboard` 没有。  
**修复**：`copy_to_clipboard` 增加 `vello_background` 分支。

---

#### 2.3 `close_and_reset` 状态泄漏

**位置**：`manager.rs` 第 134-157 行  
**现象**：Vello 快照完成后切换到 GDI 划区，出现选区锁定、比例异常  
**根因**：`close_and_reset` 不清除以下关键字段：

| 字段 | 残留影响 |
|:---|:---|
| `is_snapshot_mode` | 下次采集被当作快照处理 |
| `capture_engine` | 引擎判断被上次残留覆盖 |
| `active_workflow` | `show_overlay_at` 读取旧工作流的引擎设置 |
| `vello_background` | `save_selection` 优先使用旧数据 |
| `hbitmap_dim/bright` | GDI 渲染可能使用上次的位图 |
| `is_capturing` | 并发保护标志未清除 |

**修复**：在 `close_and_reset` 中显式重置所有采集相关字段。

---

### P2 — 中等（设计缺陷）

#### 2.4 GDI/Vello 引擎交叉污染

**位置**：`capture.rs` 第 64-90 行，`save.rs` 第 76-99 行  
**现象**：GDI 模式性能损耗，颜色逻辑复杂  
**根因**：

- GDI 采集路径**无条件**执行 BGRA→RGBA 转换并设置 `vello_background`
- `save_selection` 优先使用 `vello_background`（即使当前是 GDI 模式）
- 导致 GDI 模式产生不必要的全屏像素遍历开销

```
GDI 采集 → BitBlt → hbitmap_bright (BGRA, 正确)
         → get_bitmap_bits → BGRA→RGBA → vello_background (多余)
         
保存时 → 优先取 vello_background → 再做 RGBA→HBITMAP → 再 GetDIBits BGRA→RGBA
结果：同一批像素被转换了3次
```

**正确设计**：

- GDI 模式：只设 `hbitmap_bright/dim`，不碰 `vello_background`
- Vello 模式：只设 `vello_background`，不碰 `hbitmap_bright/dim`
- 保存时根据 `capture_engine` 走对应分支

---

#### 2.5 JPG 质量参数未接入

**位置**：`config/types.rs` 第 122 行，`win32/bitmap.rs` 第 80 行  
**现象**：设置中的 JPG 质量滑块无实际效果  
**根因**：

- `AppConfig.jpg_quality` 已定义（默认 90）
- `ConfigState::set_jpg_quality()` 已实现
- 但 `save_bitmap_to_file()` 不接受质量参数
- `image::save_buffer()` 根据扩展名推断格式，不传质量参数
- JPG 格式需要使用 `image::codecs::jpeg::JpegEncoder::new_with_quality()`

```rust
// bitmap.rs:80 — 硬编码，不支持质量参数
image::save_buffer(path, &pixels, width, height, image::ColorType::Rgba8)?;
```

**修复**：`save_bitmap_to_file` 增加 `format` + `quality` 参数，JPG 使用专用编码器。

---

#### 2.6 `save_bitmap_to_file` 冗余颜色转换

**位置**：`win32/bitmap.rs` 第 73-78 行  
**现象**：保存耗时增加  
**根因**：`GetDIBits` 返回 BGRA，`save_bitmap_to_file` 内部做一次 BGRA→RGBA。如果调用者已经传入了 vello_background 的 RGBA 数据（经 `create_bitmap_from_pixels` 转成 HBITMAP），那么 `GetDIBits` 再取出的可能是 BGRA，又要再转一次。总共 3 次转换。

---

### P3 — 低（占位/未完成功能）

#### 2.7 全屏截图 (`Fullscreen`) — 占位符

**位置**：`config/types.rs` 第 69-71 行，`workflow/mod.rs` 第 52 行  
**现象**：前端下拉可选，但行为与区域选取完全相同  
**根因**：后端 `execute_capture_workflow` 将 `Fullscreen` 映射到 `CaptureMode::Standard`，无全屏自动选区逻辑  
**预期行为**：应自动将 `selection` 设为整个屏幕区域，跳过用户拖拽步骤

---

#### 2.8 窗口捕获 (`Window`) — 占位符

**位置**：`config/types.rs` 第 72-74 行，`workflow/mod.rs` 第 53 行  
**现象**：同上，行为与区域选取完全相同  
**根因**：无窗口检测和自动对齐选区逻辑  
**预期行为**：应检测鼠标下的窗口，自动将 `selection` 设为窗口边界

---

#### 2.9 `bitmap_to_png_bytes` 硬编码 PNG

**位置**：`win32/bitmap.rs` 第 92-169 行  
**现象**：仅支持 PNG 编码  
**影响**：如需支持 JPG 等格式的剪贴板或内存操作，需要重构此函数

---

## 三、优先级建议

| 优先级 | 问题 | 影响 |
|:---:|:---|:---|
| **P1** | 2.1 副屏黑屏 | 多屏用户无法使用 Vello |
| **P1** | 2.2 Vello 复制不工作 | Vello 用户无法复制截图 |
| **P1** | 2.3 状态泄漏 | 模式切换后功能异常 |
| **P2** | 2.4 引擎交叉污染 | 性能损耗 + 维护复杂度 |
| **P2** | 2.5 JPG 质量未接入 | 设置项无效果 |
| **P2** | 2.6 冗余颜色转换 | 性能损耗 |
| **P3** | 2.7 全屏截图占位 | 功能未实现 |
| **P3** | 2.8 窗口捕获占位 | 功能未实现 |
| **P3** | 2.9 PNG 硬编码 | 扩展性 |

---

## 四、已确认正常工作的功能

- ✅ 保存路径解析（`save_path` + workflow 覆盖）
- ✅ 文件命名模板（`chrono::format`）
- ✅ 快照尺寸记忆（`save.rs` 第 19-51 行 + 第 276-307 行，两个保存路径都有）
- ✅ GDI 渲染管道（`hbitmap_dim/bright` → `UpdateLayeredWindow`）
- ✅ Vello 渲染管道（`vello_background` → Scene → DX11 SwapChain）
- ✅ 工作流系统（CaptureWorkflow → CaptureAction → CaptureOutput）
- ✅ 通知系统（保存/复制成功后的系统通知）
