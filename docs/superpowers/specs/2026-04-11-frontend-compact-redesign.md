# Front-end Compact Redesign (De-AI & Efficiency Focus)

Date: 2026-04-11
Status: Proposed
Author: Antigravity

## 1. Objective
Refactor the NexSpot user interface to transition from a "consumer web" aesthetic to a "high-density professional efficiency tool." This involves removing all AI-related features and reclaiming screen real estate for a more compact, metadata-rich experience.

## 2. Design Principles
- **Efficiency Over Aesthetics**: Prioritize information density and quick-glance status checks.
- **Rationalist Geometry**: Minimize corner radii (2px max) and remove shadows in favor of thin borders.
- **Split Responsibility Indicators**: Separate hotkey registration status from rendering engine health to eliminate ambiguity.
- **Activity Bar Pattern**: Use a narrow vertical navigation bar to maximize main content horizontal space.

## 3. Scope of Changes

### A. AI Removal (Cleanup)
- **Delete Components**:
  - `src/components/Dashboard/AISection.tsx`
  - `src/components/Dashboard/AIShortcutModal.tsx`
  - `src/components/Overlay/AiPromptModal.tsx`
  - `src/components/Settings/tabs/AgentCoreTab.tsx`
- **Clean up Stores/Types**: Remove AI-specific states and configuration fields from `useAppStore` and `AppConfig`.

### B. Layout Overhaul
- **`Navigator.tsx`**:
  - Reduce width to ~48px.
  - Remove text labels; use icons only (Activity Bar pattern).
  - Use `border-right` instead of full background shading for separation.
- **`Dashboard.tsx`**:
  - Replace `PresetCard` (hero-style) with a unified `DashboardList`.
  - Grid-based rows (Protocol Name, Specs, Ext, Status, Hotkey).
  - Add a dedicated field/icon for "Open Target Directory".

### C. Visual Identity (CSS)
- **`index.css` / `App.css`**:
  - Global reduction of `padding` and `gap` variables.
  - Enforce `font-family: monospace` for technical metadata.
  - Implement a `status-bar` at the bottom for global app health.

## 4. Technical Details

### Dashboard Row Structure
| Field | Data Source | Visualization |
| :--- | :--- | :--- |
| **Identifier** | `workflow.label` | Bold, primary color |
| **Specs** | `action.type` + `engine` | Muted mono text (e.g., `GDI:Select`) |
| **Ext** | `output.format` | Mono text |
| **KEY Status** | `hotkeyStatus.registrations` | Icon (Blue = OK, Red = Conflict) |
| **ENG Status** | `velloStatus` / `gdiStatus` | Icon (Green = Ready, Yellow = Pending) |
| **Dir** | `output.target_folder` | Folder icon (triggers `shell.open`) |
| **Hotkey** | `workflow.shortcut` | Highlighted mono text |

## 5. Implementation Sequence
1. **Phase 1**: UI Stripping (Remove AI components and unused styles).
2. **Phase 2**: Global Layout Refactor (Navigator & App Frame).
3. **Phase 3**: Dashboard List Implementation & Metadata Alignment.
4. **Phase 4**: Verification & Polish (DPI scaling, hover states).

## 6. Open Questions
- Should the "Open Directory" icon only appear on hover, or remain static? (Current preference: Static column for "Efficiency" feel).
