//! Delaunay 三角剖分算法
//!
//! Delaunay 三角剖分是一种将平面上的点集连接成三角形网格的方法，
//! 具有最大化最小角的特性，避免产生狭长的三角形。
//!
//! # 算法原理
//! 使用增量插入算法：
//! 1. 创建一个包含所有点的超级三角形
//! 2. 逐个插入点，每次插入后通过边翻转维护 Delaunay 性质
//! 3. 删除与超级三角形相关的三角形
//!
//! # Delaunay 性质
//! 对于三角剖分中的任意三角形，其外接圆内不包含其他点。
//! 这个性质保证了三角剖分的唯一性和最优性。
//!
//! # 在地图生成中的应用
//! Delaunay 三角剖分用于：
//! 1. 生成 Voronoi 图（Delaunay 的对偶图）
//! 2. Voronoi 图的顶点作为地图的不规则网格节点
//!
//! # 参考来源
//! - M. Berg, "Computational geometry", Berlin: Springer, 2000, Chapter 9
//! - 原始 C++ 实现: src/delaunay.h, src/delaunay.cpp

use crate::data_structures::dcel::{Dcel, Face, HalfEdge, Point, Ref, Vertex};
use crate::data_structures::geometry::{line_intersection, line_segment_intersection};

/// 对点集进行 Delaunay 三角剖分
///
/// # 算法流程
/// 1. 创建包含所有点的超级三角形
/// 2. 逐个插入点到三角剖分中
/// 3. 每次插入后通过边翻转维护 Delaunay 性质
/// 4. 删除与超级三角形相关的三角形
///
/// # 参数
/// * `points` - 要剖分的点集（会被修改，点会被逐个弹出）
///
/// # 返回
/// 包含三角剖分结果的 DCEL 数据结构
///
/// # 时间复杂度
/// O(n log n)，其中 n 是点的数量
///
/// # 参考来源
/// - M. Berg, "Computational geometry", Chapter 9
/// - 原始 C++ 实现: src/delaunay.cpp, triangulate()
pub fn triangulate(points: &mut Vec<Point>) -> Dcel {
    if points.is_empty() {
        return Dcel::new();
    }

    // ===================================
    // 1. 初始化：创建超级三角形
    // ===================================
    let mut t = init_triangulation(points);

    // ===================================
    // 2. 增量插入点
    // ===================================
    while let Some(p) = points.pop() {
        // 定位点所在的三角形
        let f = locate_triangle_at_point(p, &t);
        if f.id.is_valid() {
            // 将点插入三角剖分，并维护 Delaunay 性质
            insert_point_into_triangulation(p, f, &mut t);
        }
    }

    // ===================================
    // 3. 清理：删除超级三角形
    // ===================================
    cleanup(&mut t);
    t
}

/// 计算包含所有点的超级三角形
///
/// 超级三角形必须足够大，能够包含所有输入点。
/// 算法结束后，与超级三角形相关的三角形会被删除。
///
/// # 构造方法
/// 1. 计算所有点的边界框
/// 2. 扩展边界框
/// 3. 构造一个等腰三角形，顶点在上方，底边在下方
///
/// # 为什么需要超级三角形
/// 增量插入算法需要一个初始的三角剖分。
/// 超级三角形提供了这个初始结构，简化了算法实现。
///
/// # 参数
/// * `points` - 输入点集
///
/// # 返回
/// 超级三角形的三个顶点 (p1, p2, p3)
///
/// # 参考来源
/// - 原始 C++ 实现: src/delaunay.cpp, getSuperTriangle()
fn get_super_triangle(points: &[Point]) -> (Point, Point, Point) {
    let eps = 1e-3;
    
    // 计算点集的边界框
    let mut minx = points[0].x;
    let mut miny = points[0].y;
    let mut maxx = minx + eps;
    let mut maxy = miny + eps;
    for p in points {
        if p.x < minx { minx = p.x; }
        if p.y < miny { miny = p.y; }
        if p.x > maxx { maxx = p.x; }
        if p.y > maxy { maxy = p.y; }
    }
    
    // 扩展边界框，确保超级三角形足够大
    let expand = f64::max(0.1 * (maxx - minx), 0.1 * (maxy - miny));
    minx -= expand;
    miny -= 5.0 * expand;  // 底边向下扩展更多，确保所有点都在三角形内
    maxx += expand;
    maxy += expand;

    // 构造等腰三角形
    // p1: 顶点（在上方中央）
    let p1x = 0.5 * (minx + maxx);
    let p1y = maxy + 0.5 * (maxy - miny);
    let p1 = Point::new(p1x, p1y);

    // p2: 左下角顶点
    let m = (maxy - p1y) / (maxx - p1x);
    let p2x = (1.0 / m) * (miny - p1y + m * p1x);
    let p2 = Point::new(p2x, miny);

    // p3: 右下角顶点
    let m2 = (maxy - p1y) / (minx - p1x);
    let p3x = (1.0 / m2) * (miny - p1y + m2 * p1x);
    let p3 = Point::new(p3x, miny);

    (p1, p2, p3)
}

