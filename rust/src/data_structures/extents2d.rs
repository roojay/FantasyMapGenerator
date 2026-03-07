//! 二维边界框
//!
//! 表示平面上的一个矩形区域，用于定义采样范围、地图边界等。
//!
//! # 参考来源
//! - 原始 C++ 实现: src/extents2d.h

use super::dcel::Point;

/// 二维边界框（轴对齐矩形）
///
/// 使用最小和最大坐标来定义一个矩形区域。
/// 常用于：
/// - 定义 Poisson 采样的区域
/// - 定义地图的边界
/// - 空间查询和碰撞检测
#[derive(Clone, Copy, Debug, Default)]
pub struct Extents2d {
    /// 矩形左边界的 x 坐标
    pub minx: f64,
    
    /// 矩形下边界的 y 坐标
    pub miny: f64,
    
    /// 矩形右边界的 x 坐标
    pub maxx: f64,
    
    /// 矩形上边界的 y 坐标
    pub maxy: f64,
}

impl Extents2d {
    /// 创建一个新的边界框
    ///
    /// # 参数
    /// * `minx` - 左边界
    /// * `miny` - 下边界
    /// * `maxx` - 右边界
    /// * `maxy` - 上边界
    pub fn new(minx: f64, miny: f64, maxx: f64, maxy: f64) -> Self {
        Extents2d { minx, miny, maxx, maxy }
    }

    /// 检查点是否在边界框内
    ///
    /// # 边界处理
    /// 左边界和下边界包含在内（>=），右边界和上边界不包含（<）。
    /// 这种半开区间的定义避免了边界上的歧义。
    ///
    /// # 参数
    /// * `p` - 要检查的点
    ///
    /// # 返回
    /// 如果点在边界框内返回 true
    pub fn contains_point(&self, p: Point) -> bool {
        p.x >= self.minx && p.x < self.maxx && p.y >= self.miny && p.y < self.maxy
    }

    /// 检查坐标是否在边界框内
    ///
    /// 功能与 `contains_point` 相同，但直接接受坐标值。
    ///
    /// # 参数
    /// * `x` - x 坐标
    /// * `y` - y 坐标
    ///
    /// # 返回
    /// 如果坐标在边界框内返回 true
    pub fn contains_xy(&self, x: f64, y: f64) -> bool {
        x >= self.minx && x < self.maxx && y >= self.miny && y < self.maxy
    }
}
