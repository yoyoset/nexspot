# 02 · 架构

## 系统拓扑

```
┌──────────────────────── Webview (React/TS) ────────────────────────┐
│  主窗口: Navigator + (Dashboard | ActivityHub | SettingsPanel)      │
│  独立窗口 (hash 路由): #pin-collection / #scrolling-preview /        │
│                        #ocr-result                                  │
└───────────────┬─────────────────────────────────────────┬──────────┘
                │ invoke()                          emit() │  (事件)
                ▼                                          ▲
┌──────────────────────── Tauri 主进程 (Rust) ───────────────────────┐
│  commands/          对外 IPC 接口（见下表）                          │
│  service/                                                          │
│    config/          AppConfig 持久化 + workflow + 热键注册           │
│    hotkey/          全局热键监听线程                                  │
│    workflow/        热键命中 → 构造采集动作 → 触发覆盖层              │
│    native_overlay/  ★ 核心：采集 / 交互 / 渲染 / 保存 / 状态机        │
│    win32/           所有 unsafe 原生封装（GDI / GDI+ / WGC / 窗口）   │
│    pin/ ocr/ stitch/ activity/ tray/ ai/ ...  扩展服务              │
└───────────────┬─────────────────────────────────────────┬──────────┘
                │ Win32 API                                │
                ▼                                          ▼
        ┌───────────────┐                        ┌──────────────────┐
        │  gdi_hwnd      │                        │  vello_hwnd      │
        │  WS_EX_LAYERED │                        │  DWM 合成 / DXGI │
        │  GDI 采集+渲染 │                        │  WGC 采集 + Vello│
        └───────────────┘                        └──────────────────┘
```

应用入口与初始化序列见 [`src-tauri/src/lib.rs`](../src-tauri/src/lib.rs) 的 `run()`：
插件注册 → `ConfigState` 加载 → `OverlayManager::new` → 注册全部热键 → 建托盘 → 显示主窗口 → 检测热键冲突 / 混合 DPI → GDI 预热 → 启动热键监听线程 → （若启用）Vello 异步预热。

## 真实模块树（后端）

> ⚠ 归档文档中的文件路径已全部失效（旧版为 `state.rs`/`interaction.rs`/`save.rs`/`manager.rs` 单文件，现已模块化）。以下为 `main` 当前真实结构。

```
src-tauri/src/
├── main.rs                     入口（panic hook + run()）
├── lib.rs                      Tauri Builder、setup、invoke_handler 总表
├── app_state.rs                AppState（共享状态容器）
├── commands/                   IPC 命令实现
│   ├── capture.rs              start_capture / trigger_capture
│   ├── pin.rs                  PIN 相关命令
│   └── config/                 配置类命令（io/hotkey/snapshot/vello/workflow）
└── service/
    ├── config/                 AppConfig 类型、默认值、加载/迁移、热键注册
    │   ├── types.rs            ★ AppConfig / CaptureWorkflow / CaptureAction
    │   ├── defaults.rs  manager.rs  io.rs  hotkey.rs  fonts.rs  l10n…
    ├── hotkey/mod.rs           全局热键监听线程
    ├── workflow/mod.rs         execute_capture_workflow：热键→采集→显示
    ├── activity.rs             活动 feed 持久化 + get_activity
    ├── ocr.rs                  OCR 命令与结果
    ├── stitch.rs               长截图拼接
    ├── pin.rs                  PIN 状态与窗口
    ├── tray/mod.rs             系统托盘
    ├── notification.rs  logger.rs  l10n.rs
    ├── native_overlay/         ★★ 截图内核
    │   ├── mod.rs              模块导出
    │   ├── manager/            OverlayManager（双 HWND 生命周期）
    │   │   ├── mod.rs creation.rs lifecycle.rs actions.rs engine.rs
    │   ├── capture/            采集分发
    │   │   ├── mod.rs gdi.rs wgc.rs
    │   ├── engine_mgmt.rs      GDI→Vello 会话内升级
    │   ├── events.rs           WM_* 消息处理
    │   ├── handlers.rs         鼠标按下/移动/抬起分发
    │   ├── input_handler/      cursor / keyboard / mouse
    │   ├── interaction/        mouse_down / mouse_up / mouse_move/*
    │   │   └── mouse_move/     drawing_handler, handle_selection, hover,
    │   │                       move_selection, resize_selection, transform_object
    │   ├── modes/              ★ 四种采集模式（独立）
    │   │   └── mod.rs(trait CaptureModeHandler) selection/fullscreen/window/snapshot
    │   ├── magnifier.rs        放大镜逻辑
    │   ├── snapping.rs         窗口磁吸
    │   ├── scrolling.rs        滚动长截图命令
    │   ├── state/              状态机 SSOT
    │   │   ├── overlay_state/  data.rs(字段) logic.rs(行为) mod.rs
    │   │   ├── drawing_object/ bounds, hit_test, mosaic, mod
    │   │   └── types.rs        CaptureEngine / InteractionMode / DrawingTool 等枚举
    │   ├── render/             渲染分发
    │   │   ├── mod.rs          render_frame：GDI / Vello 分发
    │   │   ├── selection.rs magnifier.rs
    │   │   ├── drawing/        GDI 绘图（tools/: arrow,effects,freehand,number,shapes,text）
    │   │   ├── toolbar/        工具栏（builder,layout,render,property_bar,tooltip,widgets…）
    │   │   └── vello_engine/   Vello 渲染
    │   │       ├── init.rs offscreen.rs surface.rs rendering.rs mod.rs
    │   │       └── renderer/   mod.rs + tools/ + ui/(selection,toolbar,magnifier,
    │   │                       icons/,property_bar/,tooltip) + utils/(styles,text)
    │   └── save/               clipboard / file / pin_capture / render / utils / mod
    └── win32/                  原生封装
        ├── monitor.rs          显示器枚举（rect/dpi/name）
        ├── window/             creation,attributes,enumeration,wnd_proc,types,mod
        ├── gdi/                dc,cache,effects,resources,shapes,text,mod
        ├── gdiplus/            drawing,text,wrappers,mod
        ├── wgc/                capture(WgcStreamManager/OneShot),mod
        ├── bitmap.rs clipboard.rs send_sync.rs
```