/// 初始化三角剖分
///
/// 创建一个只包含超级三角形的初始 DCEL 结构。
/// 这个超级三角形将作为增量插入算法的起点。
///
/// # 参数
/// * `points` - 输入点集（用于计算超级三角形的大小）
///
/// # 返回
/// 包含超级三角形的 DCEL 结构
///
/// # 参考来源
/// - 原始 C++ 实现: src/delaunay.cpp, initTriangulation()
fn init_triangulation(points: &[Point]) -> Dcel {
    let (s1, s2, s3) = get_super_triangle(points);
    let mut t = Dcel::new();

    let mut p1 = t.create_vertex(s1);
    let mut p2 = t.create_vertex(s2);
    let mut p3 = t.create_vertex(s3);

    let mut e12 = t.create_half_edge();
    let mut e23 = t.create_half_edge();
    let mut e31 = t.create_half_edge();
    let mut e13 = t.create_half_edge();
    let mut e32 = t.create_half_edge();
    let mut e21 = t.create_half_edge();
    let mut f0 = t.create_face();

    p1.incident_edge = e12.id;
    p2.incident_edge = e23.id;
    p3.incident_edge = e31.id;
    t.update_vertex(p1);
    t.update_vertex(p2);
    t.update_vertex(p3);

    e12.origin = p1.id;
    e12.twin = e21.id;
    e12.incident_face = f0.id;
    e12.next = e23.id;
    e12.prev = e31.id;
    t.update_edge(e12);

    e23.origin = p2.id;
    e23.twin = e32.id;
    e23.incident_face = f0.id;
    e23.next = e31.id;
    e23.prev = e12.id;
    t.update_edge(e23);

    e31.origin = p3.id;
    e31.twin = e13.id;
    e31.incident_face = f0.id;
    e31.next = e12.id;
    e31.prev = e23.id;
    t.update_edge(e31);

    e13.origin = p1.id;
    e13.twin = e31.id;
    e13.next = e32.id;
    e13.prev = e21.id;
    t.update_edge(e13);

    e32.origin = p3.id;
    e32.twin = e23.id;
    e32.next = e21.id;
    e32.prev = e13.id;
    t.update_edge(e32);

    e21.origin = p2.id;
    e21.twin = e12.id;
    e21.next = e13.id;
    e21.prev = e32.id;
    t.update_edge(e21);

    f0.outer_component = e12.id;
    t.update_face(f0);

    t
}

/// 判断点是否在三角形内部
///
/// 使用重心坐标法判断点是否在三角形内部。
/// 重心坐标 (s, t, 1-s-t) 表示点相对于三角形三个顶点的权重。
///
/// # 算法原理
/// 如果点在三角形内部，则其重心坐标满足：
/// - s >= 0
/// - t >= 0
/// - 1 - s - t >= 0
///
/// # 参数
/// * `p` - 待测试的点
/// * `f` - 三角形面
/// * `t` - DCEL 数据结构
///
/// # 返回
/// 如果点在三角形内部（包括边界）返回 true，否则返回 false
///
/// # 参考来源
/// - 原始 C++ 实现: src/delaunay.cpp, isPointInsideTriangle()
fn is_point_inside_triangle(p: Point, f: &Face, t: &Dcel) -> bool {
    let h = t.outer_component(f);
    let p0 = t.origin(h).position;
    let h2 = t.next(h);
    let p1 = t.origin(h2).position;
    let p2 = t.origin(t.next(h2)).position;

    // 计算三角形面积
    let area = 0.5 * (-p1.y * p2.x + p0.y * (-p1.x + p2.x) + p0.x * (p1.y - p2.y) + p1.x * p2.y);
    
    // 计算重心坐标
    let s = 1.0 / (2.0 * area) * (p0.y * p2.x - p0.x * p2.y + (p2.y - p0.y) * p.x + (p0.x - p2.x) * p.y);
    let t_val = 1.0 / (2.0 * area) * (p0.x * p1.y - p0.y * p1.x + (p0.y - p1.y) * p.x + (p1.x - p0.x) * p.y);

    // 检查重心坐标是否都非负
    s >= 0.0 && t_val >= 0.0 && 1.0 - s - t_val >= 0.0
}

