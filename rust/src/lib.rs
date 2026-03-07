//! Fantasy Map Generator - Rust 实现
//!
//! 这是一个幻想地图生成器，基于 Martin O'Leary 的地图生成方法。
//! 它使用 Voronoi 图、Delaunay 三角剖分和各种地形生成算法来创建
//! 具有山脉、河流、城市和边界的幻想地图。
//!
//! # 主要特性
//!
//! - **不规则网格生成**: 使用 Poisson 圆盘采样和 Voronoi 图
//! - **地形生成**: 使用高度图原语（山丘、锥体、斜坡）
//! - **地形侵蚀**: 模拟水流侵蚀效果
//! - **河流生成**: 基于流量图生成河流网络
//! - **城市放置**: 基于评分系统放置城市和城镇
//! - **领地划分**: 使用移动成本计算领地边界
//! - **标签放置**: 使用模拟退火算法优化标签位置
//!
//! # 算法参考
//!
//! 本项目实现了多个经典的计算几何和地形生成算法：
//!
//! - **Poisson 圆盘采样**: R. Bridson, "Fast Poisson Disk Sampling", SIGGRAPH 2007
//! - **Delaunay 三角剖分**: M. Berg, "Computational geometry", Chapter 9
//! - **Voronoi 图**: M. Berg, "Computational geometry", Chapter 7
//! - **凹陷填充**: O. Planchon and F. Darboux, "A fast, simple and versatile algorithm 
//!   to fill the depressions of digital elevation models", CATENA, 2002
//! - **标签放置**: S. Edmondson et al., "A General Cartographic Labeling Algorithm", 
//!   MERL, 1996
//!
//! # 项目结构
//!
//! - `algorithms`: 几何算法（Delaunay、Voronoi、Poisson 采样）
//! - `data_structures`: 核心数据结构（DCEL、边界框、网格等）
//! - `utils`: 工具函数（随机数生成器、字体处理）
//! - `map_generator`: 地图生成核心逻辑
//! - `config`: 配置和命令行参数
//! - `cli`: 命令行应用程序
//!
//! # 使用示例
//!
//! ## 作为库使用
//!
//! ```rust,no_run
//! use fantasy_map_generator::{Config, MapGenerator, Extents2d, GlibcRand};
//!
//! // 创建地图生成器
//! let extents = Extents2d::new(0.0, 0.0, 20.0, 10.0);
//! let rng = GlibcRand::new(12345);
//! let mut map = MapGenerator::new(extents, 0.08, 1920, 1080, rng);
//!
//! // 初始化并生成地图
//! map.initialize();
//! // ... 添加地形特征
//! ```
//!
//! ## 作为命令行工具使用
//!
//! ```bash
//! # 使用固定种子生成地图
//! cargo run --release -- --seed 12345 --cities 5 --towns 10
//!
//! # 使用随机种子
//! cargo run --release -- --timeseed --resolution 0.08
//! ```
//!
//! # 与 C++ 版本的兼容性
//!
//! 本 Rust 实现与原始 C++ 版本保持完全兼容：
//! - 使用相同的随机数生成器（glibc rand）
//! - 实现相同的算法逻辑
//! - 生成相同格式的输出
//!
//! 相同的种子会产生完全相同的地图。
//!
//! # 参考来源
//!
//! - M. O'Leary, "Generating fantasy maps", https://mewo2.com/notes/terrain/
//! - 原始 C++ 实现: https://github.com/rlguy/FantasyMapGenerator

#![allow(dead_code, unused_imports, unused_variables)]

// ===================================
// 核心模块（按功能组织）
// ===================================

/// 几何算法模块
///
/// 包含 Delaunay 三角剖分、Voronoi 图生成和 Poisson 圆盘采样算法。
pub mod algorithms;

/// 核心数据结构模块
///
/// 包含 DCEL、几何工具、边界框和各种映射数据结构。
pub mod data_structures;

/// 工具模块
///
/// 包含随机数生成器、字体处理等辅助功能。
pub mod utils;

// ===================================
// 地图生成核心
// ===================================

/// 地图生成器
///
/// 协调所有算法和数据结构，生成完整的幻想地图。
pub mod map_generator;

// ===================================
// 应用程序接口
// ===================================

/// 配置和命令行参数
pub mod config;

/// 命令行应用程序
pub mod cli;

/// 渲染模块（需要 render feature）
#[cfg(feature = "render")]
pub mod render;

/// WASM 绑定模块（需要 wasm feature）
#[cfg(feature = "wasm")]
pub mod wasm;

// ===================================
// 便捷的重导出
// ===================================

/// 配置结构体
pub use config::Config;

/// 常用数据结构
pub use data_structures::{Extents2d, Point};

/// 地图生成器
pub use map_generator::MapGenerator;

/// 随机数生成器
pub use utils::GlibcRand;
