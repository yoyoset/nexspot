# NexSpot

高性能 Windows 截图工具 —— 双引擎（GDI / Vello+WGC）、原生标注、OCR（Windows 内置 + PaddleOCR 组件）、滚动长截图、PIN 贴图。

- 技术栈：Tauri v2 · Rust/Win32 · React 18 + TypeScript + Tailwind v4
- 当前版本：**v0.3.0**（Studio UI 重设计 + OCR 子系统重建）
- 文档：[`docs/`](docs/README.md)（架构 / 引擎 / 配置 / 当前状态见 [09-STATUS](docs/09-STATUS.md)）

## 构建

```bash
npm install
npm run tauri dev     # 开发
npm run tauri build   # 发布（NSIS + MSI，见 src-tauri/target/release/bundle/）
```
