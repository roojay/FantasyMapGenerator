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
        Extents2d {
            minx,
            miny,
            maxx,
            maxy,
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_bounds() {
        let e = Extents2d::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(e.minx, 1.0);
        assert_eq!(e.miny, 2.0);
        assert_eq!(e.maxx, 3.0);
        assert_eq!(e.maxy, 4.0);
    }

    #[test]
    fn contains_point_interior() {
        let e = Extents2d::new(0.0, 0.0, 10.0, 10.0);
        assert!(e.contains_point(Point::new(5.0, 5.0)));
    }

    #[test]
    fn contains_point_on_min_boundary_inclusive() {
        let e = Extents2d::new(0.0, 0.0, 10.0, 10.0);
        assert!(e.contains_point(Point::new(0.0, 0.0)));
    }

    #[test]
    fn contains_point_on_max_boundary_exclusive() {
        let e = Extents2d::new(0.0, 0.0, 10.0, 10.0);
        assert!(!e.contains_point(Point::new(10.0, 5.0)));
        assert!(!e.contains_point(Point::new(5.0, 10.0)));
    }

    #[test]
    fn contains_point_outside() {
        let e = Extents2d::new(0.0, 0.0, 10.0, 10.0);
        assert!(!e.contains_point(Point::new(-1.0, 5.0)));
        assert!(!e.contains_point(Point::new(5.0, -1.0)));
        assert!(!e.contains_point(Point::new(11.0, 5.0)));
        assert!(!e.contains_point(Point::new(5.0, 11.0)));
    }

    #[test]
    fn contains_xy_matches_contains_point() {
        let e = Extents2d::new(1.0, 2.0, 3.0, 4.0);
        let cases = [(2.0, 3.0, true), (0.0, 3.0, false), (3.0, 3.0, false)];
        for (x, y, expected) in cases {
            assert_eq!(e.contains_xy(x, y), expected, "({x}, {y})");
            assert_eq!(e.contains_point(Point::new(x, y)), expected, "({x}, {y}) point");
        }
    }

    #[test]
    fn default_is_zero_area() {
        let e = Extents2d::default();
        assert_eq!(e.minx, 0.0);
        assert_eq!(e.miny, 0.0);
        assert_eq!(e.maxx, 0.0);
        assert_eq!(e.maxy, 0.0);
    }
}
