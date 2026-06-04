# NexSpot Capture Engine Reference (Frozen v0.2.3)

> **Date**: 2026-02-25  
> **Status**: Frozen — reflects the Dual-HWND Engine Isolation architecture + verified render semantics

---

## 1. Architecture Overview

NexSpot supports two independent capture and rendering engines, each with a **dedicated HWND** to prevent cross-contamination:

| Property | GDI Engine | Vello (WGC) Engine |
|---|---|---|
| **Dedicated HWND** | `gdi_hwnd` (class: `HyperLensOverlayGDI`) | `vello_hwnd` (class: `HyperLensOverlayVello`) |
| **Capture API** | Win32 `BitBlt` (full virtual desktop) | Windows Graphics Capture (per-monitor) |
| **Render API** | GDI `UpdateLayeredWindow` | WGPU/DirectX via Vello |
| **Window Style** | `WS_EX_LAYERED` | DWM Composition (non-layered) |
| **Background Storage** | `state.gdi.hbitmap_dim` / `hbitmap_bright` (HBITMAP) | `state.vello.background` (ImageData RGBA8) |
| **Coordinate System** | Global virtual desktop, offset by `state.x`/`state.y` | Local to target monitor, transformed via `global_transform` |
| **Multi-Monitor** | Single composite bitmap of entire virtual desktop | Individual per-monitor capture via WGC stream or one-shot |

```
┌─────────────────────────────────────────────────────────┐
│                    Workflow Trigger                      │
│ (hotkey → workflow/mod.rs → CaptureAction + engine)     │
└────────────────────────┬────────────────────────────────┘
                         │
                         ▼
              ┌─────────────────────┐
              │  perform_capture()  │  ← capture.rs
              │  Reads: engine enum │
              └──────┬─────┬────────┘
                     │     │
          ┌──────────┘     └───────────┐
          ▼                            ▼
    ┌───────────┐              ┌──────────────┐
    │ GDI Path  │              │  WGC Path    │
    │ BitBlt    │              │ Stream/1-shot│
    │ full VDT  │              │ per-monitor  │
    └─────┬─────┘              └──────┬───────┘
          │                           │
          ▼                           ▼
    state.gdi.*               state.vello.background
          │                           │
          └─────────┬─────────────────┘
                    ▼
          ┌──────────────────┐
          │ show_overlay_at()│  ← manager.rs
          │  active_hwnd()   │  ← selects correct HWND
          │ → render_frame() │
          └──────┬───────────┘
                 │
        ┌────────┴──────────────┐
        ▼                       ▼
  GDI Render               Vello Render
  (gdi_hwnd)               (vello_hwnd)
  Layered Window           WGPU Surface
```

---

## 2. Engine Selection Flow

### 2.1 Configuration Source

- **`config.vello_enabled`**: Global toggle stored in `AppConfig` (persisted JSON)
- **`workflow.action.engine`**: Per-workflow engine string (`"gdi"` or `"vello"`)

### 2.2 Dispatch Chain (workflow/mod.rs → capture.rs → manager.rs)

```
1. workflow/mod.rs:
   engine_enum = if target_engine == "vello" { Wgc } else { Gdi }
   state.capture_engine = engine_enum
   → perform_capture(state, wgc_stream_states)

2. capture.rs (perform_capture):
   match state.capture_engine {
       Gdi => { BitBlt full VDT → hbitmap_dim/bright }
       Wgc => { Stream frame or One-shot → vello.background }
   }
   return (x, y, w, h)  // GDI: full VDT coords, WGC: single monitor coords

3. manager.rs (show_overlay_at):
   Re-derives engine from active_workflow (sacred, never overridden)
   → SetWindowPos(hwnd, x, y, w, h)
   → render_frame()
```

### 2.3 Engine Priority Rules (show_overlay_at lines 233-254)

1. If `active_workflow` exists → use its `action.engine` (sacred)
2. Else → use `state.capture_engine` from config
3. Auto-upgrade to Vello only when NO active workflow AND `vello_pref == true`

---

## 3. GDI Engine Details

### 3.1 Capture (capture.rs, GDI branch)

- **Scope**: Full virtual desktop (`union_rect` of all monitors)
- **API**: `BitBlt(hdc_mem, 0, 0, w, h, hdc_screen, x, y, SRCCOPY)`
- **Output**: Two HBITMAPs:
  - `hbitmap_bright`: Original screenshot (used for selection highlight cutout)
  - `hbitmap_dim`: Darkened copy via `AlphaBlend` with 45% opacity black overlay
- **Dim Mask**: Created via 1:1 `AlphaBlend` using a full-size black bitmap (NOT 1x1 stretch — driver-safe)

### 3.2 Render (render/mod.rs, GDI path)

