# Web 示例

这个目录是 Rust 版 Fantasy Map Generator 的网页演示工程。

当前技术栈：

- `pnpm`
- `Vite 7`
- `React 19`
- `TypeScript 5`
- `Three.js`
- `Mantine 8`
- `Tailwind CSS 4`

前端通过两个 Worker 使用上一级 `pkg/` 中的 `wasm-pack` 产物：

- `mapGenerator.worker.ts`：初始化 WASM、预热、小图生成、按需 JSON 导出
- `svgRender.worker.ts`：后台生成标准 SVG

## 启动前准备

如果你刚修改过 `rust/src/`、`Cargo.toml` 或 WASM 导出接口，先在 `rust/` 目录重建 WASM：

```bash
cd ..
./build-wasm.sh
```

Windows：

```bat
cd ..
build-wasm.bat
```

产物输出到：

- `rust/examples/pkg/`

## 本地开发

```bash
pnpm install
pnpm dev
```

默认 Vite 开发服务器配置见 `vite.config.ts`，当前监听：

- `host: 0.0.0.0`
- `port: 5173`

## 生产构建

```bash
pnpm build
pnpm preview
```

构建输出目录：

- `rust/examples/dist/`

当前构建会把主应用和 vendor 依赖分块：

- `react-vendor`
- `ui-vendor`
- 主应用代码
- `three.webgpu` 等动态渲染依赖

## 常见开发路径

### 只改前端 UI / 交互 / 样式

```bash
pnpm dev
```

通常不需要重建 WASM。

### 改了 Rust 生成逻辑或 WASM 导出契约

```bash
cd ..
./build-wasm.sh
cd examples
pnpm dev
```

### 产出生产包

```bash
pnpm build
pnpm preview
```

### 只改 Rust/WASM 后快速验证网页

```bash
cd ..
build-wasm.bat
cd examples
pnpm build
pnpm preview
```

## 当前示例能力

- 后台生成地图，不阻塞主线程
- Worker 启动后自动做一次轻量 warmup，降低首击生成延迟
- 支持 `WebGPU` 与 `SVG` 两种渲染模式
- 支持 PNG / SVG / JSON 导出
- 支持 JSON 导入
- 支持图层开关、主题切换和中英文切换

说明：

- JSON 导出当前是**按需生成**，首次导出会回源调用 Rust 生成完整 JSON，随后会缓存到当前地图对象中
- 标准 SVG 始终基于 `svg_json` 构建，不依赖完整地图 JSON
- Worker 会先做一次轻量 warmup，所以首次点击“生成地图”通常会比完全冷启动更快

## 运行时流程

页面启动后当前大致顺序如下：

1. 创建 `mapGenerator.worker`
2. worker 初始化 WASM 并做一次轻量 warmup
3. 页面进入 `WASM 已就绪` 状态
4. 用户点击“生成地图”
5. worker 返回 render packet
6. 主线程根据当前渲染模式走 `WebGPU` 或 `SVG`
7. 如用户首次导出 JSON，则再通过 worker 按需生成完整 JSON

## 什么时候需要重建 WASM

需要重建：

- 修改了 `rust/src/`
- 修改了 `Cargo.toml`
- 修改了 `rust/src/wasm.rs`
- 修改了 render packet / SVG 导出契约

通常不需要重建：

- 只修改 `examples/src/`
- 只修改样式、文案、布局

## 关键文件

- `src/App.tsx`：页面状态、导入导出、渲染器切换
- `src/hooks/useMapGenerator.ts`：生成 worker 通道、ready 握手、按需 JSON 导出
- `src/workers/mapGenerator.worker.ts`：WASM 初始化、预热、地图生成
- `src/workers/svgRender.worker.ts`：后台标准 SVG 构建
- `src/lib/FantasyMapThreeRenderer.ts`：Three.js / SVG 双渲染实现
- `src/lib/mapScenePacket.ts`：WASM packet 到前端 packet 的转换
- `src/lib/standardMapSvg.ts`：标准 SVG 导出入口
- `src/types/map.ts`：前端地图数据契约

## 导出行为

当前 3 种导出路径分别是：

- `PNG`：直接从当前渲染器导出位图
- `SVG`：始终通过 `svgRender.worker.ts` 后台构建标准 SVG
- `JSON`：如果当前地图还没有完整 JSON，会先回源请求 Rust 生成一次，然后缓存

这意味着：

- 首次 JSON 导出会比再次导出慢
- 再次导出同一张图时，JSON 基本只走前端缓存

## 常见命令

### 本地开发

```bash
pnpm dev
```

### 生产构建

```bash
pnpm build
```

### 本地预览生产包

```bash
pnpm preview
```

### 前端代码格式化

```bash
pnpm format:write
```

### 前端静态检查

```bash
pnpm lint
```

## 验证建议

```bash
pnpm build
pnpm preview
```

如果你同时改了 Rust 和前端：

```bash
cd ..
./build-wasm.sh
cd examples
pnpm build
```
