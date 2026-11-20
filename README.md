# NexSpot

高性能 Windows 截图工具 —— 双引擎（GDI / Vello+WGC）、原生标注、OCR（Windows 内置 + PaddleOCR 组件）、PIN 贴图。

- 下载：[GitHub Releases](https://github.com/yoyoset/nexspot/releases/latest)
- 技术栈：Tauri v2 · Rust/Win32 · React 19 + TypeScript + Tailwind v4
- 当前版本：**v0.3.0**（Studio UI 重设计 + OCR 子系统重建）
- 许可证：[MIT](LICENSE)
- 文档：[`docs/`](docs/README.md)（架构 / 引擎 / 配置 / 当前状态见 [09-STATUS](docs/09-STATUS.md)）

## 能做什么

- **截图**：区域框选、全屏、窗口、固定窗口；GDI 与 Vello+WGC 两套引擎
- **标注**：矩形、椭圆、直线、箭头、画笔、文字、序号、马赛克
- **OCR**：Windows 内置识别（零依赖）+ PaddleOCR 组件（应用内一键下载安装）
- **贴图（PIN）**：把截图钉在屏幕上
- **工作流**：自定义截图动作 + 全局热键，状态栏分别显示引擎与热键是否就绪

## 已知限制（v0.3.0 公开版）

- 安装包**没有代码签名**，Windows 会提示「未知发布者」，这是正常的，请对照 Release 页的 SHA256。
- **滚动长截图没有开入口**：代码在 `scrolling.rs` / `stitch.rs`，实机全流程（不同浏览器、列表控件的滚轮响应）还没验完，所以这一版工具栏里不出现这个按钮。
- Vello 引擎下的 OCR 在后一轮改动后**没有复测**；默认引擎已设为 GDI，OCR 请用 GDI 引擎。
- 多屏和高 DPI 只在作者本机的组合上验过，其他组合没测。
- 只有 Windows。Vello 引擎要求显卡支持 WGPU/Vulkan。

## 构建

```bash
npm install
npm run tauri dev     # 开发
npm run tauri build   # 发布（NSIS，见 src-tauri/target/release/bundle/）
```

需要 Node 20+、Rust 1.8x、Windows 10/11 与 WebView2 运行时。