/// 计算三角形的重心
///
/// 三角形的重心是三个顶点坐标的算术平均值。
///
/// # 参数
/// * `f` - 三角形面
/// * `t` - DCEL 数据结构
///
/// # 返回
/// 三角形的重心坐标
///
/// # 参考来源
/// - 原始 C++ 实现: src/delaunay.cpp, computeTriangleCentroid()
fn compute_triangle_centroid(f: &Face, t: &Dcel) -> Point {
    let h = t.outer_component(f);
    let p0 = t.origin(h).position;
    let p1 = t.origin(t.next(h)).position;
    let p2 = t.origin(t.prev(h)).position;
    let frac = 1.0 / 3.0;
    Point::new(frac * (p0.x + p1.x + p2.x), frac * (p0.y + p1.y + p2.y))
}

/// 判断线段是否与边相交
///
/// 用于在点定位算法中判断从当前三角形重心到目标点的射线
/// 是否穿过某条边，从而确定应该移动到哪个相邻三角形。
///
/// # 参数
/// * `p0` - 线段起点
/// * `p1` - 线段终点
/// * `h` - 待测试的半边
/// * `t` - DCEL 数据结构
///
/// # 返回
/// 如果线段与边相交返回 true，否则返回 false
///
/// # 参考来源
/// - 原始 C++ 实现: src/delaunay.cpp, isSegmentIntersectingEdge()
fn is_segment_intersecting_edge(p0: Point, p1: Point, h: HalfEdge, t: &Dcel) -> bool {
    let c = t.origin(h).position;
    let d = t.origin(t.twin(h)).position;
    line_segment_intersection(p0, p1, c, d)
}

/// 定位包含指定点的三角形
///
/// 使用"行走"算法从任意三角形开始，沿着指向目标点的方向
/// 逐步移动到相邻三角形，直到找到包含目标点的三角形。
///
/// # 算法流程
/// 1. 从第一个三角形开始
/// 2. 如果点在当前三角形内，返回该三角形
/// 3. 否则，找到从重心到目标点的射线穿过的边
/// 4. 移动到该边对面的相邻三角形
/// 5. 重复步骤 2-4
///
/// # 防止无限循环
/// - 记录最近访问的 3 个三角形，检测循环
/// - 设置最大迭代次数限制
///
/// # 参数
/// * `p` - 目标点
/// * `t` - DCEL 数据结构
///
/// # 返回
/// 包含目标点的三角形面，如果未找到返回无效面
///
/// # 参考来源
/// - 原始 C++ 实现: src/delaunay.cpp, locateTriangleAtPoint()
fn locate_triangle_at_point(p: Point, t: &Dcel) -> Face {
    if t.faces.is_empty() {
        return Face::new();
    }
    
    // 从第一个面开始
    let mut f = t.face(Ref::new(0));
    
    // 最大迭代次数：与三角形数量的平方根成正比
    let max_count = (2.0 * (t.faces.len() as f64).sqrt()) as i32;
    let mut count = 0;
    
    // 记录最近访问的 3 个面，用于检测循环
    let mut face_history = [-1i32; 3];

    loop {
        // 检查点是否在当前三角形内
        if is_point_inside_triangle(p, &f, t) {
            return f;
        }
        
        // 从重心向目标点移动，找到穿过的边
        let p0 = compute_triangle_centroid(&f, t);
        let mut neighbour_found = false;
        let h0 = t.outer_component(&f);
        let mut h = h0;
        
        // 检查三条边
        for _ in 0..3 {
            if is_segment_intersecting_edge(p0, p, h, t) {
                let tw = t.twin(h);
                if tw.incident_face.is_valid() {
                    f = t.face(tw.incident_face);
                    neighbour_found = true;
                    break;
                }
            }
            h = t.next(h);
        }
        
        if !neighbour_found { break; }

        // 更新访问历史，检测循环
        face_history[2] = face_history[1];
        face_history[1] = face_history[0];
        face_history[0] = f.id.id;
        if face_history[0] == face_history[2] { break; }

        // 检查迭代次数
        count += 1;
        if count > max_count { break; }
    }
    
    Face::new()
}

