# Fantasy Map Generator Rust 版本

这个目录包含项目的 Rust 实现，当前同时提供 4 条使用路径：

- `map_generation` CLI：生成地图 JSON，启用 `render` feature 时可额外输出 PNG
- Rust 库：直接调用 `MapGenerator` 和导出数据结构
- `wasm` 绑定：供浏览器侧生成地图、导出 SVG、导出 WebGPU scene packet
- `examples/` 前端示例：`Vite + React + TypeScript + Three.js + Mantine`

## 当前实现概览

- 核心地图生成流程已完成：不规则网格、地形、侵蚀、河流、城市、领土、标签
- 默认导出格式为 JSON
- `render` feature 提供基于 `wgpu` 的 PNG 渲染
- `wasm` feature 提供 `wasm-bindgen` 接口和前端示例依赖的导出能力
- `presentation` 模块提供面向不同展示层的插件式数据输出

## 目录结构

```text
rust/
├── Cargo.toml
├── README.md
├── build-wasm.sh
├── build-wasm.bat
├── src/
│   ├── lib.rs
│   ├── main.rs
│   ├── cli.rs
│   ├── config.rs
│   ├── map_generator.rs
│   ├── wasm.rs
│   ├── standard_svg.rs
│   ├── satellite_svg.rs
│   ├── algorithms/
│   ├── data_structures/
│   ├── presentation/
│   ├── render/
│   ├── citydata/
│   └── fontdata/
├── tests/
│   ├── snapshot_test.rs
│   └── baselines/
└── examples/
    ├── README.md
    ├── src/
    └── package.json
```

## 快速开始

### 1. 仅生成 JSON

```bash
cd rust
cargo run --release -- --seed 12345 --output examples/my_map
```

输出文件：

- `examples/my_map.json`

### 2. 生成 JSON + PNG

```bash
cd rust
cargo run --release --features render -- --seed 12345 --output examples/my_map
```

输出文件：

- `examples/my_map.json`
- `examples/my_map.png`

如果只想在启用 `render` 的情况下跳过 PNG：

```bash
cargo run --release --features render -- --seed 12345 --no-render --output examples/my_map
```

### 3. 生成 JSON + 标准风格 SVG

需要启用 `svg` feature：

```bash
cd rust
cargo run --release --features svg -- --seed 12345 --svg --output examples/my_map
```

输出文件：

- `examples/my_map.json`
- `examples/my_map-standard.svg`

### 4. 生成 JSON + 卫星风格 SVG

需要启用 `svg` feature：

```bash
cd rust
cargo run --release --features svg -- --seed 12345 --satellite-svg --output examples/my_map
```

输出文件：

- `examples/my_map.json`
- `examples/my_map-satellite.svg`

### 5. 生成所有格式

同时启用 `render` 和 `svg` feature：

```bash
cd rust
cargo run --release --features "render svg" -- --seed 12345 --svg --satellite-svg --output examples/my_map
```

输出文件：

- `examples/my_map.json`
- `examples/my_map.png`
- `examples/my_map-standard.svg`
- `examples/my_map-satellite.svg`

### 6. 查看 CLI 参数

```bash
cd rust
cargo run -- --help
```

当前常用参数：

- `--seed` / `--timeseed`
- `--output`
- `--resolution`
- `--erosion-amount`
- `--erosion-steps`
- `--cities`
- `--towns`
- `--size`
- `--draw-scale`
- `--no-slopes`
- `--no-rivers`
- `--no-contour`
- `--no-borders`
- `--no-cities`
- `--no-towns`
- `--no-labels`
- `--no-arealabels`
- `--no-render`，仅在 `render` feature 下可用
- `--svg`，仅在 `svg` feature 下可用
- `--satellite-svg`，仅在 `svg` feature 下可用

补充说明：

- 默认输出路径是 `examples/output`
- `--size` 支持 `1920:1080` 和 `1920x1080`
- `--drawing-supported`、`-v/--verbose` 目前会被解析，但还没有额外行为

## Cargo features

