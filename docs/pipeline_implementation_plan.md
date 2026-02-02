# Implementation Plan: Dynamic Action Pipeline (NexSpot 2.0)

Transform NexSpot from a simple screenshot tool into a visual data pipeline by implementing a dynamic, plugin-based toolbar.

## Goal

Enable users to define custom "Tools" (AI prompts, Webhooks, Local Scripts) in the frontend that appear as buttons on the native overlay's toolbar.

## Proposed Changes

### 1. Data Contract (JSON Schema)

Define the structure for a "Skill/Tool".

- `id`: Unique identifier.
- `label`: Display name.
- `icon`: Icon identifier (compatible with GDI font icons).
- `type`: `AI_PROMPT` | `WEBHOOK` | `LOCAL_SCRIPT`.
- `payload`: Config specifics (e.g., prompt text, URL, file path).

### 2. Frontend (React/Vite)

- **Settings UI**: A new "Skills/Marketplace" tab to add/edit tools.
- **Persistence**: Save tools to `app_settings.json`.

### 3. Backend (Rust - Native Overlay)

#### [MODIFY] [render/toolbar.rs](file:///f:/my%20ai/HyperLens/src-tauri/src/service/native_overlay/render/toolbar.rs)

- Update `Toolbar` to load dynamic tools from `AppState` or a shared config file.
- Implement icons for dynamic tools (Mapping label/icon to GDI glyphs).

#### [MODIFY] [mod.rs](file:///f:/my%20ai/HyperLens/src-tauri/src/service/native_overlay/mod.rs)

- Update `on_mouse_down` to detect clicks on dynamic tools.
- Emit a Tauri event (`hyperlens://action-trigger`) when a dynamic tool is clicked, passing the `ActionID` and current `SelectionBuffer`.

### 4. Action Runner (Tauri/Frontend)

- Implement an event listener in the main Tauri window (background).
- When an action is triggered:
    1. Capture the current selection as a Base64 image.
    2. Route it to the appropriate processor (e.g., send via `fetch` to OpenAI).
    3. Display results in a non-intrusive notification or sidebar.

---

## Verification Plan

### Manual Verification

1. Define a "Mock AI" tool in Settings.
2. Take a screenshot.
3. Click the new "Mock AI" button on the toolbar.
4. Verify that a Tauri event is received with the selection data.
5. Verify that a mock response is displayed to the user.
