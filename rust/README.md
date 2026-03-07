# Fantasy Map Generator - Rust Implementation

这是 Fantasy Map Generator 的 Rust 实现版本，采用了 Rust 社区最佳实践的项目结构。

## 快速开始

### 构建项目

```bash
cd rust

# 基础构建（仅生成 JSON 数据）
cargo build --release

# 启用 WebGPU 渲染功能
cargo build --release --features render
```

### 运行程序

```bash
# 查看帮助
cargo run --release -- --help

# 生成地图 JSON 数据
cargo run --release -- --seed 12345 --output examples/my_map

# 生成地图并渲染为 PNG（需要 render feature）
cargo run --release --features render -- --seed 12345 --output examples/my_map

# 自定义参数
cargo run --release --features render -- --seed 12345 --cities 5 --towns 10 --size 1920:1080
```

### 常见游戏地图尺寸示例

以下是一些常用的地图尺寸：

```bash
# 16:9 标准尺寸
cargo run --release --features render -- --seed 12345 --size 1920:1080   # Full HD
cargo run --release --features render -- --seed 12345 --size 2560:1440   # 2K
cargo run --release --features render -- --seed 12345 --size 3840:2160   # 4K

# 21:9 超宽屏（适合策略游戏）
cargo run --release --features render -- --seed 12345 --size 2560:1080   # UW-FHD
cargo run --release --features render -- --seed 12345 --size 3440:1440   # UW-QHD
cargo run --release --features render -- --seed 12345 --size 5120:2160   # UW-5K

# 4:3 经典比例（老式策略游戏）
cargo run --release --features render -- --seed 12345 --size 1024:768
cargo run --release --features render -- --seed 12345 --size 1600:1200

# 16:10 比例（部分笔记本和显示器）
cargo run --release --features render -- --seed 12345 --size 1920:1200
cargo run --release --features render -- --seed 12345 --size 2560:1600

# 正方形地图（适合回合制策略游戏）
cargo run --release --features render -- --seed 12345 --size 2048:2048
cargo run --release --features render -- --seed 12345 --size 4096:4096

# 移动端尺寸
cargo run --release --features render -- --seed 12345 --size 1080:1920   # 竖屏
cargo run --release --features render -- --seed 12345 --size 2340:1080   # 横屏（18:9）
```

**注意**：所有示例输出都保存在 `examples/` 目录下，避免污染项目根目录。

### 运行测试

```bash
cargo test
```

### 作为库使用

在其他 Rust 项目的 `Cargo.toml` 中添加：

```toml
[dependencies]
fantasy_map_generator = { path = "../FantasyMapGenerator/rust" }
```

在代码中使用：

```rust
use fantasy_map_generator::{Config, MapGenerator, Extents2d, GlibcRand};

fn main() {
    let extents = Extents2d::new(0.0, 0.0, 20.0, 10.0);
    let rng = GlibcRand::new(12345);
    let mut map = MapGenerator::new(extents, 0.08, 1920, 1080, rng);
    
    map.initialize();
    // 使用地图生成器...
}
```

## 项目结构

