# CHANGELOG

## [0.2.0] - 2026-02-01

### Added

- **Hybrid Native Rendering (Scheme B)**: Achieved <50ms zero-latency capture using GDI native window for rendering and Webview for UI.
- **Polling State Sync**: Robust frontend-backend state synchronization to prevent race conditions.
- **Pointer Events**: Improved multi-monitor drag selection using `setPointerCapture`.

### Fixed

- **Port 1420 Conflict**: Fixed zombie process issues preventing dev server startup.
- **Visual Artifacts**: Fixed inverted selection and white artifacting by switching to CSS Box-Shadow masking.
- **Event Race Condition**: Fixed "Ready: false" state issue on startup.

### Changed

- Refactored `native_overlay.rs` to remove conflicting `SetLayeredWindowAttributes` calls.
- Upgraded capture pipeline to bypass BMP encoding for direct BGRA blit.