/// 计算点到边的距离
///
/// 计算点到直线（由边定义）的垂直距离。
/// 用于判断新插入的点是否非常接近某条边。
///
/// # 算法
/// 使用点到直线距离公式：
/// distance = |ax + by + c| / sqrt(a² + b²)
///
/// # 参数
/// * `p0` - 待测试的点
/// * `h` - 边（半边）
/// * `t` - DCEL 数据结构
///
/// # 返回
/// 点到边的垂直距离，如果边长度为 0 返回无穷大
///
/// # 参考来源
/// - 原始 C++ 实现: src/delaunay.cpp, pointToEdgeDistance()
fn point_to_edge_distance(p0: Point, h: HalfEdge, t: &Dcel) -> f64 {
    let p1 = t.origin(h).position;
    let p2 = t.origin(t.twin(h)).position;
    let vx = p2.x - p1.x;
    let vy = p2.y - p1.y;
    let len = (vx * vx + vy * vy).sqrt();
    
    // 边长度为 0，返回无穷大
    if len < 1e-12 { return f64::INFINITY; }
    
    // 计算垂直距离
    ((vx * (p1.y - p0.y) - (p1.x - p0.x) * vy) / len).abs()
}

/// 将点插入三角剖分
///
/// 根据点的位置选择合适的插入方式：
/// - 如果点在三角形内部：分裂三角形为 3 个新三角形
/// - 如果点在边上：分裂相邻的 2 个三角形为 4 个新三角形
/// - 如果点在顶点上（距离 2 条边都很近）：忽略该点
///
/// # 参数
/// * `p` - 要插入的点
/// * `f` - 包含该点的三角形
/// * `t` - DCEL 数据结构
///
/// # 参考来源
/// - 原始 C++ 实现: src/delaunay.cpp, insertPointIntoTriangulation()
fn insert_point_into_triangulation(p: Point, f: Face, t: &mut Dcel) {
    let eps = 1e-9;
    let mut close_edge_count = 0;
    let mut close_edge = HalfEdge::new();

    // 检查点是否非常接近某条边
    let h0 = t.outer_component(&f);
    let mut h = h0;
    for _ in 0..3 {
        let dist = point_to_edge_distance(p, h, t);
        if dist < eps {
            close_edge = h;
            close_edge_count += 1;
            // 如果点接近 2 条边，说明点在顶点上，忽略
            if close_edge_count == 2 {
                return;
            }
        }
        h = t.next(h);
    }

    // 根据点的位置选择插入方式
    if close_edge_count == 0 {
        // 点在三角形内部
        insert_point_into_triangle(p, f, t);
    } else {
        // 点在边上
        insert_point_into_triangle_edge(p, f, close_edge, t);
    }
}