```
rust/
├── src/
│   ├── lib.rs                    # 库入口，导出公共 API
│   ├── main.rs                   # 二进制程序入口
│   ├── cli.rs                    # CLI 应用逻辑
│   ├── config.rs                 # 配置和命令行参数
│   ├── map_generator.rs          # 核心地图生成器
│   │
│   ├── algorithms/               # 几何算法模块
│   │   ├── mod.rs
│   │   ├── delaunay.rs          # Delaunay 三角剖分
│   │   ├── poisson_disc.rs      # Poisson 圆盘采样
│   │   └── voronoi.rs           # Voronoi 图生成
│   │
│   ├── data_structures/          # 核心数据结构
│   │   ├── mod.rs
│   │   ├── dcel.rs              # 双连边表 (DCEL)
│   │   ├── extents2d.rs         # 2D 边界框
│   │   ├── geometry.rs          # 几何工具函数
│   │   ├── node_map.rs          # 节点映射
│   │   ├── spatial_point_grid.rs # 空间点网格
│   │   └── vertex_map.rs        # 顶点映射
│   │
│   ├── utils/                    # 工具模块
│   │   ├── mod.rs
│   │   ├── font_face.rs         # 字体处理
│   │   └── rand.rs              # 随机数生成器
│   │
│   ├── render/                   # 渲染模块（WebGPU）
│   │   ├── mod.rs               # 模块入口
│   │   ├── config.rs            # 渲染配置
│   │   ├── error.rs             # 错误类型
│   │   ├── renderer.rs          # 核心渲染器
│   │   ├── primitives.rs        # 顶点结构
│   │   ├── text.rs              # 文字渲染（占位）
│   │   ├── tests.rs             # 单元测试
│   │   └── shaders/
│   │       ├── line.wgsl        # 线条着色器
│   │       └── circle.wgsl      # 圆形着色器
│   │
│   ├── citydata/                 # 城市数据
│   │   └── countrycities.json
│   │
│   └── fontdata/                 # 字体数据
│       └── fontdata.json
│
├── tests/                        # 集成测试
│   ├── snapshot_test.rs
│   └── baselines/
│       └── baseline_output.json
│
├── examples/                     # 示例输出目录
│
├── Cargo.toml                    # 项目配置
├── Cargo.lock                    # 依赖锁定文件
├── README.md                     # 本文档
└── COMMENTING_GUIDE.md           # 注释规范
```

## WebGPU 渲染

本项目实现了基于 WebGPU 的地图渲染功能，可以将生成的地图数据渲染为 PNG 图像。

### 功能特性

- ✅ GPU 加速渲染
- ✅ 跨平台支持（Windows、Linux、macOS）
- ✅ 无 Python 依赖
- ✅ 支持所有地图元素（坡度、河流、等高线、边界、城市、城镇）
- ✅ 自定义错误类型和精确错误信息
- ✅ 灵活的 API 设计（支持自定义样式）
- ✅ 完善的单元测试
- ⚠️ 文字渲染暂未实现
- ⚠️ 虚线边界暂未实现
- ⚠️ 线宽控制暂未实现

### 使用方法

#### 方式 1：命令行工具

```bash
# 生成地图并渲染为 PNG
cargo run --release --features render -- --seed 123456 --output examples/my_map

# 仅生成 JSON，不渲染 PNG
cargo run --release --features render -- --seed 123456 --no-render --output examples/my_map
```

#### 方式 2：作为库使用（便捷函数）

```rust
use fantasy_map_generator::render::render_map;

let json_data = std::fs::read_to_string("examples/map_data.json")?;
render_map(&json_data, "examples/output.png")?;
```

#### 方式 3：作为库使用（渲染器）

```rust
use fantasy_map_generator::render::MapRenderer;
use serde_json::json;

let data = json!({
    "image_width": 1920,
    "image_height": 1080,
    "draw_scale": 1.0,
    // ... 其他数据
});

let mut renderer = MapRenderer::new(1920, 1080)?;
renderer.render(&data)?;
renderer.save_png("examples/output.png")?;
```

#### 方式 4：自定义样式

```rust
use fantasy_map_generator::render::{MapRenderer, RenderStyle, Color};

let mut style = RenderStyle::default();
style.river_color = Color::new(0.0, 0.0, 1.0, 1.0); // 蓝色河流
style.river_line_width = 3.0;

let mut renderer = MapRenderer::with_style(1920, 1080, style)?;
renderer.render(&data)?;
renderer.save_png("examples/custom_style.png")?;
```

### 渲染 API 参考

#### MapRenderer

```rust
// 构造函数
pub fn new(width: u32, height: u32) -> RenderResult<Self>
pub fn with_style(width: u32, height: u32, style: RenderStyle) -> RenderResult<Self>

// 方法
pub fn render(&mut self, data: &Value) -> RenderResult<()>
pub fn save_png(&self, path: &str) -> RenderResult<()>
```

#### RenderStyle

```rust
pub struct RenderStyle {
    // 颜色
    pub background_color: Color,
    pub slope_color: Color,
    pub river_color: Color,
    pub contour_color: Color,
    pub border_color: Color,
    pub marker_color: Color,
    pub text_color: Color,
    
    // 线宽（像素）
    pub slope_line_width: f32,
    pub river_line_width: f32,
    pub contour_line_width: f32,
    pub border_line_width: f32,
    
    // 标记半径（像素）
    pub city_marker_outer_radius: f32,
    pub city_marker_inner_radius: f32,
    pub town_marker_radius: f32,
}

// 方法
pub fn default() -> Self
pub fn with_scale(&self, scale: f32) -> Self
```