`Cargo.toml` 当前定义了三个可选 feature：

### `render`

用于本地 PNG 渲染，依赖：

- `wgpu`
- `pollster`
- `image`
- `bytemuck`

典型用法：

```bash
cargo run --release --features render -- --seed 12345 --output examples/output
cargo test --features render
```

### `svg`

用于导出 SVG（标准风格和卫星风格），依赖：

- `image`
- `base64`

典型用法：

```bash
cargo run --release --features svg -- --seed 12345 --svg --output examples/output
cargo run --release --features svg -- --seed 12345 --satellite-svg --output examples/output
```

### `wasm`

用于浏览器集成，依赖：

- `wasm-bindgen`
- `js-sys`
- `console_error_panic_hook`
- `wee_alloc`
- `image`
- `base64`

典型用法：

```bash
cargo check --features wasm
./build-wasm.sh
```

Windows 下可使用：

```bash
build-wasm.bat
```

## 地图生成流程

当前 CLI、Rust 库和 `WasmMapGenerator` 共享同一套核心生成逻辑，整体流程如下：

1. 使用 Poisson 圆盘采样生成点集
2. 对点集做 Delaunay 三角剖分，并构造 Voronoi 网格
3. 叠加山丘、圆锥、斜坡等地形原语生成初始高度图
4. 执行归一化、圆滑或松弛，随后做多轮侵蚀
5. 重新设定海平面，计算流向、流量、河流、等高线和坡度阴影
6. 按评分放置城市和城镇，并根据移动成本划分领土
7. 为城市名、地区名生成候选位置，使用模拟退火优化标签布局
8. 导出 `MapDrawData`，再按需要交给 PNG 渲染器、SVG 构建器或 WebGPU scene packet

CLI 中默认的地形初始化与侵蚀策略也与当前实现一致：

- 地形原语随机组合包含 hill、cone、slope
- 侵蚀通过多轮 `erode(amount / iterations)` 完成
- 侵蚀结束后调用 `set_sea_level_to_median()`，保持陆海比例稳定

## 算法说明

当前实现涉及的主要算法与职责如下：

- Poisson 圆盘采样：生成分布均匀的基础点集
- Delaunay 三角剖分：建立邻接关系和三角网格
- Voronoi 图：作为地图不规则网格与地形采样基础
- Planchon-Darboux 洼地填充：保证水流路径可达
- 流向与流量计算：驱动河流与侵蚀
- Dijkstra 移动成本：用于城市势力范围和领土边界
- 模拟退火：用于标签候选位置优化

和原始 C++ 版本保持一致的部分仍然保留，包括：

- `glibc rand` 风格随机数生成器
- 一些看起来“多余”的随机数调用顺序
- 同种子下尽量保持一致的地图生成行为

## 渲染流程

启用 `render` feature 后，`MapRenderer::render()` 当前按下面的顺序执行：

1. 清空背景
2. 绘制坡度阴影
3. 绘制领土边界底色和边界线
4. 绘制河流
5. 绘制等高线
6. 绘制城市标记
7. 绘制城镇标记
8. 处理文字标签

需要注意：

- 第 8 步的文字渲染接口已经接好，但 `TextRenderer` 当前仍是占位实现
- 城市和城镇圆形标记半径会受 `draw_scale` 影响
- `RenderStyle` 中保留了线宽配置字段，但当前 `wgpu` 线条路径仍主要依赖基础线段绘制能力

## 坐标系统

当前文档和实现使用的坐标约定如下：

- `MapDrawData` 中的几何坐标是归一化坐标，范围通常为 `[0, 1]`
- 线段、路径和点在进入 WebGPU 前会执行一次 Y 轴翻转，即 `y = 1.0 - y`
- WGSL 顶点着色器再把 `[0, 1]` 坐标映射到 NDC `[-1, 1]`

这意味着：

- 导出的 JSON 更适合跨渲染后端复用
- WebGPU 渲染器内部负责坐标系适配
- `presentation` / WASM 层可以在不改核心生成逻辑的前提下复用同一份地图数据