- Window style: `WS_EX_LAYERED = true`
- Background: `BitBlt` from `hbitmap_dim` to backbuffer
- Selection cutout: `BitBlt` from `hbitmap_bright` for selected region
- Drawing objects: GDI drawing module renders shapes/text
- Output: `UpdateLayeredWindow(hwnd, hdc_mem, point, size, hdc_mem, ...)`

### 3.3 Coordinate System

- All coordinates in global virtual desktop space
- Device coordinates = global coords - `state.x`, `state.y`
- **Critical**: `SetWindowOrgEx` is NOT used on memory DCs (caused bugs in multi-monitor setups)
- Cross-monitor selection: allowed unless DPIs are mixed (clamped via `restrict_to_monitor`)

---

## 4. Vello (WGC) Engine Details

### 4.1 VelloContext Lifecycle (vello_engine/mod.rs)

```
App Start (vello_enabled=true)
  └→ async spawn: VelloContext::new(hwnd)
       ├→ wgpu::Instance::default()
       ├→ instance.request_adapter(HighPerformance)
       ├→ adapter.request_device()
       └→ Renderer::new(device, AaSupport::all())
     Result stored in: OverlayManager.vello_ctx = Some(Arc<VelloContext>)

close_and_reset() — ESC pressed
  └→ Clear surfaces/configs/caps/proxy_textures (HashMap::clear)
     VelloContext (device, adapter, renderer) stays ALIVE
     ⚠ CRITICAL: Do NOT set vello_ctx = None (causes 10s GPU re-init)

App Exit (Drop)
  └→ stop_pre_heat() → wgc_stream.take()
     VelloContext dropped naturally with OverlayManager
```

**Key Struct Fields** (`VelloContext`):

| Field | Type | Purpose | Cleared on ESC? |
|---|---|---|---|
| `instance` | `Arc<Instance>` | WGPU instance | ✗ |
| `adapter` | `Arc<Adapter>` | GPU adapter | ✗ |
| `device` | `Arc<Device>` | GPU device | ✗ |
| `queue` | `Arc<Queue>` | Command queue | ✗ |
| `renderer` | `Mutex<Renderer>` | Vello renderer | ✗ |
| `scene` | `Mutex<Scene>` | Current scene | ✗ (reset per-frame) |
| `surfaces` | `Mutex<HashMap<isize, Arc<Surface>>>` | DXGI swapchains keyed by HWND | ✓ |
| `surface_configs` | `Mutex<HashMap<isize, SurfaceConfiguration>>` | Surface configs | ✓ |
| `surface_caps` | `Mutex<HashMap<isize, SurfaceCapabilities>>` | Surface capabilities | ✓ |
| `proxy_textures` | `Mutex<HashMap<isize, (Texture, TextureView)>>` | Proxy textures | ✓ |

### 4.2 WGC Capture Pipeline

#### 4.2.1 WgcStreamManager (wgc/capture.rs)

Pre-heat streaming capture running in background threads:

```
OverlayManager::new() or start_pre_heat()
  └→ WgcStreamManager::new() + .start()
       └→ For each Monitor::enumerate():
            └→ std::thread::spawn → WgcStreamHandler::start(settings)
               └→ on_frame_arrived: frame.buffer().to_vec() → StreamState.image
               └→ on_closed: StreamState.is_alive = false
```

**StreamState** (per-monitor, keyed by `usize` index):

| Field | Purpose |
|---|---|
| `image: Option<ImageData>` | Latest RGBA8 frame from WGC |
| `size: (u32, u32)` | Frame dimensions |
| `stop: bool` | Signal to stop thread |
| `is_alive: bool` | Thread health indicator |

**Monitor Index**: Uses `usize` enumeration index (NOT monitor name string — identical monitors have identical names causing HashMap collisions).

#### 4.2.2 Capture Priority (capture.rs, WGC branch)

```
1. Try stream cache: states_map.get(target_monitor_index)
   → Check is_alive AND image.is_some()
   → If alive: use cached frame (fast path, ~0ms)
   → If dead: log warning, fall through

2. Fallback: One-shot capture via capture_monitor_to_vello()
   → Monitor::enumerate() → find by index
   → OneShotHandler::start(settings) → frame.buffer().to_vec()
   → ~100-200ms
```

#### 4.2.3 stop_pre_heat Lifecycle

```
stop_pre_heat():
  self.wgc_stream.take()  // Takes ownership, drops stream manager
  → Sets stop=true on all StreamState entries
  → Thread handles dropped, threads terminate

start_pre_heat():
  if self.wgc_stream.is_none() → create new WgcStreamManager
```

**Critical**: `stop_pre_heat` uses `.take()` (not `&mut`) to ensure the stream manager is fully dropped, allowing `start_pre_heat` to create a fresh one.

