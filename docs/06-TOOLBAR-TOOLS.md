# 06 · 原生工具栏工具规格

> 本文整理自归档的 `TOOLS.md`（产品规格/需求，非全部已实现）。工具栏与绘图在**原生层**渲染：GDI 路径见 `render/toolbar/` + `render/drawing/tools/`；Vello 路径见 `render/vello_engine/renderer/ui/toolbar.rs` + `renderer/tools/`。
> 实现完成度与规格差异见 [08-AUDIT](08-AUDIT.md)。

## 通用交互模型（所有绘图工具）

- **二级菜单**：选中工具后弹出属性条（大小、颜色、填充、透明度等）。属性条实现见 `render/.../property_bar/`。
- **选中逻辑**：点击已绘制对象的命中区域进入选中态（命中测试见 `state/drawing_object/hit_test.rs`）。
- **选中态**：显示方向手柄 + 手柄间连接线；拖边框移动整体，拖方向点改大小/位置；点击空白处退出选中。
- **退出**：点击其它位置取消选中。

## GDI 引擎工具栏

GDI 工具为**纯色、无透明度、无风格化**（极速优先）。

| 工具 | 手柄 | 属性 | 备注 |
|------|------|------|------|
| 矩形 | 8 向点 + 连接线 | 大小、填充、颜色 | 点边框选中 |
| 椭圆 | 8 向点 + 连接线 | 大小、填充、颜色 | 点圆弧任意位置精准选中 |
| 直线 | 2 端点 + 连接线 | 大小、颜色 | |
| 箭头 | 6 点（尾1+箭柄2+两翼2+头1）+ 连接线 | 大小、颜色 | 拖动改大小或位置 |
| 画笔 | 范围框选 | 大小、颜色 | 选中后整体拖动 |
| 文字 | 范围选框 | 大小、颜色 | |
| 序号 | 范围选框 | 大小、颜色 | 选白色时字体自动转黑，其余为白 |
| 撤销 | — | — | 撤销上一步 |
| PIN/置顶 | — | — | 见下节 |

## Vello 引擎工具栏

在 GDI 全部能力之上：**每个工具新增"透明度"属性，且受 `AestheticStyle` 风格影响**。

- 设置中选择风格（Default / Neon / PaperCut / Sketch / Glass）后，工具绘制效果与配色随之改变。
- 例如 Neon 不应使用纯色，而是带辉光的霓虹色彩。
- 序号同样有"白色转黑字"规则。
- 风格 → 渲染映射在 `render/vello_engine/renderer/utils/styles.rs` 与各 `tools/`。

## PIN / 置顶（跨引擎共用规格）

点击后将当前选区截图存为临时文档并置顶：

- 标题栏可拖动移动；窗口可缩放。
- 标题栏操作：**下放到下一层**（保留内容但不置顶）、**清空全部**、**关闭**。
- 所有 PIN 在**一个合集窗口**（`#pin-collection`）内以**卡片**形式呈现。
- 新 PIN 图或新 AI 回复 = 一张新卡片。
- 每张卡片独立的：保存、复制、关闭/删除。
- 从卡片内拖拽可把图片/文字粘贴到外部。
- 固定最小 PIN 窗口尺寸；缩小后以**翻页**呈现其余卡片；限制最大数量。
- 定位：PIN 合集 = 临时多剪贴板合集 + AI 信息回流展示窗。
- 后端：`service/pin.rs`；命令见 [02-ARCHITECTURE](02-ARCHITECTURE.md) PIN 段；前端：`src/components/Pin/`。

### 卡片级操作语义
- **保存** → 存到设置的位置。
- **复制** → 复制到剪贴板。
- **关闭** → 关闭当前窗口/卡片。

## AI 截图宏（📝 规格 / 💤 未接线）

> `service/ai/{mod,openai}.rs` 与 `AgentCoreTab.tsx` 已存在，但 AI 命令**未在 `lib.rs` 的 `invoke_handler` 注册**，当前不可用。以下为目标规格：

- 将截图 + 预设提示词一起发给当前生效的 AI，回复回流到 PIN 合集窗口（作为卡片）。
- 提供常驻 `custom` 项，允许用户自定义文字提示词。

## 统一图标（remixicon）

资源：`src-tauri/resources/remixicon.ttf`。

| 工具 | 图标 | 码点 |
|------|------|------|
| 矩形 | square-line | `` |
| 椭圆 | circle-line | `` |
| 箭头 | arrow-right-up-long-line | `` |
| 画笔 | pencil-line | `` |
| 文字 | text | `` |
| 撤销 | arrow-go-back-line | `` |
| PIN | pushpin-line | `` |
| 保存 | save-line | — |
| 复制 | file-copy-line | — |
| 直线 / 序号 / 关闭 | 待定 | — |

> 图标在 Vello 路径的实现见 `renderer/ui/icons/`。
