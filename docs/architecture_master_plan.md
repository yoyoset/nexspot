# NexSpot 2.0 长期架构设计 (Master Plan)

**版本**: 3.0 (2026 Advanced Edition)
**状态**: 核心指导纲领 (The Bible)
**愿景**: 打造 Windows 平台上最快、最美、最智能的视觉生产力工具。

---

## 1. 核心引擎架构 (Engine Architecture)

NexSpot 2.0 采用 **智能混合引擎 (Smart Hybrid Engine)**，在性能与能力之间寻求极致平衡。

### 1.1 分层捕获策略 (Layered Capture Strategy)

我们定义三种捕获层级，系统根据上下文自动降级或升级：

| 层级 | 技术栈 | 触发场景 | 优势 | 劣势 |
| :--- | :--- | :--- | :--- | :--- |
| **L1: GDI** | `BitBlt` | 静态桌面、文档、代码编辑器、常规选区 | **< 10ms 延迟**，CPU 消耗忽略不计 | 黑屏 (DirectX/DRM)，不支持透明窗口 |
| **L2: WGC** | `Windows.Graphics.Capture` | 全屏游戏、视频会议、高动态内容、快照模式 | **支持 3D/全屏**，GPU 加速 | 初始化开销较高 (>50ms)，显存占用 |
| **L3: DWM** | `DwmGetDxSharedSurface` | (备选) 极少数 WGC 失效的旧版 DX 游戏 | 最后的兼容性防线 | 未公开 API，不稳定性高 |

### 1.2 增强型 WGC (Enhanced WGC 2026)

* **Session Pool**: 预创建捕获会话池，消除初始化延迟，实现 "Zero-Latency" 快照。
* **HDR Tone Mapping**: 针对 HDR 屏幕，在 Shader 中实现 Reinhard 或 ACES 色调映射，防止截图过曝泛白。
* **DirectComposition (DComp)**: 在 Phase 2 引入。让 Overlay 跳过 GDI 窗口系统，直接在 DWM 层面与桌面合成，实现“系统级”丝滑动画。
* **Smart Detector**: 自动探测目标窗口属性（Fullscreen Exclusive / Protected Media Path），智能切换 GDI/WGC。

### 1.3 核心数据结构

* **PhysicalRect**: 所有的坐标计算严格基于物理像素，仅在渲染层做逻辑映射。
* **ImageBuffer**: 统一内存格式 `Vec<u8, RGBA8888>`。

---

## 2. 交互与视觉系统 (Interaction & Visuals)

### 2.1 商业级渲染引擎 (Native Render Engine 2.0)

* **预乘 Alpha**: 所有资源必须是 `Premultiplied Alpha` 格式。
* **多尺度资源池 (Multi-scale Asset Pool)**:
  * 针对 100%, 150%, 200% 等常见 DPI，预生成多套 `HBITMAP`。
  * 避免运行时缩放导致的模糊或性能损耗。
* **九宫格算法 (Pixel-Perfect 9-Slice)**:
  * 使用 `GdiAlphaBlend` 进行合成。
  * **拉伸公式**: $$S_{\text{middle}} = \text{Round}(\frac{W_{\text{target}} - (L_{\text{fixed}} + R_{\text{fixed}})}{W_{\text{source\_middle}}})$$
  * 确保四舍五入对齐，防止 1px 边缘模糊。

### 2.2 交互范式

* **Fixed Toolbar**: 智能跟随，边缘避让。
* **Dynamic Tray**: 类似于手机“快捷设置”，支持手动排序和开关。

### 2.3 反馈系统

* **HUD**: 极简状态栏通知。
* **Haptic Visuals**: 像素级微动效。

---

## 3. 工作流与自动化 (Workflow & RPA)

这是 NexSpot 2.0 的护城河，使其进化为 **轻量级 RPA 工具**。

### 3.1 开放数据协议

所有插件通过标准 JSON 通信，支持 `Image + OCR Text + Metadata` 的上下文传递。

### 3.2 自动化宏 (RPA Macros)

* **防抖动 (Debounce)**: 在连续自动截图时，对比画面相似度 (Perceptual Hash)。只有内容变化显著时才触发后续动作，防止重复无效处理。
* **元数据注入 (Exif Injection)**: 将 OCR 文本或 AI 摘要写入图片文件的 Exif/IPTC 字段，支持系统级搜索。

---

## 4. 安全、隐私与合规

* **Local First**: 默认内存流转。
* **Clear Consent**: 外部 API 调用需明确授权。
* **No Telemetry**: 不收集用户内容。

---

## 5. 实施路线图 (Phase 0 Added)

### Phase 0: Refactoring & Cleanse (重构与清洗) - **[Current Focus]**

* **目标**: 偿还技术债务，为新特性铺平道路。
* **交付物**:
  * 模块解耦: 将 `snapping.rs` (吸附) 和 `interaction.rs` (鼠标逻辑) 彻底分离。
  * 状态机重构: 引入明确的 `OverlayState` (Idle, Snapping, Selecting, Editing)。

### Phase 1: Native UI (原生进化)

* 实现 9-Slice 渲染框架。
* 构建多尺度 Asset Manager。
* 产出具有“果冻感”的高清原生工具栏。

### Phase 2: Hybrid Engine (混合引擎)

* 引入 WGC 和 HDR 映射。
* 实现快照模式和全屏游戏支持。

### Phase 3: Automation (自动化)

* 实现 RPA 宏、Notion 集成和元数据注入。