/// 将点插入三角形内部
///
/// 将一个三角形分裂为 3 个新三角形，新点作为公共顶点。
/// 插入后需要对 3 条新边进行合法化检查。
///
/// # 算法流程
/// 1. 创建 3 条从新点到原三角形顶点的边
/// 2. 更新 DCEL 结构，形成 3 个新三角形
/// 3. 对 3 条原边进行合法化检查
///
/// # 参数
/// * `p` - 要插入的点
/// * `f` - 包含该点的三角形
/// * `t` - DCEL 数据结构
///
/// # 参考来源
/// - 原始 C++ 实现: src/delaunay.cpp, insertPointIntoTriangle()
fn insert_point_into_triangle(p: Point, f: Face, t: &mut Dcel) {
    let eij = t.outer_component(&f);
    let ejk = t.next(eij);
    let eki = t.next(ejk);
    let f1 = f;
    let pi = t.origin(eij);
    let pj = t.origin(ejk);
    let pk = t.origin(eki);

    let mut eri = t.create_half_edge();
    let mut eir = t.create_half_edge();
    let mut erj = t.create_half_edge();
    let mut ejr = t.create_half_edge();
    let mut erk = t.create_half_edge();
    let mut ekr = t.create_half_edge();
    let mut f2 = t.create_face();
    let mut f3 = t.create_face();
    let mut pr = t.create_vertex(p);

    let mut eij = eij;
    let mut ejk = ejk;
    let mut eki = eki;
    let mut f1 = f1;

    eij.next = ejr.id;
    eij.prev = eri.id;
    t.update_edge(eij);

    ejk.incident_face = f2.id;
    ejk.next = ekr.id;
    ejk.prev = erj.id;
    t.update_edge(ejk);

    eki.incident_face = f3.id;
    eki.next = eir.id;
    eki.prev = erk.id;
    t.update_edge(eki);

    f1.outer_component = eij.id;
    t.update_face(f1);

    eri.origin = pr.id;
    eri.twin = eir.id;
    eri.incident_face = f1.id;
    eri.next = eij.id;
    eri.prev = ejr.id;
    t.update_edge(eri);

    eir.origin = pi.id;
    eir.twin = eri.id;
    eir.incident_face = f3.id;
    eir.next = erk.id;
    eir.prev = eki.id;
    t.update_edge(eir);

    erj.origin = pr.id;
    erj.twin = ejr.id;
    erj.incident_face = f2.id;
    erj.next = ejk.id;
    erj.prev = ekr.id;
    t.update_edge(erj);

    ejr.origin = pj.id;
    ejr.twin = erj.id;
    ejr.incident_face = f1.id;
    ejr.next = eri.id;
    ejr.prev = eij.id;
    t.update_edge(ejr);

    erk.origin = pr.id;
    erk.twin = ekr.id;
    erk.incident_face = f3.id;
    erk.next = eki.id;
    erk.prev = eir.id;
    t.update_edge(erk);

    ekr.origin = pk.id;
    ekr.twin = erk.id;
    ekr.incident_face = f2.id;
    ekr.next = erj.id;
    ekr.prev = ejk.id;
    t.update_edge(ekr);

    f2.outer_component = ejk.id;
    t.update_face(f2);

    f3.outer_component = eki.id;
    t.update_face(f3);

    pr.incident_edge = eri.id;
    t.update_vertex(pr);

    legalize_edge(pr, eij, t);
    legalize_edge(pr, ejk, t);
    legalize_edge(pr, eki, t);
}

