# [ABANDONED] HyperLens 1.0.4 Technical Specification Archive

> **⚠️ DEPRECATED**: This document is obsolete. The active architecture is defined in `ARCHITECTURE_LOCKED.md`.
> **Document Type**: Technical Specification  
> **Author**: Claude (Antigravity)  
> **Date**: 2026-01-31  
> **Version**: 1.0.4 "Adaptive DPI Overlay"

---

## 1. Background Research

### 1.1 WeChat Screenshot Architecture Analysis

**Probe Method**: Custom Rust tool (`wechat_probe.exe`) that enumerates all WeChat windows before and after Alt+A activation.

**Key Findings**:

| Aspect | WeChat Implementation |
|--------|----------------------|
| Window creation | **Dynamic** - created on Alt+A, not pre-cached |
| Window count | **1 single window** covering virtual desktop |
| Window size | 2667×1707 (union of both monitors) |
| Window flags | `TOPMOST` only, **no** `TOOLWINDOW` |
| Alt+Tab | **Visible** in task switcher |
| DPI handling | Single window spans all monitors |

**Evidence**:

```json
{
  "new_windows": [{
    "hwnd": "0x001019A0",
    "width": 2667,
    "height": 1707,
    "x": -960,
    "y": -227,
    "class_name": "Qt51514QWindowIcon",
    "flags": ["TOPMOST"]
  }],
  "visibility_changes": []
}
```

---

### 1.2 User Environment

| Monitor | Resolution | DPI | Position |
|---------|-----------|-----|----------|
| DISPLAY1 (Primary) | 1707×960 | Unknown | (0, 0) |
| DISPLAY2 (Vertical) | 960×1707 | Unknown | (-960, -227) |

**Virtual Desktop**: 2667×1707 starting at (-960, -227)

---

## 2. Architecture Decision

### 2.1 Chosen Approach: Adaptive DPI Strategy

After analyzing WeChat and considering DPI complexity:

```
if all_monitors_same_dpi:
    Mode A: Single Window covering virtual desktop
    Cross-screen selection: ENABLED
else:
    Mode B: Per-Monitor Windows
    Cross-screen selection: DISABLED
    Show user notification
```

### 2.2 Rationale

| WeChat Approach | Our Approach | Why Different |
|----------------|--------------|---------------|
| Always single window | Adaptive based on DPI | Avoid DPI scaling artifacts |
| Dynamic creation | Pre-created at startup | Faster response time |
| Visible in Alt+Tab | Optional TOOLWINDOW | Professional behavior |

---

## 3. Technical Constraints

### 3.1 windows-capture Crate Limitation

The `windows-capture` crate can only capture **one monitor at a time**. It uses Windows Graphics Capture API which requires a specific `Monitor` target.

**Implication**: Even for SingleWindow mode, we must:

1. Capture each monitor separately
2. Stitch images into single canvas
3. Display stitched result

### 3.2 Tauri Window Lifecycle

Tauri windows created via `WebviewWindowBuilder::new()` in async contexts have lifecycle issues. Windows should be created during `setup()` phase and reused via show/hide.

### 3.3 TOPMOST Flag

Tauri doesn't expose `TOPMOST` directly. Must use Win32 API:

```rust
SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW)
```

---

## 4. Data Flow

```
┌──────────────────────────────────────────────────────────────────┐
│                        STARTUP                                   │
├──────────────────────────────────────────────────────────────────┤
│ enumerate_monitors()                                             │
│     ↓                                                            │
│ Vec<MonitorData> { device_name, bounds, dpi_scale, is_primary } │
│     ↓                                                            │
│ is_dpi_uniform() → OverlayMode                                   │
│     ↓                                                            │
│ init_overlay_windows() → hidden WebviewWindows                   │
│     ↓                                                            │
│ OverlayState stored in AppState                                  │
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│                      HOTKEY FLOW                                 │
├──────────────────────────────────────────────────────────────────┤
│ GlobalHotKeyEvent received                                       │
│     ↓                                                            │
│ show_overlay() → windows visible with dark mask                  │
│     ↓                                                            │
│ capture_all_monitors() → Vec<(MonitorData, PNG bytes)>          │
│     ↓                                                            │
│ [if SingleWindow] stitch_images() → single PNG                   │
│     ↓                                                            │
│ set_last_capture_raw() → store in memory                         │
│     ↓                                                            │
│ emit "capture-ready" { mode, show_dpi_warning }                  │
│     ↓                                                            │
│ Frontend fetches image, renders as background                    │
│     ↓                                                            │
│ User draws selection                                             │
│     ↓                                                            │
│ [Save/Copy/ESC] → invoke hide_overlay                            │
│     ↓                                                            │
│ Windows hidden, ready for next capture                           │
└──────────────────────────────────────────────────────────────────┘
```