## JSON 导出格式

CLI 和库最终导出的核心结构是 `MapDrawData`，主要字段包括：

```json
{
  "image_width": 1920,
  "image_height": 1080,
  "draw_scale": 1.0,
  "contour": [],
  "river": [],
  "slope": [],
  "city": [],
  "town": [],
  "territory": [],
  "label": []
}
```

按导出选项不同，还可能包含这些可选字段：

- `heightmap`
- `flux_map`
- `land_mask`
- `land_polygons`

其中：

- CLI 默认会导出这些栅格数据
- Web 场景可以通过 `MapExportOptions { include_raster_data: false }` 关闭，减少 WASM 到 JS 的数据传输

## 作为 Rust 库使用

### 基础生成

```rust
use fantasy_map_generator::{Extents2d, GlibcRand, MapExportOptions, MapGenerator};

let extents = Extents2d::new(0.0, 0.0, 20.0, 10.0);
let rng = GlibcRand::new(12345);
let mut generator = MapGenerator::new(extents, 0.08, 1920, 1080, rng);

generator.initialize();
generator.add_hill(10.0, 5.0, 3.0, 1.2);
generator.normalize();

let draw_data = generator.collect_draw_data_with_options(MapExportOptions {
    include_raster_data: false,
});
```

### 导出标准 SVG

需要启用 `svg` feature：

```rust
use fantasy_map_generator::standard_svg::build_map_svg;

let map_json = std::fs::read_to_string("examples/output.json")?;
let layers_json = serde_json::json!({
    "slope": true,
    "river": true,
    "contour": true,
    "border": true,
    "city": true,
    "town": true,
    "label": true
})
.to_string();

let svg = build_map_svg(&map_json, &layers_json)?;
std::fs::write("examples/output-standard.svg", svg)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

### 导出仅保留河流和边界的简化 SVG

需要启用 `svg` feature：

```rust
use fantasy_map_generator::standard_svg::build_map_svg;

let map_json = std::fs::read_to_string("examples/output.json")?;
let layers_json = serde_json::json!({
    "slope": false,
    "river": true,
    "contour": false,
    "border": true,
    "city": false,
    "town": false,
    "label": false
})
.to_string();

let svg = build_map_svg(&map_json, &layers_json)?;
std::fs::write("examples/output-rivers-borders.svg", svg)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

### 导出卫星风格 SVG

需要启用 `svg` feature，并且输入 JSON 里必须包含 `heightmap` 和 `land_mask`。

```rust
use fantasy_map_generator::satellite_svg::build_satellite_svg;

let map_json = std::fs::read_to_string("examples/output.json")?;
let layers_json = serde_json::json!({
    "slope": true,
    "river": true,
    "contour": true,
    "border": true,
    "city": true,
    "town": true,
    "label": true
})
.to_string();

let svg = build_satellite_svg(&map_json, &layers_json)?;
std::fs::write("examples/output-satellite.svg", svg)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

### 导出更轻量的卫星预览 SVG

预览 SVG 会通过更小的内嵌纹理和更低的 JPEG 质量，换取更小的文件体积。

```rust
use fantasy_map_generator::satellite_svg::build_satellite_svg_with_options;

let map_json = std::fs::read_to_string("examples/output.json")?;
let layers_json = serde_json::json!({
    "slope": false,
    "river": true,
    "contour": false,
    "border": false,
    "city": false,
    "town": false,
    "label": false
})
.to_string();

let options_json = serde_json::json!({
    "max_embedded_image_size": 512,
    "jpeg_quality": 60
})
.to_string();

let svg = build_satellite_svg_with_options(&map_json, &layers_json, Some(&options_json))?;
std::fs::write("examples/output-satellite-preview.svg", svg)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

### 从内存中的 `MapDrawData` 直接导出卫星 SVG

如果你不想先落盘 JSON，可以直接从 `MapGenerator` 导出：