/// 将点插入三角形的边上
///
/// 将相邻的 2 个三角形分裂为 4 个新三角形。
/// 新点位于两个三角形的公共边上。
/// 插入后需要对 4 条边进行合法化检查。
///
/// # 算法流程
/// 1. 删除公共边
/// 2. 创建 4 条从新点到对面顶点的边
/// 3. 更新 DCEL 结构，形成 4 个新三角形
/// 4. 对 4 条边进行合法化检查
///
/// # 参数
/// * `p` - 要插入的点
/// * `_f` - 包含该点的三角形（未使用）
/// * `h` - 点所在的边
/// * `t` - DCEL 数据结构
///
/// # 参考来源
/// - 原始 C++ 实现: src/delaunay.cpp, insertPointIntoTriangleEdge()
fn insert_point_into_triangle_edge(p: Point, _f: Face, h: HalfEdge, t: &mut Dcel) {
    let eij = h;
    let ejk = t.next(eij);
    let eki = t.next(ejk);
    let eji = t.twin(eij);
    let eil = t.next(eji);
    let elj = t.next(eil);

    let f1 = t.incident_face(eij);
    let f2 = t.incident_face(eji);

    let pj = t.origin(ejk);
    let pk = t.origin(eki);
    let pl = t.origin(elj);

    let eir = eij;
    let eri = eji;

    let mut erj = t.create_half_edge();
    let mut ejr = t.create_half_edge();
    let mut erk = t.create_half_edge();
    let mut ekr = t.create_half_edge();
    let mut erl = t.create_half_edge();
    let mut elr = t.create_half_edge();
    let mut f3 = t.create_face();
    let mut f4 = t.create_face();
    let mut pr = t.create_vertex(p);

    let eij2 = eij;
    let mut ejk2 = ejk;
    let mut eki2 = eki;
    let eji2 = eji;
    let mut eil2 = eil;
    let mut elj2 = elj;
    let mut f1 = f1;
    let mut f2 = f2;
    let mut eir2 = eir;
    let mut eri2 = eri;

    ejk2.incident_face = f4.id;
    ejk2.next = ekr.id;
    ejk2.prev = erj.id;
    t.update_edge(ejk2);

    eki2.next = eir2.id;
    eki2.prev = erk.id;
    t.update_edge(eki2);

    eil2.next = elr.id;
    eil2.prev = eri2.id;
    t.update_edge(eil2);

    elj2.incident_face = f3.id;
    elj2.next = ejr.id;
    elj2.prev = erl.id;
    t.update_edge(elj2);

    f1.outer_component = eki2.id;
    t.update_face(f1);

    f2.outer_component = eil2.id;
    t.update_face(f2);

    let mut pj2 = pj;
    pj2.incident_edge = ejk2.id;
    t.update_vertex(pj2);

    eir2.next = erk.id;
    eir2.prev = eki2.id;
    t.update_edge(eir2);

    eri2.origin = pr.id;
    eri2.next = eil2.id;
    eri2.prev = elr.id;
    t.update_edge(eri2);

    erj.origin = pr.id;
    erj.twin = ejr.id;
    erj.incident_face = f4.id;
    erj.next = ejk2.id;
    erj.prev = ekr.id;
    t.update_edge(erj);

    ejr.origin = pj.id;
    ejr.twin = erj.id;
    ejr.incident_face = f3.id;
    ejr.next = erl.id;
    ejr.prev = elj2.id;
    t.update_edge(ejr);

    erk.origin = pr.id;
    erk.twin = ekr.id;
    erk.incident_face = f1.id;
    erk.next = eki2.id;
    erk.prev = eir2.id;
    t.update_edge(erk);

    ekr.origin = pk.id;
    ekr.twin = erk.id;
    ekr.incident_face = f4.id;
    ekr.next = erj.id;
    ekr.prev = ejk2.id;
    t.update_edge(ekr);

    erl.origin = pr.id;
    erl.twin = elr.id;
    erl.incident_face = f3.id;
    erl.next = elj2.id;
    erl.prev = ejr.id;
    t.update_edge(erl);

    elr.origin = pl.id;
    elr.twin = erl.id;
    elr.incident_face = f2.id;
    elr.next = eri2.id;
    elr.prev = eil2.id;
    t.update_edge(elr);

    f3.outer_component = elj2.id;
    t.update_face(f3);

    f4.outer_component = ejk2.id;
    t.update_face(f4);

    pr.incident_edge = eri2.id;
    t.update_vertex(pr);

    legalize_edge(pr, eil2, t);
    legalize_edge(pr, elj2, t);
    legalize_edge(pr, ejk2, t);
    legalize_edge(pr, eki2, t);
}

/// 判断边是否满足 Delaunay 性质（合法）
///
/// 一条边是合法的，当且仅当其对面顶点不在另一侧三角形的外接圆内。
/// 这是 Delaunay 三角剖分的核心性质。
///
/// # 算法原理
/// 1. 计算包含新插入点 pr 的三角形的外接圆圆心
/// 2. 检查对面三角形的第三个顶点是否在外接圆内
/// 3. 如果在圆内，边不合法，需要翻转
///
/// # 外接圆圆心计算
/// 通过求两条边的垂直平分线的交点来计算外接圆圆心。
///
/// # 参数
/// * `pr` - 新插入的顶点
/// * `e` - 待检查的边
/// * `t` - DCEL 数据结构
///
/// # 返回
/// 如果边合法返回 true，否则返回 false
///
/// # 参考来源
/// - M. Berg, "Computational geometry", Chapter 9
/// - 原始 C++ 实现: src/delaunay.cpp, isEdgeLegal()
fn is_edge_legal(pr: Vertex, e: HalfEdge, t: &Dcel) -> bool {
    let tw = t.twin(e);
    
    // 边界边总是合法的
    if t.is_boundary(tw) {
        return true;
    }
    
    let p0 = pr.position;
    let pi = t.origin(e).position;
    let pj = t.origin(tw).position;
    let pk = t.origin(t.prev(tw)).position;

    // 计算边 pi-pj 的垂直平分线
    let p = Point::new(0.5 * (pi.x + pj.x), 0.5 * (pi.y + pj.y));  // 中点
    let r = Point::new(-(pj.y - pi.y), pj.x - pi.x);  // 垂直方向向量
    
    // 计算边 pi-p0 的垂直平分线
    let q = Point::new(0.5 * (pi.x + p0.x), 0.5 * (pi.y + p0.y));  // 中点
    let s = Point::new(-(p0.y - pi.y), p0.x - pi.x);  // 垂直方向向量

    // 两条垂直平分线的交点即为外接圆圆心
    match line_intersection(p, r, q, s) {
        None => false,  // 平行，不应该发生
        Some(center) => {
            // 计算外接圆半径的平方
            let dvx = p0.x - center.x;
            let dvy = p0.y - center.y;
            let crsq = dvx * dvx + dvy * dvy;
            
            // 计算对面顶点到圆心的距离平方
            let dkx = pk.x - center.x;
            let dky = pk.y - center.y;
            let distsq = dkx * dkx + dky * dky;
            
            // 如果对面顶点在圆外或圆上，边合法
            distsq >= crsq
        }
    }
}

