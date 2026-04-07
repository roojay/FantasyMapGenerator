# Fantasy Map Generator Rust 版

这个目录是项目的 Rust 实现，当前同时提供 4 条实际可用路径：

- CLI：生成地图 JSON，启用 `render` feature 时可额外输出 PNG
- Rust 库：直接调用 `MapGenerator`、`MapDrawData` 和相关导出结构
- WASM 绑定：供浏览器端生成地图、构建标准 SVG、导出 WebGPU scene packet
- `examples/`：基于 Vite/React/Three.js 的网页演示

## 当前目录结构

```text
rust/
├── Cargo.toml
├── README.md
├── build-wasm.sh
├── build-wasm.bat
├── src/
│   ├── cli.rs
│   ├── config.rs
│   ├── lib.rs
│   ├── main.rs
│   ├── map_generator.rs
│   ├── standard_svg.rs
│   ├── wasm.rs
│   ├── algorithms/
│   ├── data_structures/
│   ├── presentation/
│   ├── render/
│   └── utils/
├── tests/
└── examples/
```

## 已实现能力

- 不规则网格生成：Poisson disk + Delaunay + Voronoi
- 地形生成：hill / cone / slope 原语
- 多轮侵蚀、河流、等高线、坡线
- 城市、城镇、领土、标签布局
- 标准 SVG 导出
- WASM `generate_render_packet(...)` 导出前端场景包
- 可选 PNG 渲染（`render` feature）

当前**没有**这些能力：

- `satellite_svg`
- 卫星风格 SVG CLI 选项
- 独立 npm 包发布流程

## 当前推荐工作流

按实际用途，建议这样使用：

- 批处理出图或归档：CLI
- 程序内复用生成结果：Rust 库
- 浏览器实时生成与交互：WASM + `examples/`
- 标准矢量导出：`build_map_svg(...)`

如果目标是稳定产出素材，推荐顺序通常是：

1. 先生成 JSON
2. 需要位图时启用 `render`
3. 需要矢量图时启用 `svg`

## 快速开始

### 1. 仅生成 JSON

```bash
cd rust
cargo run --release -- --seed 12345 --output examples/my_map
```

输出：

- `examples/my_map.json`

### 2. 生成 JSON + PNG

```bash
cd rust
cargo run --release --features render -- --seed 12345 --output examples/my_map
```

输出：

- `examples/my_map.json`
- `examples/my_map.png`

如果只想在启用 `render` 后跳过 PNG：

```bash
cargo run --release --features render -- --seed 12345 --no-render --output examples/my_map
```

### 3. 生成 JSON + 标准 SVG

```bash
cd rust
cargo run --release --features svg -- --seed 12345 --svg --output examples/my_map
```

输出：

- `examples/my_map.json`
- `examples/my_map-standard.svg`

### 4. 查看 CLI 参数

```bash
cd rust
cargo run -- --help
```

当前实际支持的常用参数：

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
- `--no-render`，仅在 `render` feature 下编译可用
- `--svg`，仅在 `svg` feature 下编译可用

当前会被解析但没有额外行为的参数：

- `--drawing-supported`
- `-v` / `--verbose`

## 地图生成流程

当前 CLI、库和 WASM 共用同一套核心逻辑，顺序如下：

1. Poisson 采样生成点集
2. Delaunay 三角剖分并构建 Voronoi 网格
3. 随机叠加 hill / cone / slope 生成初始地形
4. 归一化、圆滑或松弛
5. 多轮侵蚀，期间计算流向和流量
6. 生成河流、等高线、坡线
7. 放置城市、城镇并计算领土边界
8. 生成并优化标签布局
9. 导出 `MapDrawData` 或 Web 场景包

当前实现已经包含这些与性能相关的落地优化：

- `calculate_flux_map()` 使用线性传播，而不是重复沿流向路径累加
- hill / cone 原语使用局部空间查询，而不是每个原语全图扫描
- Web 首图默认不保留完整 JSON，JSON 导出改为按需生成

## 算法说明

当前实现里最重要的算法和职责如下：

- Poisson 圆盘采样：生成分布均匀的基础点集，避免规则网格痕迹
- Delaunay 三角剖分：建立点之间的稳定邻接关系
- Voronoi 图：作为地图的不规则网格和面级分析基础
- 地形原语：通过 hill / cone / slope 组合出初始地貌
- 洼地填充：保证水流路径可达，不会被局部低洼困住
- 流向 / 流量计算：驱动河流与侵蚀
- 侵蚀：逐轮平滑地形、刻蚀河谷
- 城市评分与移动成本：决定城市位置和领土划分
- 模拟退火：在候选位置中优化标签布局