### 4.3 Vello Render Pipeline (renderer/mod.rs)

```
render_state_to_scene(state, toolbar, vello_ctx, scene):
  scene.reset()

  1. Background: scene.draw_image(vello.background, bg_transform)
     → bg_transform = scale_non_uniform(window_w/bg_w, window_h/bg_h)
     → Compensates for DPI or capture size mismatches

  2. Compute Global Transform:
     global_transform = Affine::translate(-(state.x), -(state.y))
     → Shifts global coordinates to window-local (0,0) space

  3. Inner Scene (intermediate, drawn in GLOBAL coordinates):
     inner_scene = Scene::new()
     → draw_selection_ui → 4-rect dim mask + selection cutout + handles
     → draw_object (for each committed DrawingObject — with optional glow/shadow/opacity layers)
     → draw_magnifier
     → draw_toolbar_ui
     → draw_tooltip_ui (Parley text layout)
     → draw_tool_preview (brush/mosaic circle)

  4. Append with Explicit Transform:
     scene.append(&inner_scene, Some(global_transform))
     → Applies global_transform to EVERY transform in inner_scene (O(N))
     ⚠ This is the ONLY place where global→local coordinate shifting happens.
     ⚠ push_layer transform only applies to the clip shape, NOT child geometry.
       Do NOT use push_layer for coordinate shifting.
```

**Clip Rect**: Static `[-1000000.0, 1000000.0]` — used for glow/shadow/opacity `push_layer` calls within inner_scene. Intentionally oversized to avoid clipping any multi-monitor content.

### 4.4 Dual-HWND Window Management (manager.rs)

**Architecture**: Each engine has a **dedicated, permanently assigned HWND**. They are NEVER shared.

| Field | Class Name | Purpose | Created When? |
|---|---|---|---|
| `gdi_hwnd` | `HyperLensOverlayGDI` | GDI Layered Window rendering | Always (startup) |
| `vello_hwnd` | `HyperLensOverlayVello` | WGPU/DXGI Swapchain rendering | Only if `vello_enabled` |

**Why**: Windows DWM treats DXGI Swapchain and Layered Window as **separate compositing layers** with DXGI taking higher priority. If both engines share the same HWND, the DXGI residual frame contaminates GDI captures. No amount of GDI-level flushing (`UpdateLayeredWindow`) or DWM API calls (`DwmExtendFrameIntoClientArea`) can clear DXGI residuals from the other layer.

**Key Methods**:

- `active_hwnd(engine)` → returns the correct HWND for the given engine
- `show_overlay_at()` → shows the active HWND, hides the other
- `close_and_reset()` → hides BOTH HWNDs
- `set_user_data()` → registers WindowEventHandler on BOTH HWNDs

**Vello Window Rendering**:

- `render_frame` (Vello path): Disables `WS_EX_LAYERED`, enables DWM composition via `DwmExtendFrameIntoClientArea(margins=-1)` on `vello_hwnd`
- `show_overlay_at`: `SetWindowPos(vello_hwnd, x, y, w, h)` — sized to target monitor only
- **Engine upgrade** (engine_mgmt.rs): Uses `vello_hwnd` for `SetWindowPos` and `VelloContext::new()`

### 4.5 Cross-Monitor Selection Constraints

| Engine | Constraint | Implementation |
|---|---|---|
| WGC (Vello) | **Always** restricted to capture monitor | `restrict_to_monitor = Some(target_monitor_rect)` |
| GDI | Restricted only when DPIs are mixed | `restrict_to_monitor = if has_mixed_dpi { Some(...) } else { None }` |

Selection clamping enforced in `mouse_move.rs` for modes: `Selecting`, `Moving`, `Resizing`.

---

## 5. State Isolation & Cleanup (manager.rs)

### close_and_reset() — called on ESC

| State Field | Cleared? | Notes |
|---|---|---|
| `state.is_visible` | → false | |
| `state.selection` | → None | |
| `state.interaction_mode` | → None | |
| `state.objects` | → cleared | |
| `state.vello.background` | → None | RGBA8 ImageData |
| `state.gdi.hbitmap_dim` | → None | GDI bitmap handle |
| `state.gdi.hbitmap_bright` | → None | GDI bitmap handle |
| `state.active_workflow` | → None | |
| `self.vello_ctx` | **KEPT** | Only surfaces cleared (see §4.1) |
| `self.wgc_stream` | **KEPT** | Streams continue running |
| `self.gdi_hwnd` | **Hidden** | KillTimer + hide_window |
| `self.vello_hwnd` | **Hidden** | KillTimer + hide_window |

---

## 6. Known Constraints & Gotchas

