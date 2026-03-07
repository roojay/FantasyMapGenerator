# Rust 代码注释指南

本文档说明如何为 Fantasy Map Generator 的 Rust 实现添加详细的中文注释。

## 注释规范

### 1. 文档注释 (Doc Comments)

使用 `///` 为公共 API 添加文档注释，使用 `//!` 为模块添加文档注释。

#### 模块级注释示例

```rust
//! Poisson 圆盘采样算法
//!
//! 生成一组随机点，保证任意两点之间的距离不小于指定的最小距离。
//! 这种采样方法产生的点分布比纯随机采样更均匀。
//!
//! # 算法原理
//! 使用 Robert Bridson 的快速 Poisson 圆盘采样算法。
//! 算法通过维护一个活跃点列表和一个背景网格来加速邻域查询。
//!
//! # 参考来源
//! - R. Bridson, "Fast Poisson Disk Sampling in Arbitrary Dimensions", 
//!   ACM SIGGRAPH 2007 Sketches Program, 2007.
//! - M. O'Leary, "Generating fantasy maps", https://mewo2.com/notes/terrain/
//! - 原始 C++ 实现: src/poissondiscsampler.h, src/poissondiscsampler.cpp
```

#### 函数注释示例

```rust
/// 生成 Poisson 圆盘采样点
///
/// 在指定的矩形区域内生成一组随机点，保证任意两点之间的距离
/// 不小于 `min_distance`。
///
/// # 算法步骤
/// 1. 初始化背景网格，用于快速邻域查询
/// 2. 选择一个随机起始点
/// 3. 从活跃点列表中随机选择点，尝试在其周围生成新点
/// 4. 对每个候选点，检查是否与现有点的距离满足要求
/// 5. 重复直到无法生成新点
///
/// # 参数
/// * `rng` - 随机数生成器
/// * `extents` - 采样区域的边界
/// * `min_distance` - 点之间的最小距离
/// * `k` - 每个点周围尝试生成新点的次数（通常为 30）
///
/// # 返回
/// 生成的点的集合
///
/// # 参考来源
/// - R. Bridson, "Fast Poisson Disk Sampling", SIGGRAPH 2007
pub fn generate_samples(
    rng: &mut GlibcRand,
    extents: Extents2d,
    min_distance: f64,
    k: usize,
) -> Vec<Point> {
    // 实现...
}
```

### 2. 常规注释 (Regular Comments)

使用 `//` 解释复杂逻辑、设计决策或临时方案。

#### 算法解释

```rust
// ===================================
// 1. 初始化背景网格
// ===================================
// 将采样区域划分为网格，每个网格单元的边长为 min_distance/√2
// 这样可以保证每个单元最多包含一个采样点
let cell_size = min_distance / 2.0_f64.sqrt();
let grid_width = ((extents.maxx - extents.minx) / cell_size).ceil() as usize + 1;
let grid_height = ((extents.maxy - extents.miny) / cell_size).ceil() as usize + 1;
```

#### 魔法数字解释

```rust
// 使用 tanh 函数将值平滑地限制在 [-1, 1] 区间
// 这样可以防止极端值导致的数值不稳定
let normalized = (value / 50.0).tanh();

// k = 30 是 Bridson 论文中推荐的值
// 经验表明这个值在质量和性能之间取得了良好的平衡
const DEFAULT_K: usize = 30;
```

#### 兼容性说明

```rust
// 注意：这里的双重赋值是有意为之
// 为了与 C++ 版本保持完全一致的 RNG 状态，
// 我们需要调用两次 random_double()，但只使用第二次的结果
let _px_discard = rng.random_double(expanded.minx, expanded.maxx);
let px = rng.random_double(expanded.minx, expanded.maxx);
```

### 3. 区域注释

对于较长的函数，使用区域注释划分逻辑块：

```rust
fn generate_map(&mut self) {
    // ===================================
    // 1. 生成不规则网格
    // ===================================
    self.init_voronoi_data();
    
    // ===================================
    // 2. 生成地形高度图
    // ===================================
    self.initialize_heightmap();
    self.erode_heightmap();
    
    // ===================================
    // 3. 生成河流和等高线
    // ===================================
    self.generate_rivers();
    self.generate_contours();
    
    // ===================================
    // 4. 生成城市和边界
    // ===================================
    self.place_cities();
    self.generate_borders();
}
```

## 需要添加注释的关键文件

### 算法模块 (src/algorithms/)

1. **delaunay.rs** - Delaunay 三角剖分
   - 参考: M. Berg, "Computational geometry", Chapter 9
   - 原始实现: src/delaunay.h, src/delaunay.cpp
   - 关键函数: `triangulate()`, `insert_point_into_triangulation()`, `legalize_edge()`

2. **voronoi.rs** - Voronoi 图生成
   - 参考: M. Berg, "Computational geometry", Chapter 7
   - 原始实现: src/voronoi.h, src/voronoi.cpp
   - 关键函数: `delaunay_to_voronoi()`

3. **poisson_disc.rs** - Poisson 圆盘采样
   - 参考: R. Bridson, "Fast Poisson Disk Sampling", SIGGRAPH 2007
   - 原始实现: src/poissondiscsampler.h, src/poissondiscsampler.cpp
   - 关键函数: `generate_samples()`

### 数据结构模块 (src/data_structures/)

1. **extents2d.rs** - 2D 边界框
2. **vertex_map.rs** - 顶点映射
3. **node_map.rs** - 节点映射
4. **spatial_point_grid.rs** - 空间点网格

### 核心模块