/// 合法化边（边翻转）
///
/// 如果边不满足 Delaunay 性质，通过翻转边来恢复该性质。
/// 边翻转将两个相邻三角形的公共边替换为连接两个对面顶点的边。
///
/// # 算法流程
/// 1. 检查边是否合法
/// 2. 如果不合法，翻转边
/// 3. 递归检查翻转后产生的两条新边
///
/// # 边翻转示意
/// ```text
/// 翻转前:          翻转后:
///     k                k
///    /|\              / \
///   / | \            /   \
///  /  |  \          /     \
/// i---j---r   =>   i-------r
///  \  |  /          \     /
///   \ | /            \   /
///    \|/              \ /
///     l                l
/// ```
/// 边 i-j 被翻转为边 k-r
///
/// # 参数
/// * `pr` - 新插入的顶点
/// * `eij` - 待合法化的边
/// * `t` - DCEL 数据结构
///
/// # 参考来源
/// - M. Berg, "Computational geometry", Chapter 9
/// - 原始 C++ 实现: src/delaunay.cpp, legalizeEdge()
fn legalize_edge(pr: Vertex, eij: HalfEdge, t: &mut Dcel) {
    // 如果边已经合法，无需处理
    if is_edge_legal(pr, eij, t) {
        return;
    }

    // 获取边和顶点
    let ejr = t.next(eij);
    let eri = t.next(ejr);
    let eji = t.twin(eij);
    let eik = t.next(eji);
    let ekj = t.next(eik);

    let f1 = t.incident_face(eij);
    let f2 = t.incident_face(eji);

    let pi = t.origin(eij);
    let pj = t.origin(eji);
    let pk = t.origin(ekj);

    // 边翻转：eij -> erk, eji -> ekr
    let mut erk = eij;
    let mut ekr = eji;

    let mut ejr2 = ejr;
    let mut eri2 = eri;
    let mut eik2 = eik;
    let mut ekj2 = ekj;
    let mut f1 = f1;
    let mut f2 = f2;
    let mut pi2 = pi;
    let mut pj2 = pj;
    let mut pk2 = pk;
    let mut pr2 = pr;

    ejr2.incident_face = f2.id;
    ejr2.next = erk.id;
    ejr2.prev = ekj2.id;
    t.update_edge(ejr2);

    eri2.next = eik2.id;
    eri2.prev = ekr.id;
    t.update_edge(eri2);

    eik2.incident_face = f1.id;
    eik2.next = ekr.id;
    eik2.prev = eri2.id;
    t.update_edge(eik2);

    ekj2.next = ejr2.id;
    ekj2.prev = erk.id;
    t.update_edge(ekj2);

    f1.outer_component = ekr.id;
    t.update_face(f1);

    f2.outer_component = erk.id;
    t.update_face(f2);

    pi2.incident_edge = eik2.id;
    t.update_vertex(pi2);

    pj2.incident_edge = ejr2.id;
    t.update_vertex(pj2);

    pk2.incident_edge = ekr.id;
    t.update_vertex(pk2);

    pr2.incident_edge = erk.id;
    t.update_vertex(pr2);

    erk.origin = pr.id;
    erk.twin = ekr.id;
    erk.incident_face = f2.id;
    erk.next = ekj2.id;
    erk.prev = ejr2.id;
    t.update_edge(erk);

    ekr.origin = pk.id;
    ekr.twin = erk.id;
    ekr.incident_face = f1.id;
    ekr.next = eri2.id;
    ekr.prev = eik2.id;
    t.update_edge(ekr);

    let pr_now = t.vertex(pr.id);
    legalize_edge(pr_now, eik2, t);
    legalize_edge(pr_now, ekj2, t);
}