#### Color

```rust
pub struct Color { pub r: f32, pub g: f32, pub b: f32, pub a: f32 }

// 预定义颜色
pub const WHITE: Self
pub const BLACK: Self
pub const BLACK_TRANSPARENT: Self

// 方法
pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self
pub const fn to_array(self) -> [f32; 4]
```

#### RenderError

```rust
pub enum RenderError {
    AdapterNotFound,                    // 未找到 WebGPU 适配器
    DeviceCreationFailed(String),       // 设备创建失败
    BufferMapFailed,                    // 缓冲区映射失败
    ImageCreationFailed,                // 图像创建失败
    ImageSaveFailed(std::io::Error),    // 图像保存失败
    JsonParseFailed(serde_json::Error), // JSON 解析失败
    InvalidJsonFormat(String),          // 无效的 JSON 格式
}
```

### 实现细节

#### 渲染流程

1. 清空背景为白色
2. 绘制坡度阴影（半透明黑色线段）
3. 绘制领土边界（先白色底，再黑色线）
4. 绘制河流（黑色路径）
5. 绘制等高线（黑色路径）
6. 绘制城市标记（双圆环：外黑内白）
7. 绘制城镇标记（单黑圆）
8. 绘制文字标签（占位实现）

#### 坐标系统

- **输入坐标**: JSON 数据中的坐标为归一化坐标 [0, 1]
- **Y 轴翻转**: `y_ndc = 1.0 - y_normalized`（WebGPU 的 Y 轴向上）
- **NDC 转换**: 着色器中将 [0, 1] 转换为 [-1, 1]

#### 技术特点

- **零拷贝传输**: 使用 `bytemuck` 实现顶点数据零拷贝传输到 GPU
- **自定义错误类型**: 提供精确的错误信息，便于调试
- **灵活的 API**: 支持默认样式和自定义样式
- **单元测试**: 11 个单元测试，覆盖关键功能

### 已知限制

1. **文字渲染**: 占位实现，不渲染文字标签
   - 需要字体文件和复杂的文字光栅化管线
   
2. **虚线边界**: 当前为实线
   - WebGPU 不直接支持虚线模式
   - 可通过 CPU 端预处理或着色器实现
   
3. **线宽控制**: 所有线条为 1 像素宽
   - WebGPU 的 LineList 拓扑不支持线宽设置
   - 可通过几何着色器扩展或三角形带实现

### 故障排除

#### 未找到 WebGPU 适配器

**错误**: `RenderError::AdapterNotFound`

**解决方案**:
1. 确保系统支持 WebGPU
2. 更新显卡驱动
3. 在 Windows 上，确保安装了最新的 DirectX

#### JSON 解析失败

**错误**: `RenderError::JsonParseFailed` 或 `RenderError::InvalidJsonFormat`

**解决方案**:
1. 验证 JSON 格式
2. 确保所有必需字段存在
3. 检查数据类型是否正确

## 命令行选项

```
Options:
  -s, --seed <SEED>                      Random seed [default: 0]
      --timeseed                         Use current time as seed
  -o, --output <OUTPUT>                  Output file (without extension) [default: output]
  -r, --resolution <RESOLUTION>          Resolution (poisson disc sampling distance) [default: 0.08]
  -e, --erosion-amount <EROSION_AMOUNT>  Erosion amount (-1 for random) [default: -1.0]
      --erosion-steps <EROSION_STEPS>    Erosion iterations [default: 3]
  -c, --cities <CITIES>                  Number of cities (-1 for random) [default: -1]
  -t, --towns <TOWNS>                    Number of towns (-1 for random) [default: -1]
      --size <SIZE>                      Image size (e.g. "1920:1080") [default: 1920:1080]
      --draw-scale <DRAW_SCALE>          Draw scale [default: 1.0]
      --no-slopes                        Disable slopes
      --no-rivers                        Disable rivers
      --no-contour                       Disable contour
      --no-borders                       Disable territory borders
      --no-cities                        Disable cities
      --no-towns                         Disable towns
      --no-labels                        Disable labels
      --no-arealabels                    Disable area labels
      --no-render                        Disable PNG rendering (requires render feature)
  -v, --verbose                          Enable verbose output
  -h, --help                             Print help
```