1. **map_generator.rs** - 地图生成器
   - 参考: M. O'Leary, "Generating fantasy maps"
   - 原始实现: src/mapgenerator.h, src/mapgenerator.cpp
   - 关键部分:
     - 地形生成: `initialize_heightmap()`, `erode()`
     - 河流生成: `calculate_flux_map()`, 河流路径追踪
     - 城市放置: 城市评分计算，领地划分
     - 标签放置: 模拟退火算法

2. **config.rs** - 配置管理
3. **cli.rs** - CLI 应用逻辑

### 工具模块 (src/utils/)

1. **rand.rs** - 随机数生成器
   - 说明: 实现 glibc 的 rand() 算法以保持与 C++ 版本的兼容性
   
2. **font_face.rs** - 字体处理

## 注释模板

### 算法函数模板

```rust
/// [函数功能简述]
///
/// [详细说明算法原理和步骤]
///
/// # 算法原理
/// [解释算法的核心思想]
///
/// # 参数
/// * `param1` - [参数说明]
/// * `param2` - [参数说明]
///
/// # 返回
/// [返回值说明]
///
/// # 注意事项
/// [特殊情况、边界条件等]
///
/// # 参考来源
/// - [论文或文档引用]
/// - 原始 C++ 实现: [文件路径]
pub fn algorithm_function(...) -> ... {
    // 实现
}
```

### 数据结构模板

```rust
/// [数据结构名称]
///
/// [数据结构的用途和特点]
///
/// # 字段说明
/// - `field1`: [字段用途]
/// - `field2`: [字段用途]
///
/// # 使用场景
/// [在项目中的应用]
///
/// # 参考来源
/// - [相关文档]
pub struct DataStructure {
    /// [字段说明]
    pub field1: Type1,
    /// [字段说明]
    pub field2: Type2,
}
```

## 特殊注释标记

使用标准的注释标记来标识特殊情况：

```rust
// TODO: 未来需要实现的功能
// FIXME: 需要修复的问题
// NOTE: 重要说明
// HACK: 临时解决方案
// PERF: 性能相关的说明
// COMPAT: 兼容性相关的说明
```

## 示例：完整注释的函数

```rust
/// 使用 Planchon-Darboux 算法填充高度图中的凹陷
///
/// 在生成流向图之前，必须确保高度图中没有凹陷（局部最低点）。
/// 否则水流会被困在凹陷中，无法流向地图边缘。
///
/// # 算法原理
/// Planchon-Darboux 算法通过迭代提升凹陷区域的高度，
/// 直到所有点都有一条下坡路径通向地图边缘。
///
/// 算法步骤：
/// 1. 初始化：边界点保持原高度，内部点设为无穷大
/// 2. 迭代：对每个内部点，如果其高度大于任一邻居+ε，则降低其高度
/// 3. 重复直到收敛（没有点的高度发生变化）
///
/// # 为什么需要 ε
/// ε 是一个很小的正数（1e-5），用于确保填充后的地形有微小的坡度。
/// 这样可以保证水流有明确的流向，不会在平坦区域徘徊。
///
/// # 参数
/// 无（直接修改 self.height_map）
///
/// # 副作用
/// 修改 `self.height_map`，填充所有凹陷区域
///
/// # 参考来源
/// - O. Planchon and F. Darboux, "A fast, simple and versatile algorithm 
///   to fill the depressions of digital elevation models", 
///   CATENA, vol. 46, no. 2-3, pp. 159-176, 2002.
/// - 原始 C++ 实现: src/mapgenerator.cpp, fillDepressions()
fn fill_depressions(&mut self) {
    let max_h = self.height_map.max_val();
    let n = self.vertex_map.size();
    
    // 初始化最终高度图：边界点保持原值，内部点设为最大值
    let mut final_hm = NodeMap::new_filled(n, max_h);

    // 设置边界点的高度
    for i in 0..self.vertex_map.edge.len() {
        let v = self.vertex_map.edge[i];
        let idx = self.vertex_map.get_vertex_index(v) as usize;
        let hval = *self.height_map.get(idx);
        final_hm.set(idx, hval);
    }

    // ε 值：用于创建微小的坡度
    // 这个值足够小，不会显著改变地形，但足够大，可以确定流向
    let eps = 1e-5;
    
    // 迭代直到收敛
    loop {
        let mut updated = false;
        
        for i in 0..n {
            let h = *self.height_map.get(i);
            let fh = *final_hm.get(i);
            
            // 如果当前点已经是原始高度，跳过
            if h == fh { continue; }
            
            // 检查所有邻居
            for &nb in &self.neighbour_map[i] {
                let nval = *final_hm.get(nb);
                
                // 如果原始高度高于邻居，可以保持原始高度
                if h >= nval + eps {
                    final_hm.set(i, h);
                    updated = true;
                    break;
                }
                
                // 否则，设置为邻居高度 + ε（创建微小坡度）
                let hval = nval + eps;
                if fh > hval && hval > h {
                    final_hm.set(i, hval);
                    updated = true;
                }
            }
        }
        
        // 如果没有任何更新，算法收敛
        if !updated { break; }
    }
    
    self.height_map = final_hm;
}
```

## 注释检查清单

在添加注释时，确保：

- [ ] 所有 `pub` 函数都有文档注释
- [ ] 所有 `pub` 结构体都有文档注释
- [ ] 复杂算法有详细的步骤说明
- [ ] 魔法数字有解释
- [ ] 引用了原始论文或文档
- [ ] 标注了原始 C++ 实现的位置
- [ ] 解释了"为什么"而不仅仅是"做什么"
- [ ] 特殊的兼容性处理有说明
