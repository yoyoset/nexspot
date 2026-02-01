# 🏗️ Decision Log

## [2026-02-01] Architecture Pivot: Hybrid Native Rendering (Scheme B)

### Context

Originally planned to use **Scheme A (Native Dimming)** where the native overlay window renders a dimmed screenshot.
**Blocker**: The frontend needs to "highlight" the selection (show original brightness). Since the backend (to optimize for zero-latency) does **not** send image data to the frontend, the frontend cannot reconstruct the original bright pixels to overlay on the dimmed background.

### Decision

Switch to **Scheme B (Hybrid Masking)**.

#### Technical Details

1. **Native Backend**: Renders the **Original, Bright, Undimmed** screenshot via GDI directly (`UpdateLayeredWindow`).
    * *Benefit*: Zero encoding latency, zero IPC transfer.
2. **Web Frontend**: Renders a **Dimming Mask** (`bg-black/40`) over the entire screen.
3. **Interaction**: When user selects a region, the Frontend "punches a hole" in the mask (renders 4 surrounding divs), revealing the bright Native background underneath.

### Comparison

| Feature | Scheme A (Native Dimming) | Scheme B (Hybrid Masking) |
| :--- | :--- | :--- |
| **Highlight Logic** | Impossible without IPC | Trivial (CSS Mask) |
| **Performance** | High | High (Zero Copy) |
| **Complexity** | High (Dual Native Windows) | Low (Native + Web) |

### Outcome

* Eliminated 1.5s encoding latency.

* Achieved expected "WeChat-like" visual interaction.
* Codebase simplified (removed resizing/cropping logic from backend).
