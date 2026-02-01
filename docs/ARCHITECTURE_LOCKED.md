# 📐 Design Guide: Hybrid Native Rendering (Scheme B)

> **Status**: Verified & Fixed (Feb 2026)
> **Goal**: Zero-Latency Capture with Web UI Flexibility.

## Core Architecture

HyperLens uses a **Hybrid Scheme** that combines the performance of Native (Win32/GDI) with the flexibility of Web (React/CSS).

### 1. The Rendering Stack (Z-Order)

The visual stack is composed of two synchronized windows, strictly ordered:

1. **Frontend Window (Top)**: `WebView` (transparent, `AlwaysOnTop`).
    * **Role**: Displays the Dimming Mask (CSS Box Shadow), Selection Tools, Toolbar, and Debug Info.
    * **Click-Through**: NO. It captures all mouse/touch input.
    * **Visual**: Renders a "Hole" in the mask using `box-shadow` to reveal the Native Window below.

2. **Native Window (Bottom)**: `HyperLensNativeOverlay` (Layered, TopMost but below Web).
    * **Role**: Displays the **high-quality, frozen screenshot**.
    * **Click-Through**: YES (`WS_EX_TRANSPARENT`). It allows clicks to pass through to the desktop if the Web Window wasn't covering it, but since Web Window covers it, clicks hit the Web Window.
    * **Visual**: Renders raw pixel data (RGBA) directly from the capture engine via GDI `UpdateLayeredWindow`. No encoding (PNG/JPG) is used, ensuring <20ms latency.

### 2. The Capture Flow (Zero Latency)

1. **Hot-Standby**: `CaptureService` continuously captures frames (DXGI/WGC) in the background to avoid "Cold Start" delay.
2. **Trigger**: User presses Hotkey.
3. **Snapshot**: The *latest* frame is grabbed instantly.
4. **Native Update**: Raw pixels are blitted to the **Native Window** instantly.
5. **Show**: Native Window calls `ShowWindow`. User sees screen "freeze" immediately.
6. **Web Sync**: Frontend is notified (`capture-ready`). It shows the Mask UI.
    * *Note*: Frontend uses **Polling** (100ms) to inspect backend state (Race Condition proof).
    * *Note*: Backend has a safety delay (300ms) for event push, but polling is the primary robustness mechanism.

### 3. Critical Technical Patterns

* **Pixel Format**: Swizzling (RGBA -> BGRA) happens in Rust before GDI blit. Alpha is set to 255 (Opaque).
* **Masking Strategy**: Use `box-shadow: 0 0 0 9999px rgba(0,0,0,0.4)` on the selection box. This creates a perfect "inverted" mask without needing multiple divs or complex path clipping.
* **Input Handling**: Use `PointerEvents` (`onPointerDown`, `setPointerCapture`) instead of MouseEvents to handle multi-monitor dragging reliably.

## Why this works (The "WeChat" Secret)

Native apps like WeChat use pure C++ for everything. We emulate this by doing the *heavy* pixel rendering in C++ (Rust/Win32) and the *light* UI in Web.
* **Native**: Handles 4K/8K bitmaps instantly.
* **Web**: Handles buttons, styling, and drag logic easily.

---
**Do Not Regress**: Do not revert to sending Base64 strings to frontend. Do not revert to "Native Dimming" (Scheme A) as it complicates highlighting.
