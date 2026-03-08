# Fantasy Map Generator Web Demo

网页示例已重构为 `pnpm + Vite + React + TypeScript + Tailwind CSS + Mantine` 技术栈，并补齐了暗色模式与中英文国际化支持。

## 开发

```bash
pnpm install
pnpm dev
```

## 构建

```bash
pnpm build
pnpm preview
```

## 重新构建 WASM

修改 Rust 代码后，在 `rust/` 目录执行：

```bash
./build-wasm.sh
# 或 build-wasm.bat
```

## 主要能力

- React 组件化控制面板、状态栏与地图视图
- Mantine 主题 + CSS Variables + Tailwind 协同样式
- 深色 / 浅色模式切换并持久化
- 中英文界面切换并持久化
- Three.js 统一驱动的 WebGPU / SVG 双后端渲染
- JSON / PNG / SVG 导入导出