1. **Vello `push_layer` does NOT transform geometry** — only transforms the clip shape. Verified in Vello source (`scene.rs:82-117`): the `transform` arg only applies to the clip path via `encode_transform` + `encode_shape`. Use `scene.append(inner_scene, Some(transform))` for actual coordinate shifting (applies transform to every inner element, O(N)).

2. **Monitor indices can shift** — `Monitor::enumerate()` from `windows-capture` and `EnumDisplayMonitors` from Win32 may return different orderings. The system uses `usize` indices as keys, which stay consistent within a single `enumerate()` call but may differ between the two APIs.

3. **WGC stream threads die silently** — During DRM events, resolution changes, or monitor sleep/wake, the capture thread's `on_frame_arrived` stops being called. The `is_alive` field tracks this. Dead streams fall through to one-shot capture automatically.

4. **GDI `AlphaBlend` stretching** — Stretching a 1×1 black pixel across 4K resolution fails on certain WDDM drivers. Always use full-size source bitmaps for `AlphaBlend`.

5. **`SetWindowOrgEx` corrupts `UpdateLayeredWindow`** — Do NOT use `SetWindowOrgEx` on memory DCs that are later passed to `UpdateLayeredWindow`. The DC's origin offset affects the `pptSrc` parameter silently.

6. **VelloContext GPU init costs ~5-10s** — Never drop/recreate VelloContext between capture sessions. Only clear DXGI surfaces (instant, ~0ms).

7. **DWM DXGI/Layered Isolation (RESOLVED)** — DWM treats DXGI Swapchain and Layered Window as **independent compositing layers** with DXGI on top. If both engines share one HWND, the DXGI residual frame contaminates GDI captures. **Resolved via Dual-HWND isolation** (v0.2.2): `gdi_hwnd` is never touched by DXGI, `vello_hwnd` is never used for GDI. Previous failed approaches: GDI flush, DWM disable, HWND recreation, WGPU transparent present.

8. **Parley uses `new_for_non_complex_scripts()` for segmenters** — This excludes Japanese, Chinese, Thai, Khmer, Myanmar, and Lao from LSTM/dictionary-based segmentation. Adding `lstm` feature to `icu_segmenter` in Cargo.toml does NOT change Parley's constructor choice. If CJK segmentation errors occur, the fix must be in Parley upstream, not in feature flags.

9. **Global hotkey `AlreadyRegistered` = external occupation** — `global_hotkey::Error::AlreadyRegistered` means another OS-level application (e.g., WeChat, Discord) already owns the hotkey. NexSpot will NEVER receive events for it. Must report as error, not silently succeed.

10. **High-frequency logs must use `trace!`** — `WM_MOUSEMOVE`, per-frame render state, and selection rect logs fire at 60+ Hz. Using `info!` causes log file explosion and potential I/O blocking in the render loop. Always use `log::trace!()` for per-frame/per-mousemove diagnostics.

---

## 7. File Map

```
src-tauri/src/service/
├── native_overlay/
│   ├── capture.rs         — perform_capture(): GDI/WGC dispatch
│   ├── manager.rs         — OverlayManager: dual-HWND lifecycle, active_hwnd(), render_frame, close_and_reset
│   ├── engine_mgmt.rs     — upgrade_to_vello(): GDI→Vello mid-session upgrade
│   ├── events.rs          — WM_* message handling (mouse, keyboard, timer)
│   ├── handlers.rs        — on_mouse_down/up/move dispatchers
│   ├── interaction/
│   │   └── mouse_move.rs  — Selection/Moving/Resizing with restrict_to_monitor clamping
│   ├── state/
│   │   ├── overlay_state.rs — OverlayState struct (all state fields)
│   │   └── types.rs       — CaptureEngine, InteractionMode, DrawingTool enums
│   └── render/
│       ├── mod.rs          — render_frame(): GDI/Vello dispatch
│       ├── selection.rs    — GDI selection rendering
│       ├── drawing.rs      — GDI drawing object rendering
│       └── vello_engine/
│           ├── mod.rs      — VelloContext struct + create_surface + render
│           └── renderer/
│               ├── mod.rs  — render_state_to_scene (inner_scene + append pattern)
│               ├── ui/     — selection, toolbar, magnifier, icons, property_bar
│               └── tools/  — shapes, arrow, freehand, effects, text, number
├── win32/
│   ├── wgc/
│   │   └── capture.rs     — WgcStreamManager, OneShotHandler, StreamState
│   ├── monitor.rs         — enumerate_monitors (DPI, name, rect)
│   ├── gdi.rs             — BitBlt, AlphaBlend, CreateCompatibleBitmap wrappers
│   └── window.rs          — CreateOverlayWindow, SetWindowPos, UpdateLayeredWindow
└── workflow/
    └── mod.rs              — execute_workflow: hotkey → capture → show_overlay
```