## 架构设计

### 模块组织

项目按照功能领域组织成清晰的模块层次：

**algorithms/** - 几何算法实现
- Delaunay 三角剖分
- Voronoi 图生成
- Poisson 圆盘采样

**data_structures/** - 核心数据结构
- DCEL (双连边表) 用于表示平面细分
- 几何基础类型和工具函数
- 空间数据结构用于高效查询

**utils/** - 通用工具
- 随机数生成器 (兼容 C++ glibc 实现)
- 字体处理工具

**render/** - WebGPU 渲染模块
- 渲染器核心实现
- 配置和错误类型
- 顶点结构和着色器

**核心模块**
- `map_generator.rs` - 主要的地图生成逻辑
- `config.rs` - 配置管理
- `cli.rs` - CLI 应用逻辑

### 库与二进制分离

项目同时提供库和可执行文件：

- **库 (lib.rs)**: 导出所有核心功能，可被其他 Rust 项目使用
- **二进制 (main.rs)**: 提供命令行工具，调用库功能

这种设计使得：
- 核心逻辑可以被重用
- 易于编写单元测试和集成测试
- 可以构建不同的前端 (CLI, GUI, Web 等)

## 最佳实践

本项目遵循以下 Rust 社区最佳实践：

### 代码组织
- ✅ 模块化设计，按功能领域组织代码
- ✅ 库与二进制分离
- ✅ 清晰的依赖关系
- ✅ 使用 `mod.rs` 和 re-exports 管理模块可见性

### 错误处理
- ✅ 使用自定义错误类型 (`RenderError`)
- ✅ 实现 `std::error::Error` trait
- ✅ 提供 `From` 转换
- ✅ 使用 `Result` 类型别名

### API 设计
- ✅ 提供多种构造方式 (`new`, `with_style`)
- ✅ 使用 Builder 模式思想
- ✅ 提供便捷函数
- ✅ 遵循命名约定

### 类型系统
- ✅ 使用 newtype 模式 (`Color`)
- ✅ 实现 trait (`From`, `Into`, `Display`)
- ✅ 类型安全的配置
- ✅ 充分利用 Rust 的类型系统

### 文档
- ✅ 模块级文档
- ✅ 函数文档
- ✅ 使用示例
- ✅ 错误说明

### 测试
- ✅ 单元测试（11 个测试，全部通过）
- ✅ 集成测试
- ✅ 测试覆盖关键功能
- ✅ 测试错误处理

### 性能
- ✅ 使用 `with_capacity` 预分配内存
- ✅ 避免不必要的克隆
- ✅ 使用引用传递
- ✅ 零拷贝（bytemuck）

### 代码注释

项目代码包含详细的中文注释，解释：
- 算法原理和实现步骤
- 数据结构的设计意图
- 与原始论文和 C++ 实现的对应关系
- 复杂逻辑的"为什么"而非"做什么"

注释遵循统一的规范，详见 `COMMENTING_GUIDE.md`。

## 与 C++ 版本的差异对比

### 核心算法兼容性

Rust 版本保持了与 C++ 版本的核心算法兼容性：
- 相同的随机数生成器实现 (glibc rand)
- 相同的 Poisson 圆盘采样算法
- 相同的 Delaunay 三角剖分算法
- 相同的 Voronoi 图生成算法
- 相同的地形生成算法（山丘、圆锥、斜坡）
- 相同的侵蚀算法（Planchon-Darboux）
- 相同的流向和流量计算
- 相同的城市和领土生成逻辑
- 相同的 JSON 输出格式

在给定相同种子时，两个版本生成的地图数据（JSON）是一致的。

### 渲染实现差异

| 特性 | C++ 版本 | Rust 版本 |
|------|---------|----------|
| 渲染引擎 | Python + Cairo | WebGPU (wgpu-rs) |
| 外部依赖 | Python + Pycairo | 无 |
| 渲染方式 | CPU | GPU |
| 文字渲染 | 完整支持 | 占位实现 |
| 虚线边界 | 支持 | 实线 |
| 线宽控制 | 支持 | 固定 1px |
| 样式自定义 | 固定 | API 可配置 |

### 项目结构差异

#### C++ 版本
```
src/
├── main.cpp                 # 单一入口
├── mapgenerator.h/cpp       # 单一大类
├── render.h/cpp             # Python 绑定
├── dcel.h/cpp               # 数据结构
├── delaunay.h/cpp           # 算法
└── ...                      # 其他文件
```

#### Rust 版本
```
rust/src/
├── lib.rs                   # 库入口
├── main.rs                  # 二进制入口
├── cli.rs                   # CLI 逻辑
├── algorithms/              # 算法模块
│   ├── delaunay.rs
│   ├── voronoi.rs
│   └── poisson_disc.rs
├── data_structures/         # 数据结构模块
│   ├── dcel.rs
│   ├── geometry.rs
│   └── ...
├── render/                  # 渲染模块
│   ├── renderer.rs
│   ├── primitives.rs
│   └── shaders/
└── utils/                   # 工具模块
```

### 构建系统差异

| 方面 | C++ 版本 | Rust 版本 |
|------|---------|----------|
| 构建工具 | CMake | Cargo |
| 依赖管理 | 手动 | 自动 |
| 配置文件 | CMakeLists.txt | Cargo.toml |
| 特性开关 | 编译时宏 | Cargo features |
| 测试框架 | 无 | cargo test |

### API 设计差异

#### C++ 版本
```cpp
// 构造
MapGenerator map(extents, resolution, width, height);

// 渲染
render::drawMap(drawdata, filename);
```

#### Rust 版本
```rust
// 构造
let renderer = MapRenderer::new(width, height)?;
let renderer = MapRenderer::with_style(width, height, style)?;

// 渲染
renderer.render(&data)?;
renderer.save_png("output.png")?;

// 便捷函数
render_map(&json_data, "output.png")?;
```

### 性能对比

| 操作 | C++ 版本 | Rust 版本 |
|------|---------|----------|
| 地图生成 | ~2-5 秒 | ~2-5 秒 |
| 渲染 | ~1-3 秒 | ~0.1-0.5 秒 |

### 部署差异

| 方面 | C++ 版本 | Rust 版本 |
|------|---------|----------|
| 编译环境 | CMake + C++ 编译器 | Cargo |
| 运行时依赖 | Python + Pycairo | 无 |
| 分发形式 | 可执行文件 + Python 脚本 | 单一可执行文件 |

### 功能完整度对比

| 功能 | C++ 版本 | Rust 版本 |
|------|---------|----------|
| 地图生成 | ✅ | ✅ |
| JSON 输出 | ✅ | ✅ |
| PNG 渲染 | ✅ | ✅ |
| 文字标签 | ✅ | ⚠️ 占位 |
| 虚线边界 | ✅ | ⚠️ 实线 |
| 线宽控制 | ✅ | ⚠️ 固定 |
| 样式配置 | ❌ | ✅ |
| 库 API | ❌ | ✅ |
| 单元测试 | ❌ | ✅ |

## 性能特点

在典型配置下（1920x1080，resolution=0.08）：
- 地图生成: ~2-5 秒
- WebGPU 渲染: ~100-500 毫秒
- PNG 保存: ~50-200 毫秒

## 贡献指南

欢迎贡献代码改进项目！重点领域：
- 文字渲染实现
- 虚线边界支持
- 线宽控制
- 性能优化

请确保：
- 代码包含中文注释
- 遵循现有代码风格
- 添加测试用例
- 更新文档
- 输出文件保存在 `examples/` 目录

## 参考资料

- [WebGPU 规范](https://www.w3.org/TR/webgpu/)
- [wgpu-rs 文档](https://docs.rs/wgpu/)
- [WGSL 着色器语言](https://www.w3.org/TR/WGSL/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- 原始 C++ 实现: `../src/`
- 原始 Python 渲染: `../src/render/rendermap.py`
