//! 空间点网格数据结构
//!
//! SpatialPointGrid 是一个用于快速空间查询的数据结构。
//! 它将二维空间划分为均匀的网格，每个网格单元存储落在其中的点。
//!
//! # 算法原理
//! 使用空间哈希（Spatial Hashing）技术：
//! 1. 将空间划分为大小为 dx × dx 的网格
//! 2. 每个点根据其坐标映射到对应的网格单元
//! 3. 查询时只需检查相关的网格单元，而不是所有点
//!
//! # 时间复杂度
//! - 构建：O(n)，其中 n 是点的数量
//! - 查询：O(k)，其中 k 是查询区域内的点数（远小于 n）
//!
//! # 在地图生成中的应用
//! 用于 Poisson 圆盘采样算法中的邻域查询，
//! 快速检查新点周围是否有其他点。
//!
//! # 参考来源
//! - 原始 C++ 实现: src/spatialpointgrid.h, src/spatialpointgrid.cpp

use super::dcel::Point;
use super::extents2d::Extents2d;

/// 空间点网格
///
/// 使用均匀网格加速点的空间查询。
pub struct SpatialPointGrid {
    /// 网格单元大小
    dx: f64,
    /// 网格原点（左下角）
    offset: Point,
    /// 网格 x 方向的单元数
    isize: usize,
    /// 网格 y 方向的单元数
    jsize: usize,
    /// 原始点集
    points: Vec<Point>,
    /// 网格数据：grid[i + isize * j] 存储单元 (i, j) 中的点索引
    grid: Vec<Vec<usize>>,
}

impl SpatialPointGrid {
    /// 从点集创建空间网格
    ///
    /// # 算法流程
    /// 1. 计算所有点的边界框
    /// 2. 根据边界框和网格大小计算网格维度
    /// 3. 将每个点插入到对应的网格单元
    ///
    /// # 参数
    /// * `points` - 点集
    /// * `dx` - 网格单元大小
    ///
    /// # 参考来源
    /// - 原始 C++ 实现: src/spatialpointgrid.cpp, SpatialPointGrid 构造函数
    pub fn new(points: &[Point], dx: f64) -> Self {
        if points.is_empty() {
            return SpatialPointGrid {
                dx,
                offset: Point::new(0.0, 0.0),
                isize: 0,
                jsize: 0,
                points: Vec::new(),
                grid: Vec::new(),
            };
        }

        // 计算点集的边界框
        let ext = get_extents(points);
        let width = ext.maxx - ext.minx;
        let height = ext.maxy - ext.miny;

        // 计算网格维度
        let isize = (width / dx).ceil() as usize;
        let jsize = (height / dx).ceil() as usize;
        let total = isize.max(1) * jsize.max(1);
        let mut grid = vec![Vec::new(); total];

        // 将点插入到对应的网格单元
        let inv_dx = 1.0 / dx;
        for (point_idx, &p) in points.iter().enumerate() {
            // 计算点所在的网格单元索引
            let i = ((p.x - ext.minx) * inv_dx).floor() as usize;
            let j = ((p.y - ext.miny) * inv_dx).floor() as usize;
            let idx = i + isize * j;

            if idx < grid.len() {
                grid[idx].push(point_idx);
            }
        }

        SpatialPointGrid {
            dx,
            offset: Point::new(ext.minx, ext.miny),
            isize: isize.max(1),
            jsize: jsize.max(1),
            points: points.to_vec(),
            grid,
        }
    }

    /// 获取指定区域内的点数量
    ///
    /// # 算法流程
    /// 1. 计算查询区域覆盖的网格单元范围
    /// 2. 遍历这些网格单元
    /// 3. 对每个单元中的点进行精确的边界检查
    /// 4. 统计满足条件的点数量
    ///
    /// # 参数
    /// * `extents` - 查询区域
    ///
    /// # 返回
    /// 区域内的点数量
    ///
    /// # 参考来源
    /// - 原始 C++ 实现: src/spatialpointgrid.cpp, getPointCount()
    pub fn get_point_count(&self, extents: Extents2d) -> usize {
        if self.isize == 0 || self.jsize == 0 {
            return 0;
        }

        let inv_dx = 1.0 / self.dx;

        // 计算查询区域对应的网格单元范围
        let mini = ((extents.minx - self.offset.x) * inv_dx).floor() as i64;
        let minj = ((extents.miny - self.offset.y) * inv_dx).floor() as i64;
        let maxi = ((extents.maxx - self.offset.x) * inv_dx).floor() as i64;
        let maxj = ((extents.maxy - self.offset.y) * inv_dx).floor() as i64;

        // 限制在有效范围内
        let mini = mini.max(0) as usize;
        let minj = minj.max(0) as usize;
        let maxi = (maxi as usize).min(self.isize - 1);
        let maxj = (maxj as usize).min(self.jsize - 1);

        // 遍历相关的网格单元，统计点数量
        let mut count = 0;
        for j in minj..=maxj {
            for i in mini..=maxi {
                let idx = i + self.isize * j;

                // 对单元中的每个点进行精确检查
                for &point_idx in &self.grid[idx] {
                    let p = self.points[point_idx];
                    if extents.contains_point(p) {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    /// 获取指定区域内的点索引
    ///
    /// 返回值中的索引对应创建网格时传入的原始点集。
    ///
    /// # 与原始 C++ 的差异
    /// 原始 C++ 版本只有 `getPointCount()`，因为它主要服务于标签碰撞评分。
    /// Rust 版本额外暴露索引查询，是为了让后续逻辑能够在局部候选集中继续做
    /// “最近点/最近面”判定，例如 `MapGenerator::export_land_mask()` 的最近面查找。
    ///
    /// # 性能说明
    /// 这个接口本身仍然是局部网格查询 + 精确边界过滤，
    /// 但相比直接扫描全部点集，更适合需要二次排序或最近邻比较的场景。
    pub fn get_point_indices(&self, extents: Extents2d) -> Vec<usize> {
        if self.isize == 0 || self.jsize == 0 {
            return Vec::new();
        }

        let inv_dx = 1.0 / self.dx;

        let mini = ((extents.minx - self.offset.x) * inv_dx).floor() as i64;
        let minj = ((extents.miny - self.offset.y) * inv_dx).floor() as i64;
        let maxi = ((extents.maxx - self.offset.x) * inv_dx).floor() as i64;
        let maxj = ((extents.maxy - self.offset.y) * inv_dx).floor() as i64;

        let mini = mini.max(0) as usize;
        let minj = minj.max(0) as usize;
        let maxi = (maxi as usize).min(self.isize - 1);
        let maxj = (maxj as usize).min(self.jsize - 1);

        let mut indices = Vec::new();
        for j in minj..=maxj {
            for i in mini..=maxi {
                let idx = i + self.isize * j;
                for &point_idx in &self.grid[idx] {
                    let p = self.points[point_idx];
                    if extents.contains_point(p) {
                        indices.push(point_idx);
                    }
                }
            }
        }

        indices
    }
}

/// 计算点集的边界框
///
/// # 参数
/// * `points` - 点集（至少包含一个点）
///
/// # 返回
/// 包含所有点的最小边界框
fn get_extents(points: &[Point]) -> Extents2d {
    let mut e = Extents2d::new(points[0].x, points[0].y, points[0].x, points[0].y);

    for p in points {
        e.minx = e.minx.min(p.x);
        e.miny = e.miny.min(p.y);
        e.maxx = e.maxx.max(p.x);
        e.maxy = e.maxy.max(p.y);
    }

    e
}