---

## 5. API Contracts

### 5.1 Tauri Events

**`capture-ready`** (Backend → Frontend)

```typescript
interface CaptureReadyPayload {
  mode: "single" | "per_monitor";
  show_dpi_warning: boolean;
}
```

### 5.2 Tauri Commands

**`hide_overlay`** (Frontend → Backend)

```rust
#[tauri::command]
async fn hide_overlay(app_handle: AppHandle) -> Result<(), String>
```

**`get_capture_image`** (Existing, no change)

```rust
#[tauri::command]
fn get_capture_image() -> Result<Vec<u8>, String>
```

---

## 6. File Structure

```
src-tauri/src/
├── lib.rs                    [MODIFY] Add hide_overlay, setup overlay init
├── capture/
│   ├── mod.rs
│   └── engine.rs             [MODIFY] Add capture_all_monitors, stitch_images
├── service/
│   ├── mod.rs                [MODIFY] Export new modules
│   ├── monitor_info.rs       [NEW] Monitor enumeration, DPI detection
│   ├── overlay_manager.rs    [NEW] Window lifecycle, show/hide, TOPMOST
│   ├── hotkey.rs             [MODIFY] New capture flow
│   └── ...
└── Cargo.toml                [MODIFY] Add image crate, Win32 HiDpi feature

src/components/Capture/
├── SelectionOverlay.tsx      [MODIFY] Add DPI warning toast, hide_overlay call
└── SelectionOverlay.css      [MODIFY] Add toast styles
```

---

## 7. Dependencies

### 7.1 New Crate Dependencies

```toml
[dependencies]
image = "0.25"  # For image stitching

[dependencies.windows]
version = "0.58"
features = [
    "Win32_Foundation",
    "Win32_UI_WindowsAndMessaging", 
    "Win32_Graphics_Gdi",
    "Win32_UI_HiDpi"  # NEW: for GetDpiForMonitor
]
```

---

## 8. Test Scenarios

| ID | Scenario | Expected Result |
|----|----------|-----------------|
| T1 | Both monitors 100% DPI | Single overlay window, cross-screen OK |
| T2 | Monitor 1: 150%, Monitor 2: 100% | Per-monitor windows, toast shown |
| T3 | Press hotkey | Immediate dark overlay, then image fills in |
| T4 | Press ESC | Overlay hidden |
| T5 | Alt+Tab during capture | Overlay visible (no TOOLWINDOW by default) |
| T6 | Selection across monitors (T1) | Works, coordinates continuous |
| T7 | Selection across monitors (T2) | Blocked to current monitor |

---

## 9. Future Enhancements

1. **Pre-warmed capture pool**: Start WGC sessions before hotkey for even faster response
2. **TOOLWINDOW option**: Settings toggle to hide from Alt+Tab
3. **GPU-accelerated stitching**: Use WGPU for faster image composition
4. **Cross-screen selection in mixed DPI**: Complex coordinate transformation

---

## 10. References

- [Windows Graphics Capture API](https://learn.microsoft.com/en-us/windows/uwp/audio-video-camera/screen-capture)
- [Per-Monitor DPI Awareness](https://learn.microsoft.com/en-us/windows/win32/hidpi/high-dpi-desktop-application-development-on-windows)
- [Tauri WebviewWindowBuilder](https://docs.rs/tauri/latest/tauri/webview/struct.WebviewWindowBuilder.html)
- [windows-capture crate](https://crates.io/crates/windows-capture)