```rust
use fantasy_map_generator::{Extents2d, GlibcRand, MapExportOptions, MapGenerator};
use fantasy_map_generator::presentation::satellite_svg::{SatelliteSvgLayers, SatelliteSvgOptions};
use fantasy_map_generator::satellite_svg::build_satellite_svg_from_data;

let extents = Extents2d::new(0.0, 0.0, 20.0, 10.0);
let rng = GlibcRand::new(12345);
let mut generator = MapGenerator::new(extents, 0.08, 1920, 1080, rng);

generator.initialize();
generator.add_hill(10.0, 5.0, 3.0, 1.2);
generator.normalize();

let map_data = generator.collect_draw_data_with_options(MapExportOptions {
    include_raster_data: true,
});

let svg = build_satellite_svg_from_data(
    &map_data,
    SatelliteSvgLayers::default(),
    SatelliteSvgOptions::default(),
)?;

std::fs::write("examples/output-satellite-from-data.svg", svg)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

### PNG 渲染

需要启用 `render` feature：

```rust
use fantasy_map_generator::render::render_map;

let json = std::fs::read_to_string("examples/output.json")?;
render_map(&json, "examples/output.png")?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

也可以直接使用 `MapRenderer`、`RenderStyle` 和 `Color` 做自定义样式渲染。

### 一个推荐的命令行导出流程

如果你更习惯先用 CLI 生成 JSON，再用库函数导出不同 SVG，可以按这个顺序：

```bash
cd rust

# 先生成带栅格数据的 JSON
cargo run --release -- --seed 12345 --output examples/export_demo

# 然后在你自己的 Rust 小工具或测试代码里读取：
# - examples/export_demo.json
# - build_map_svg(...)
# - build_satellite_svg(...)
```

其中：

- 标准 SVG 只依赖矢量层数据
- 卫星 SVG 依赖 `heightmap` 和 `land_mask`
- 如果你在导出 JSON 时手动关闭了栅格数据，卫星 SVG 会报缺少必要字段

## 按用途分类的导出示例

### 1. 出版或高质量归档

目标：

- 保留完整图层
- 使用标准 SVG 或高质量卫星 SVG
- 优先保证可缩放和输出质量

建议：

- 标准风格用 `build_map_svg(...)`
- 卫星风格用 `build_satellite_svg(...)`
- CLI 先生成完整 JSON，不要裁掉栅格字段

示例：

```bash
cd rust
cargo run --release -- --seed 424242 --size 3840:2160 --output examples/atlas_map
```

后续可导出：

- `examples/atlas_map.json`
- `examples/atlas_map-standard.svg`
- `examples/atlas_map-satellite.svg`

### 2. 网页预览或分享图

目标：

- 文件更小
- 浏览器加载更快
- 对细节损失不敏感

建议：

- 标准 SVG 只保留关键图层
- 卫星 SVG 使用较小 `max_embedded_image_size`
- 降低 `jpeg_quality`

示例配置：

```json
{
  "max_embedded_image_size": 512,
  "jpeg_quality": 60
}
```

适合：

- 文档预览
- 前端 demo 默认导出
- 即时分享

### 3. 只导出底图

目标：

- 不显示城市、城镇、标签
- 保留自然地形和边界主体

标准 SVG 图层示例：

```json
{
  "slope": true,
  "river": true,
  "contour": true,
  "border": true,
  "city": false,
  "town": false,
  "label": false
}
```

适合：

- 后续在其他软件中叠加标注
- 作为 UI/游戏底图素材
- 做二次设计

### 4. 只导出叠加层

目标：

- 只保留河流、边界、城市或标签等信息层
- 方便叠加到其他底图或材质上

一个常见配置是仅保留河流和边界：

```json
{
  "slope": false,
  "river": true,
  "contour": false,
  "border": true,
  "city": false,
  "town": false,
  "label": false
}
```

如果想做纯标注层，也可以这样：

```json
{
  "slope": false,
  "river": false,
  "contour": false,
  "border": false,
  "city": true,
  "town": true,
  "label": true
}
```

### 5. 为前端 Three.js / WebGPU 准备数据

目标：

