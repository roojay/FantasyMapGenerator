# Web 示例

这个目录是 Rust 版 Fantasy Map Generator 的前端演示工程，当前技术栈如下：

- `pnpm`
- `Vite`
- `React 18`
- `TypeScript`
- `Three.js`
- `Mantine`
- `Tailwind CSS`

前端通过 Web Worker 加载 `../pkg/fantasy_map_generator.js`，也就是 `wasm-pack` 从上一级 Rust crate 构建出来的产物。

## 启动前准备

如果你刚克隆仓库，或者刚修改过 Rust/WASM 代码，先在 `rust/` 目录重新构建 WASM：

```bash
cd ..
./build-wasm.sh
```

Windows：

```bash
cd ..
build-wasm.bat
```

这一步会把产物输出到：

- `rust/examples/pkg/`

## 本地开发

```bash
pnpm install
pnpm dev
```

## 生产构建

```bash
pnpm build
pnpm preview
```

## 当前示例能力

- 使用 Worker 在后台调用 `WasmMapGenerator`
- 基于 `generate_render_packet(...)` 构建 Three.js 场景
- 支持标准地图与卫星风格 SVG 导出
- 支持 JSON / PNG / SVG 导入导出
- 支持图层开关、主题切换和中英文切换
- 支持从 Rust `presentation` metadata 读取展示插件信息

## 什么时候需要重建 WASM

需要重建：

- 修改了 `rust/src/` 下的 Rust 代码
- 修改了 `Cargo.toml`
- 修改了 WASM 导出接口或 `presentation` 相关逻辑

通常不需要重建：

- 只修改了 `examples/src/` 下的 React/TS/CSS 代码

## 相关文件

- `src/workers/mapGenerator.worker.ts`：生成地图和 render packet
- `src/workers/svgRender.worker.ts`：后台生成 SVG
- `src/lib/FantasyMapThreeRenderer.ts`：Three.js 渲染实现
- `src/lib/presentationPluginMetadata.ts`：读取 Rust 侧插件 metadata
- `src/lib/standardMapSvg.ts`：标准 SVG 导出
- `src/lib/satelliteMapSvg.ts`：卫星风格 SVG 导出
