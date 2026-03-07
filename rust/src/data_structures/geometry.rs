//! 几何工具函数
//!
//! 提供基本的几何计算功能，用于线段相交检测等操作。
//!
//! # 参考来源
//! - M. Berg, "Computational geometry", Berlin: Springer, 2000.
//! - 原始 C++ 实现: src/geometry.h

use super::dcel::Point;

/// 计算两条直线的交点
///
/// 使用参数方程表示直线：
/// - 第一条直线: P + t*R (其中 t 是参数)
/// - 第二条直线: Q + u*S (其中 u 是参数)
///
/// # 算法原理
/// 通过求解线性方程组找到交点：
/// ```text
/// P + t*R = Q + u*S
/// ```
/// 使用叉积来求解 t 的值。
///
/// # 参数
/// * `p` - 第一条直线上的一点
/// * `r` - 第一条直线的方向向量
/// * `q` - 第二条直线上的一点
/// * `s` - 第二条直线的方向向量
///
/// # 返回
/// - `Some(Point)` - 如果两条直线相交，返回交点
/// - `None` - 如果两条直线平行或重合
///
/// # 参考来源
/// - 原始 C++ 实现: src/geometry.h
pub fn line_intersection(p: Point, r: Point, q: Point, s: Point) -> Option<Point> {
    // 计算方向向量的叉积
    // 如果叉积为 0，说明两条直线平行
    let cross = r.x * s.y - r.y * s.x;
    let eps = 1e-9;
    if cross.abs() < eps {
        return None;
    }
    
    // 计算从 P 到 Q 的向量
    let vx = q.x - p.x;
    let vy = q.y - p.y;
    
    // 使用叉积公式求解参数 t
    let t = (vx * s.y - vy * s.x) / cross;
    
    // 计算交点: P + t*R
    Some(Point::new(p.x + t * r.x, p.y + t * r.y))
}

/// 检测两条线段是否相交
///
/// 使用方向判断法（orientation test）来检测线段相交。
///
/// # 算法原理
/// 两条线段 AB 和 CD 相交，当且仅当：
/// 1. C 和 D 在直线 AB 的两侧
/// 2. A 和 B 在直线 CD 的两侧
///
/// 通过计算叉积的符号来判断点在直线的哪一侧：
/// - 叉积 > 0: 点在直线左侧
/// - 叉积 < 0: 点在直线右侧
/// - 叉积 = 0: 点在直线上
///
/// # 参数
/// * `a` - 第一条线段的起点
/// * `b` - 第一条线段的终点
/// * `c` - 第二条线段的起点
/// * `d` - 第二条线段的终点
///
/// # 返回
/// 如果两条线段相交返回 true
///
/// # 参考来源
/// - M. Berg, "Computational geometry", Chapter 1
/// - 原始 C++ 实现: src/geometry.h
pub fn line_segment_intersection(a: Point, b: Point, c: Point, d: Point) -> bool {
    // 计算 C 和 D 相对于直线 AB 的方向
    // 使用叉积: (D-A) × (C-A) 和 (C-A) × (D-A)
    let c1 = (d.y - a.y) * (c.x - a.x) > (c.y - a.y) * (d.x - a.x);
    let c2 = (d.y - b.y) * (c.x - b.x) > (c.y - b.y) * (d.x - b.x);
    
    // 计算 A 和 B 相对于直线 CD 的方向
    let c3 = (c.y - a.y) * (b.x - a.x) > (b.y - a.y) * (c.x - a.x);
    let c4 = (d.y - a.y) * (b.x - a.x) > (b.y - a.y) * (d.x - a.x);
    
    // 如果 C 和 D 在 AB 两侧，且 A 和 B 在 CD 两侧，则相交
    (c1 != c2) && (c3 != c4)
}