- 不直接导出 SVG/PNG
- 导出给网页渲染器消费的结构化场景数据

建议：

- 浏览器端直接走 `WasmMapGenerator::generate_render_packet(...)`
- 如果是离线处理流程，则保留完整 JSON 和栅格数据

这条路径更适合：

- 交互式地图查看器
- 自定义 3D 地形展示
- Rust 生成、前端实时渲染的工作流

## 图层配置速查

标准 SVG 和卫星 SVG 当前都支持这组逻辑图层开关：

- `slope`
- `river`
- `contour`
- `border`
- `city`
- `town`
- `label`

经验上可以这样选：

- 想突出地形：开启 `slope`、`contour`
- 想突出行政和文明痕迹：开启 `border`、`city`、`town`、`label`
- 想做干净底图：关闭 `city`、`town`、`label`
- 想做 overlay：只保留 `river`、`border` 或 `label`

## WASM 与前端集成

启用 `wasm` feature 后，当前会导出这些浏览器侧能力：

- `WasmMapGenerator`
- `generate_map_simple`
- `build_map_svg`
- `build_satellite_svg`
- `build_satellite_svg_with_options`
- `presentation_plugin_metadata_json`

其中 `WasmMapGenerator` 除了返回 JSON，还支持：

- `generate_with_options(...)`
- `generate_terrain_only()`
- `generate_render_packet(...)`
- `set_draw_scale(...)`
- `get_seed()`

`generate_render_packet(...)` 会返回前端 `Three.js/WebGPU` 渲染所需的：

- 地形网格顶点、法线、UV、索引
- height / land mask / flux / albedo 等纹理数据
- slope / river / contour / border 的路径数据
- city / town / label 的展示数据

## Presentation 插件层

`src/presentation/` 是当前实现里新增的展示数据适配层，核心目的是把 `MapDrawData` 转成不同前端/渲染器更适合消费的结构化数据。

当前内置插件：

- `standard_svg`
- `webgpu_scene`
- `satellite_svg`，仅在 `svg` feature 下进入 metadata registry

可通过 `presentation_plugin_metadata()` 或 `presentation_plugin_metadata_json()` 获取插件能力元数据。

## 前端示例

`rust/examples/` 是当前配套的 Web 演示工程，详细说明见 `examples/README.md`。

常见流程：

```bash
cd rust
./build-wasm.sh

cd examples
pnpm install
pnpm dev
```

如果只改了前端代码，不改 Rust/WASM，通常不需要重新执行 `build-wasm.sh`。

## 测试与验证

当前建议的验证命令：

```bash
cd rust
cargo test
cargo test --features render
cargo check --features wasm
```

仓库里现有测试覆盖了：

- CLI 输出 JSON 基本结构
- `presentation` metadata
- `render` 配置与错误类型
- 文档示例编译

## 当前限制

- `render` 模块的文字渲染仍是占位实现，不是完整字体排版系统
- `RenderStyle` 中的线宽字段已保留，但当前渲染实现还没有把所有线宽配置完整映射到底层线条光栅化
- Web 侧主要通过 `wasm-pack` 产物给 `examples/` 使用，不是单独发布到 npm 的通用包结构
- 一些兼容性参数已保留在 CLI 中，但目前没有额外逻辑，例如 `--drawing-supported`

## 参考

- Martin O'Leary: <https://mewo2.com/notes/terrain/>
- WebGPU 规范: <https://www.w3.org/TR/webgpu/>
- WGSL 规范: <https://www.w3.org/TR/WGSL/>
- `wgpu` 文档: <https://docs.rs/wgpu/>
- Rust API Guidelines: <https://rust-lang.github.io/api-guidelines/>
- 原始 C++ 项目源码：`../src/`
- 原始 Python 渲染参考：`../src/render/rendermap.py`
- 当前 Rust 核心生成实现：`src/map_generator.rs`
- 当前 Rust PNG 渲染实现：`src/render/`
- 当前 Rust presentation 插件层：`src/presentation/`
- 当前 Rust WASM 导出层：`src/wasm.rs`
