# Refactoring Plan: Native Overlay Module / 重构计划：原生覆盖层模块

## Current Status Analysis / 现状分析

The `src/service/native_overlay` module has grown significantly in complexity.
`src/service/native_overlay` 模块的复杂度已显著增加。

- **`mod.rs` (~350 lines)**: Acts as a "God Object". It handles: / 作为“上帝对象”，处理以下内容：
  - Window creation and lifecycle (`OverlayManager::new`, `close_and_reset`). / 窗口创建与生命周期管理。
  - Input event routing (`on_mouse_down`, `up`, `move`). / 输入事件路由。
  - **Complex Geometric Logic**: Snapping calculations, collision detection, cursor state management. / **复杂的几何逻辑**: 吸附计算、碰撞检测、光标状态管理。
  - **State Mutation**: Directly modifying `OverlayState` based on raw inputs. / **状态变更**: 基于原始输入直接修改 `OverlayState`。
- **`render.rs` (~200+ lines)**: Handles all GDI drawing (Selection, Toolbar, Magnifier). / 处理所有 GDI 绘制（选区、工具栏、放大镜）。
- **`state.rs`**: Clean, pure data. / 干净的纯数据结构。
- **`capture.rs`**: Clean, focused on IO. / 聚焦于 IO 的纯净模块。

## Problem / 存在的问题

1. **Maintenance Risk**: `mod.rs` mixes "Controller" logic (lifecycle) with "Business Logic" (snapping, resizing math). Adding new interaction features (like keyboard nudging) makes `mod.rs` harder to read.
   **维护风险**: `mod.rs` 混合了“控制器”逻辑（生命周期）与“业务逻辑”（吸附、缩放计算）。添加新的交互功能（如键盘微调）会使 `mod.rs` 难以阅读。
2. **Testability**: You cannot unit test the snapping logic or resizing logic easily because it's tightly coupled with `OverlayManager` and Mutex locks.
   **可测试性**: 无法轻易对吸附逻辑或缩放逻辑进行单元测试，因为它们与 `OverlayManager` 和互斥锁紧密耦合。
3. **Readability**: `on_mouse_move` is a massive function containing heavily nested logic for 3 different modes (`Selecting`, `Moving`, `Resizing`).
   **可读性**: `on_mouse_move` 是一个庞大的函数，包含针对 3 种不同模式（`Selecting`、`Moving`、`Resizing`）的深度嵌套逻辑。

## Proposed Refactoring / 建议的重构方案

### 1. Extract Snapping Logic -> `snapping.rs` / 提取吸附逻辑 -> `snapping.rs`

Move the geometric calculation logic out of `mod.rs`. / 将几何计算逻辑移出 `mod.rs`。

- **New File**: `src/service/native_overlay/snapping.rs`
- **Functions**: / 函数：
  - `snap_coordinate`
  - `collect_snap_lines`
  - `apply_snap(val, targets, threshold) -> (val, snapped)`

### 2. Extract Interaction Logic -> `interaction.rs` / 提取交互逻辑 -> `interaction.rs`

Move the state machine transition logic out of `OverlayManager`. / 将状态机转换逻辑移出 `OverlayManager`。

- **New File**: `src/service/native_overlay/interaction.rs`
- **Struct**: `InteractionHandler`
- **Methods**: / 方法：
  - `handle_mouse_down(state: &mut OverlayState, x, y, is_ctrl: bool)`
  - `handle_mouse_move(state: &mut OverlayState, x, y, is_ctrl: bool, snap_lines: ...) -> (InteractionMode, HitZone)`
  - `handle_mouse_up(state: &mut OverlayState)`
- **Benefit**: `mod.rs` becomes a thin shim that just forwards OS events to `InteractionHandler`.
  **收益**: `mod.rs` 变成了一个精简的垫片层，仅负责将系统事件转发给 `InteractionHandler`。

### 3. (Optional) Split Rendering -> `render/` / (可选) 拆分渲染逻辑 -> `render/`

If `render.rs` grows further, split into: / 如果 `render.rs` 继续增长，可拆分为：

- `render/mod.rs`: Coordinator. / 协调器。
- `render/selection.rs`: Border and handles. / 边框与控制点。
- `render/toolbar.rs`: Toolbar drawing. / 工具栏绘制。
- `render/magnifier.rs`: Magnifier drawing. / 放大镜绘制。

## Implementation Steps / 实施步骤

1. **Phase 1 (Immediate)**: Create `snapping.rs`. Move `snap_coordinate` and `collect_snap_lines`.
   **第一阶段 (立即执行)**: 创建 `snapping.rs`。移动 `snap_coordinate` 和 `collect_snap_lines`。
    - *Low Risk/High Value.* Cleans up `mod.rs` immediately. / *低风险/高价值。* 立即清理 `mod.rs`。
2. **Phase 2 (Recommended)**: Create `interaction.rs`. Move the massive `match` blocks from `on_mouse_move` / `down`.
   **第二阶段 (建议执行)**: 创建 `interaction.rs`。移动 `on_mouse_move` / `down` 中庞大的 `match` 块。
    - *Medium Risk.* Requires careful borrowing handling. / *中等风险。* 需要仔细处理借用关系。
3. **Phase 3**: Refine `render.rs` (Only if adding more UI elements).
   **第三阶段**: 完善 `render.rs`（仅在添加更多 UI 元素时执行）。

## Immediate Action Item / 立即执行项

I recommended starting with **Phase 1 & 2** to significantly reduce `mod.rs` complexity.
建议从 **第一阶段和第二阶段** 开始，以显著降低 `mod.rs` 的复杂度。
