# 实施计划: Phase 1 - Fixed Toolbar + Drawing Tools (标注工具详解)

**当前状态**: 已完成 Phase 0 重构审计。
**本阶段目标**: 实现一套堪比 CleanShot X / QQ 截图 的专业标注系统。

## 1. 核心数据结构 (Data Structures)

#### [MODIFY] [service/native_overlay/render/toolbar.rs](file:///f:/my%20ai/HyperLens/src-tauri/src/service/native_overlay/render/toolbar.rs)

```rust
pub enum ToolType {
    Rect,       // 矩形 (支持空心/实心切换)
    Arrow,      // 箭头 (微信风格: 一头小一头大)
    Ellipse,    // 圆形 (同样支持空心/实心)
    Line,       // 直线 (支持虚线/实线)
    Brush,      // 画笔 (普通) / Highlighter (荧光笔)
    Mosaic,     // 马赛克
    Text,       // 文字 (GDI+ 抗锯齿, 商业级渲染)
    Number,     // 步骤标记 (1, 2, 3...)
    Pin,        // 钉图
    Save,       // 保存
    Copy,       // 复制
    Cancel,     // 取消
    More,       // 更多
}

// 属性控制 (在二级菜单或 State 中)
pub struct DrawingStyle {
    pub color: u32,             // ARGB
    pub stroke_width: f32,      // 3.0, 5.0, 8.0
    pub is_filled: bool,        // 实心/空心
    pub is_dashed: bool,        // 虚线
    pub arrow_size: f32,        // 箭头头部大小
}
```

## 2. 详细交互定义 (Interaction Specs)

### 2.1 矩形 (Rect)

* **默认**: 空心描边 (Stroke)。
* **变体**: 按住 `Shift` 或点击配置切换为 **半透明填充 (Solid 30%)**。
* **控制**: 8 个控制点，支持创建后再次调整大小。

### 2.2 箭头 (Arrow)

* **风格**: **"Tapered Arrow" (渐细箭头)** - 微信/CleanShot 风格。
  * 起点粗，终点细（连接箭头头）。
  * 箭头头是实心三角形，基于线条粗细自动缩放。
* **拖拽**: 拖动终点改变方向，拖动起点改变位置。

### 2.3 步骤标记 (Number Badge)

* **逻辑**: 每次点击生成一个圆形 Badge，数字自动递增 (1 -> 2 -> 3)。
* **右键**: 点击 Badge 可重置或删除。

### 2.4 画笔与荧光笔 (Brush & Highlighter)

* **画笔**: 纯色，不透明。
* **荧光笔 (Highlighter)**:
  * **颜色**: 亮黄/亮绿/亮粉。
  * **混合模式**: **Multiply (正片叠底)** 或 Alpha 50% 覆盖，确保不遮挡底部文字。
  * **笔触**: 方形笔头，模拟真实记号笔。

### 2.5 文字 (Text)

* **渲染**: GDI+ `DrawString`。
* **样式**: 白色文字 + 黑色外描边 (Halo) 或 半透明背景框，确保在任何背景下可读。

### 2.6 线条 (Line)

* 普通直线。
* **虚线支持**: GDI+ `DashStyle::Dash`。长按/配置切换。

---

## 3. 布局与二级菜单 (Layout)

考虑到工具变多，工具栏可能过长：

* **Main Bar**: [Rect] [Arrow] [Brush] [Text] [MoreTools] | [Pin] [Save] ...
* **MoreTools**: 存放 [Line] [Ellipse] [Mosaic] [Number]。
* **Sub Bar (选配)**: 选中某个工具时，在主条上方浮现属性条 (颜色/粗细/实心)，参考 Snipaste 设计。

## 4. 执行修正 (Execution Adjustment)

原计划直接做 Toolbar，现在需同步建立 `DrawingManager` (用于存储和渲染这些复杂的绘图对象)。

1. **Toolbar UI**: 实现按钮和图标。
2. **Drawing State**: 在 `OverlayState` 中增加 `Vec<DrawingObject>`。
3. **Render Impl**: 使用 GDI+ 实现上述复杂的渲染逻辑 (这是难点，尤其是渐细箭头)。

确认后，我将从 `toolbar.rs` 的枚举定义开始，一步步落实这些图形学需求。