前端结构见 [05-FRONTEND](05-FRONTEND.md)。

## IPC 命令表

> 来源：[`lib.rs`](../src-tauri/src/lib.rs) `invoke_handler`（这是命令的**唯一真相**——前端调用未在此注册的命令会失败，见 [08](08-AUDIT.md)）。

### 采集
| 命令 | 作用 |
|------|------|
| `start_capture` | 启动一次交互式采集 |
| `trigger_capture(action)` | 按 workflow id 直接触发（仪表盘 ZAP 按钮用） |

### 配置（`get_config` + setter 群）
`get_config`, `set_save_path`, `set_font_family`, `set_vello_enabled`, `set_vello_advanced_effects`, `set_vello_aesthetic_style`, `set_theme`, `set_accent_color`, `set_jpg_quality`, `set_concurrency`, `set_default_export_format`, `set_language`, `set_quick_save`, `set_snapshot_enabled`, `set_snapshot_size`, `set_selection_engine`, `set_snapshot_engine`, `emergency_reset_to_gdi`

### 工作流 / 热键 / 引擎
`add_workflow`, `remove_workflow`, `update_workflow`, `get_vello_status`, `suspend_hotkeys`, `resume_hotkeys`, `refresh_hotkeys`

### 文件 / 系统
`select_folder`, `open_folder`, `reveal_logs`, `clear_logs`

### PIN
`create_text_pin`, `get_all_pins`, `remove_pin`, `clear_all_pins`, `toggle_pin_always_on_top`, `is_pin_always_on_top`, `set_pin_window_size`, `set_pin_min_size`, `save_pin_as`

### OCR / 滚动 / 活动
`execute_ocr`, `get_last_ocr_result`, `start_scrolling`, `stop_scrolling`, `get_last_scrolled_image`, `save_scrolled_image_to`, `copy_image_to_clipboard`, `get_activity`

## 后端 → 前端事件（`emit`）

| 事件 | 触发 | 监听方 |
|------|------|--------|
| `shortcut-startup-error` | 启动时热键注册冲突 | `StartupErrorToast` / `TauriEventListener` |
| `mixed-dpi-detected` | 多屏混合 DPI | 前端提示 |
| `activity://updated` | 活动记录新增 | `ActivityHub` |

## 核心数据流（一次截图）

```
全局热键命中
  → hotkey/ 监听线程 → workflow/mod.rs::execute_capture_workflow
    → 解析 CaptureAction（模式+引擎）→ 写入 OverlayState
    → capture/ perform_capture：GDI BitBlt  或  WGC 帧
    → manager/ show_overlay_at：选定 active_hwnd，SetWindowPos
    → render/ render_frame：UpdateLayeredWindow  或  Vello Scene→DXGI
  → 用户交互（选区/绘图）→ interaction/ + handlers/ → 重渲染
  → 保存/复制（save/）或 PIN（save/pin_capture）→ activity 记录
  → ESC → manager/lifecycle close_and_reset 重置状态
```

详见 [03-ENGINES](03-ENGINES.md)（引擎细节）与 [04-CAPTURE-MODES](04-CAPTURE-MODES.md)（模式细节）。