/// 清理超级三角形
///
/// 删除与超级三角形相关的所有三角形、边和顶点，
/// 只保留输入点形成的 Delaunay 三角剖分。
///
/// # 算法流程
/// 1. 标记超级三角形的 3 个顶点（前 3 个顶点）
/// 2. 标记包含这些顶点的所有三角形为无效
/// 3. 标记这些三角形的所有边为无效
/// 4. 重建索引，删除无效元素
/// 5. 更新所有引用关系
///
/// # 参数
/// * `t` - DCEL 数据结构
///
/// # 参考来源
/// - 原始 C++ 实现: src/delaunay.cpp, cleanup()
fn cleanup(t: &mut Dcel) {
    // 找到超级三角形的 3 个顶点（前 3 个顶点）
    if t.vertices.len() < 3 {
        return;
    }
    let super_ids = [t.vertices[0].id.id, t.vertices[1].id.id, t.vertices[2].id.id];

    // Mark invalid faces (containing super-triangle vertices)
    let invalid_faces: Vec<bool> = t.faces.iter().map(|f| {
        if !f.outer_component.is_valid() { return false; }
        let h = t.edges[f.outer_component.id as usize];
        let h2 = t.edges[h.next.id as usize];
        let h3 = t.edges[h2.next.id as usize];
        let v0 = h.origin.id;
        let v1 = h2.origin.id;
        let v2 = h3.origin.id;
        super_ids.contains(&v0) || super_ids.contains(&v1) || super_ids.contains(&v2)
    }).collect();

    // Mark invalid edges (incident to invalid face or whose twin is incident to invalid face on the other side)
    let invalid_edges: Vec<bool> = t.edges.iter().map(|e| {
        if !e.incident_face.is_valid() { return true; }
        invalid_faces[e.incident_face.id as usize]
    }).collect();

    // Remove super-triangle vertices
    let invalid_verts: Vec<bool> = t.vertices.iter().map(|v| {
        super_ids.contains(&v.id.id)
    }).collect();

    // Build new indices
    let mut new_vert_idx = vec![-1i32; t.vertices.len()];
    let mut cnt = 0i32;
    for (i, &inv) in invalid_verts.iter().enumerate() {
        if !inv {
            new_vert_idx[i] = cnt;
            cnt += 1;
        }
    }

    let mut new_edge_idx = vec![-1i32; t.edges.len()];
    cnt = 0;
    for (i, &inv) in invalid_edges.iter().enumerate() {
        if !inv {
            new_edge_idx[i] = cnt;
            cnt += 1;
        }
    }

    let mut new_face_idx = vec![-1i32; t.faces.len()];
    cnt = 0;
    for (i, &inv) in invalid_faces.iter().enumerate() {
        if !inv {
            new_face_idx[i] = cnt;
            cnt += 1;
        }
    }

    // Remap
    let remap_ref = |r: Ref, new_idx: &[i32]| -> Ref {
        if !r.is_valid() { return r; }
        let new = new_idx[r.id as usize];
        Ref::new(new)
    };

    let new_verts: Vec<Vertex> = t.vertices.iter().enumerate()
        .filter(|(i, _)| !invalid_verts[*i])
        .map(|(_, v)| {
            let mut v2 = *v;
            v2.id = Ref::new(new_vert_idx[v.id.id as usize]);
            v2.incident_edge = remap_ref(v.incident_edge, &new_edge_idx);
            v2
        }).collect();

    let new_edges: Vec<HalfEdge> = t.edges.iter().enumerate()
        .filter(|(i, _)| !invalid_edges[*i])
        .map(|(_, e)| {
            let mut e2 = *e;
            e2.id = Ref::new(new_edge_idx[e.id.id as usize]);
            e2.origin = remap_ref(e.origin, &new_vert_idx);
            e2.twin = remap_ref(e.twin, &new_edge_idx);
            e2.incident_face = remap_ref(e.incident_face, &new_face_idx);
            e2.next = remap_ref(e.next, &new_edge_idx);
            e2.prev = remap_ref(e.prev, &new_edge_idx);
            e2
        }).collect();

    let new_faces: Vec<Face> = t.faces.iter().enumerate()
        .filter(|(i, _)| !invalid_faces[*i])
        .map(|(_, f)| {
            let mut f2 = f.clone();
            f2.id = Ref::new(new_face_idx[f.id.id as usize]);
            f2.outer_component = remap_ref(f.outer_component, &new_edge_idx);
            f2
        }).collect();

    t.vertices = new_verts;
    t.edges = new_edges;
    t.faces = new_faces;
}