与当前实现直接相关的几处细节：

- `calculate_flux_map()` 已不是旧版“每个点一路累加到边界”，而是按流向 DAG 做线性传播
- `fill_depressions()` 当前采用 priority-flood 风格传播，而不是全图反复扫描
- hill / cone 使用 `SpatialPointGrid` 做局部顶点筛选，减少无意义全量遍历
- 前端首图不再强制保留完整 JSON，JSON 导出改为按需回源

## Cargo features

`Cargo.toml` 当前定义了 3 个可选 feature：

### `render`

启用本地 PNG 渲染。

依赖：

- `wgpu`
- `pollster`
- `image`
- `bytemuck`

### `svg`

启用标准 SVG 导出。

当前只包含：

- `standard_svg`

### `wasm`

启用浏览器集成。

依赖：

- `wasm-bindgen`
- `js-sys`
- `console_error_panic_hook`
- `wee_alloc`
- `svg`

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

### 标准 SVG 导出

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

### PNG 渲染

```rust
use fantasy_map_generator::render::render_map;

let json = std::fs::read_to_string("examples/output.json")?;
render_map(&json, "examples/output.png")?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## 常见导出场景

### 1. 只要一份 JSON

适合：

- 调试生成逻辑
- 后续自己写渲染器
- 保存可复现输入

```bash
cargo run --release -- --seed 12345 --output examples/map_only
```

### 2. JSON + PNG

适合：

- 离线渲染
- 生成预览图或归档图

```bash
cargo run --release --features render -- --seed 12345 --output examples/map_png
```

### 3. JSON + 标准 SVG

适合：

- 矢量编辑
- 排版、出版或二次设计

```bash
cargo run --release --features svg -- --seed 12345 --svg --output examples/map_svg
```

### 4. Web 实时生成

适合：

- 前端实时换种子
- WebGPU / SVG 双渲染
- 用户交互式预览

```bash
cd rust
./build-wasm.sh
cd examples
pnpm dev
```

## 更多命令示例

### 1. 使用时间种子生成一张随机图

```bash
cd rust
cargo run --release -- --timeseed --output examples/random_map
```

### 2. 指定更高分辨率和更大尺寸

```bash
cd rust
cargo run --release -- --seed 424242 --resolution 0.06 --size 2560:1440 --output examples/large_map
```

### 3. 关闭部分图层生成更干净的底图

```bash
cd rust
cargo run --release -- --seed 12345 --no-cities --no-towns --no-labels --output examples/base_map
```

### 4. 只导出标准 SVG，不渲染 PNG

```bash
cd rust
cargo run --release --features "render svg" -- --seed 12345 --svg --no-render --output examples/vector_map
```

### 5. 前端开发前先重建 WASM

```bash
cd rust
./build-wasm.sh
cd examples
pnpm dev
```

### 6. 前端生产构建

```bash
cd rust/examples
pnpm build
pnpm preview
```

## JSON 导出格式

CLI 默认导出的核心结构是 `MapDrawData`，主要字段包括：

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

当启用栅格导出时，还会包含：

- `heightmap`
- `flux_map`
- `land_mask`
- `land_polygons`

说明：

- CLI 默认走 `get_draw_data()`，会带这些栅格字段
- Web 前端首图生成默认不保留完整 JSON，而是在用户首次导出 JSON 时按需生成并缓存
- 标准 SVG 只依赖矢量层数据

## WASM 导出接口

启用 `wasm` feature 后，当前对浏览器暴露这些接口：

- `WasmMapGenerator`
- `generate_map_simple`
- `build_map_svg`
- `presentation_plugin_metadata_json`

`WasmMapGenerator` 当前提供：

- `generate(...)`
- `generate_with_options(...)`
- `generate_terrain_only()`
- `generate_render_packet(...)`
- `set_draw_scale(...)`
- `get_seed()`

`generate_render_packet(...)` 当前返回：

- `metadata_json`
- `svg_json`
- 地形网格：positions / normals / uvs / indices
- 纹理：`height` / `land_mask` / `flux` / `terrain_albedo` / `roughness` / `ao` / `water_*`
- 图层路径：`slope` / `river` / `contour` / `border`
- 标记与标签数据

当前**不再**通过 render packet 返回完整 `map_json` 或单独 `albedo_texture`。

这意味着：

- CLI 仍然是完整 JSON 的稳定出口
- 前端首图不会为“未来可能导出 JSON”提前付序列化成本
- 用户第一次点击 JSON 导出时，前端才会回源生成完整 JSON 并缓存

## 渲染流程

### Rust `render` 模块 PNG 渲染顺序

当前 `render` feature 下的 PNG 渲染顺序与 [renderer.rs](./src/render/renderer.rs) 一致：

1. 清空背景
2. 绘制坡度阴影
3. 绘制领土边界底色和边界线
4. 绘制河流
5. 绘制等高线
6. 绘制城市标记
7. 绘制城镇标记
8. 绘制文字标签

说明：

- 第 8 步的文字渲染接口已接好，但底层仍不是完整排版系统
- 领土边界当前是“白底 + 黑线”两层渲染

### Web 前端渲染顺序

当前网页示例有两条渲染路径：

- `SVG`：后台 worker 生成标准 SVG，主线程只负责挂载与视口变换
- `WebGPU`：加载 render packet，构建地形、路径、标记和标签

当前 WebGPU 场景大致顺序是：

1. 背景平面
2. 灯光
3. 地形网格
4. 水面覆盖层
5. 坡线 / 河流 / 海岸线 / 边界
6. 城市与城镇标记
7. 标签

图层切换时：

- WebGPU 路径优先走显隐更新，不重建整棵场景
- SVG 路径只失效 SVG 缓存并按需重建

## 坐标系统

当前项目涉及 3 套主要坐标表示：

### 1. `MapDrawData` 归一化坐标

Rust 核心生成层导出的矢量坐标通常在 `[0, 1]` 范围内：

- `x = 0` 表示地图左侧
- `x = 1` 表示地图右侧
- `y = 0` 表示地图底部
- `y = 1` 表示地图顶部

这层坐标最适合跨后端复用。

### 2. 前端场景世界坐标

在前端 `mapScenePacket` 转换阶段：

- `x` 被映射到 `(-0.5 .. 0.5) * imageWidth`
- `z` 被映射到 `(-0.5 .. 0.5) * imageHeight`
- `y` 作为海拔高度或覆盖层高度

同时，采样高度时会使用 `1 - normalizedY`，把生成层的“底部为 0”转换成纹理 / 屏幕习惯的“顶部为 0”。

### 3. Three.js 坐标适配

当前前端渲染器会通过 `packetToThreeTriplets(...)` 做一次轴重排：

- 生成包里的 `(x, y, z)`
- 转成 Three.js 使用的 `(x, z, y)`

所以：

- 地图平面最终落在 `X/Z` 平面上
- 海拔抬升落在 Three.js 的 `Y` 方向

### SVG 坐标

标准 SVG 导出直接把归一化坐标映射成像素坐标：

- `x = nx * width`
- `y = height - ny * height`

也就是：

- Rust 生成层仍以“底部为 0”
- SVG 输出时翻转成浏览器 SVG 常见的“顶部为 0”

## Presentation 插件层

`src/presentation/` 当前内置的插件只有：

- `standard_svg`
- `webgpu_scene`

可以通过：

- `presentation_plugin_metadata()`
- `presentation_plugin_metadata_json()`

读取插件元数据。

## 前端示例

`rust/examples/` 是配套网页演示，详见 `examples/README.md`。

典型流程：

```bash
cd rust
./build-wasm.sh

cd examples
pnpm install
pnpm dev
```

如果只改了 `examples/src/` 下的前端代码，通常不需要重建 WASM。

## 测试与验证

建议的基本验证命令：

```bash
cd rust
cargo test
cargo test --features render
cargo check --features wasm
```

前端示例建议额外执行：

```bash
cd rust/examples
pnpm build
```

## 构建脚本

- `build-wasm.sh`
- `build-wasm.bat`

两者当前都执行同一条构建命令：

```bash
wasm-pack build --target web --out-dir examples/pkg --features wasm
```

脚本只检查 `wasm-pack` 是否存在，不会自动安装依赖。

## 当前限制

- `render` 模块中的文字渲染仍然不是完整排版系统
- `three.webgpu` 运行时代码体积仍较大，但已在前端构建中拆出主包
- `--drawing-supported` 和 `--verbose` 仍是保留参数

## 排障

### `wasm-pack not found`

先安装：

```bash
cargo install wasm-pack
```

### 启用 `render` 后没有 PNG

检查：

- 是否编译时启用了 `render`
- 是否传了 `--no-render`

### 修改 Rust 代码后前端效果没变

通常是没有重新执行：

```bash
cd rust
./build-wasm.sh
```

### Windows 下 `build-wasm.sh` 不能直接运行

Windows 当前推荐直接执行：

```bat
build-wasm.bat
```

## 参考

- Martin O'Leary: <https://mewo2.com/notes/terrain/>
- `wgpu`: <https://docs.rs/wgpu/>
- Rust and WebAssembly Book: <https://rustwasm.github.io/book/>
