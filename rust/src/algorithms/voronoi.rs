//! Voronoi 图生成算法
//!
//! Voronoi 图是 Delaunay 三角剖分的对偶图。
//! 对于平面上的一组点（称为种子点），Voronoi 图将平面划分为多个区域，
//! 每个区域内的所有点到某个种子点的距离最近。
//!
//! # Delaunay 与 Voronoi 的对偶关系
//! - Delaunay 三角剖分的每个顶点 → Voronoi 图的一个面（单元）
//! - Delaunay 三角剖分的每个三角形 → Voronoi 图的一个顶点
//! - Delaunay 三角剖分的每条边 → Voronoi 图的一条边
//!
//! # Voronoi 顶点的计算
//! Voronoi 图的顶点是 Delaunay 三角形的外接圆圆心。
//! 通过求解三个点的外接圆，可以得到 Voronoi 顶点的位置。
//!
//! # 在地图生成中的应用
//! Voronoi 图的顶点作为地图的不规则网格节点。
//! 每个节点恰好有三个邻居（因为 Voronoi 图是三角剖分的对偶）。
//! 这种结构非常适合用于地形生成和流体模拟。
//!
//! # 参考来源
//! - M. Berg, "Computational geometry", Berlin: Springer, 2000, Chapter 7
//! - M. O'Leary, "Generating fantasy maps", https://mewo2.com/notes/terrain/
//! - 原始 C++ 实现: src/voronoi.h, src/voronoi.cpp

use crate::data_structures::dcel::{Dcel, Face, HalfEdge, Point, Ref, Vertex};
use crate::data_structures::geometry::line_intersection;

/// 从 Delaunay 三角剖分生成 Voronoi 图
///
/// # 算法步骤
/// 1. 为每个 Delaunay 三角形创建一个 Voronoi 顶点（外接圆圆心）
/// 2. 为每条 Delaunay 边创建一条 Voronoi 边
/// 3. 为每个 Delaunay 顶点创建一个 Voronoi 面（单元）
/// 4. 连接 Voronoi 边，形成完整的 Voronoi 图
///
/// # 对偶关系的实现
/// - `voronoi_vertex_to_face_table`: Voronoi 顶点 → Delaunay 面
/// - `delaunay_face_to_vertex_table`: Delaunay 面 → Voronoi 顶点
/// - `vertex_edges`: 记录每个 Voronoi 顶点的关联边
///
/// # 参数
/// * `t` - Delaunay 三角剖分（DCEL 格式）
///
/// # 返回
/// Voronoi 图（DCEL 格式）
///
/// # 参考来源
/// - M. Berg, "Computational geometry", Chapter 7
/// - 原始 C++ 实现: src/voronoi.cpp, delaunayToVoronoi()
pub fn delaunay_to_voronoi(t: &Dcel) -> Dcel {
    let mut v = Dcel::new();

    // ===================================
    // 1. 创建 Voronoi 顶点
    // ===================================
    // 每个 Delaunay 三角形对应一个 Voronoi 顶点
    let mut voronoi_vertex_to_face_table: Vec<usize> = Vec::new();
    create_voronoi_vertices(t, &mut v, &mut voronoi_vertex_to_face_table);

    // 建立反向映射：Delaunay 面 → Voronoi 顶点
    let mut delaunay_face_to_vertex_table = vec![-1i32; t.faces.len()];
    for (i, &fidx) in voronoi_vertex_to_face_table.iter().enumerate() {
        delaunay_face_to_vertex_table[fidx] = i as i32;
    }

    // ===================================
    // 2. 创建 Voronoi 边
    // ===================================
    // vertex_edges[vi] = [(vj, edge_id), ...]
    // 记录从顶点 vi 到顶点 vj 的边的 ID
    let mut vertex_edges: Vec<Vec<(usize, usize)>> = vec![Vec::new(); v.vertices.len()];
    init_vertex_edge_table(t, &mut v, &delaunay_face_to_vertex_table, &mut vertex_edges);
    init_vertex_incident_edges(&mut v, &vertex_edges);

    // ===================================
    // 3. 创建 Voronoi 面（单元）
    // ===================================
    // 每个 Delaunay 顶点对应一个 Voronoi 面
    for vidx in 0..t.vertices.len() {
        let dv = t.vertices[vidx];

        // 跳过边界顶点（它们的 Voronoi 单元是无界的）
        if t.is_boundary_vertex_check(dv) {
            continue;
        }

        // 获取围绕该 Delaunay 顶点的 Voronoi 边
        let edge_loop =
            get_voronoi_cell_edge_loop(dv, t, &v, &delaunay_face_to_vertex_table, &vertex_edges);

        // 从边环创建 Voronoi 面
        init_voronoi_face_from_edge_loop(&edge_loop, &mut v, &mut vertex_edges);
    }

    v
}

/// 计算 Voronoi 顶点的位置
///
/// Voronoi 顶点是 Delaunay 三角形的外接圆圆心。
///
/// # 算法原理
/// 给定三角形的三个顶点 p0, pi, pj，外接圆圆心是：
/// 1. 边 pi-pj 的中垂线
/// 2. 边 pi-p0 的中垂线
/// 的交点。
///
/// 使用参数方程表示中垂线：
/// - 第一条中垂线: P + t*R，其中 P 是 pi-pj 的中点，R 是垂直向量
/// - 第二条中垂线: Q + u*S，其中 Q 是 pi-p0 的中点，S 是垂直向量
///
/// # 参数
/// * `t` - Delaunay 三角剖分
/// * `f` - Delaunay 三角形（面）
///
/// # 返回
/// 外接圆圆心的位置
///
/// # 参考来源
/// - 原始 C++ 实现: src/voronoi.cpp, computeVoronoiVertex()
fn compute_voronoi_vertex(t: &Dcel, f: &Face) -> Point {
    // 获取三角形的三个顶点
    let h = t.outer_component(f);
    let p0 = t.origin(h).position;
    let pi = t.origin(t.next(h)).position;
    let pj = t.origin(t.prev(h)).position;

    // 第一条中垂线：边 pi-pj 的中垂线
    let p = Point::new(0.5 * (pi.x + pj.x), 0.5 * (pi.y + pj.y)); // 中点
    let r = Point::new(-(pj.y - pi.y), pj.x - pi.x); // 垂直向量（旋转90度）

    // 第二条中垂线：边 pi-p0 的中垂线
    let q = Point::new(0.5 * (pi.x + p0.x), 0.5 * (pi.y + p0.y)); // 中点
    let s = Point::new(-(p0.y - pi.y), p0.x - pi.x); // 垂直向量

    // 计算两条中垂线的交点（外接圆圆心）
    match line_intersection(p, r, q, s) {
        None => p0, // 如果平行（理论上不应该发生），返回一个顶点
        Some(center) => center,
    }
}

/// 为每个 Delaunay 三角形创建对应的 Voronoi 顶点
///
/// # 参数
/// * `t` - Delaunay 三角剖分
/// * `v` - 正在构建的 Voronoi 图
/// * `vertex_to_face` - 输出：Voronoi 顶点索引 → Delaunay 面索引的映射
///
/// # 参考来源
/// - 原始 C++ 实现: src/voronoi.cpp, createVoronoiVertices()
fn create_voronoi_vertices(t: &Dcel, v: &mut Dcel, vertex_to_face: &mut Vec<usize>) {
    for i in 0..t.faces.len() {
        let f = &t.faces[i];

        // 跳过无效的面
        if !f.outer_component.is_valid() {
            continue;
        }

        // 计算外接圆圆心作为 Voronoi 顶点
        let p = compute_voronoi_vertex(t, f);
        v.create_vertex(p);

        // 记录映射关系
        vertex_to_face.push(i);
    }
}

/// 初始化顶点-边表
///
/// 为每对相邻的 Voronoi 顶点创建一条边。
/// 两个 Voronoi 顶点相邻，当且仅当它们对应的 Delaunay 三角形共享一条边。
///
/// # 参数
/// * `t` - Delaunay 三角剖分
/// * `v` - 正在构建的 Voronoi 图
/// * `delaunay_face_to_vertex` - Delaunay 面 → Voronoi 顶点的映射
/// * `vertex_edges` - 输出：每个顶点的关联边列表
///
/// # 参考来源
/// - 原始 C++ 实现: src/voronoi.cpp, initVertexEdgeTable()
fn init_vertex_edge_table(
    t: &Dcel,
    v: &mut Dcel,
    delaunay_face_to_vertex: &[i32],
    vertex_edges: &mut Vec<Vec<(usize, usize)>>,
) {
    // 遍历每个 Delaunay 顶点
    for vidx in 0..t.vertices.len() {
        let dv = t.vertices[vidx];

        // 跳过边界顶点
        if t.is_boundary_vertex_check(dv) {
            continue;
        }

        // 获取围绕该顶点的所有 Delaunay 三角形
        let incident_faces = t.get_incident_faces(dv);

        // 为每对相邻的三角形创建一条 Voronoi 边
        for fidx in 0..incident_faces.len() {
            let fi = &incident_faces[fidx];
            let fj = if fidx == 0 {
                incident_faces.last().unwrap()
            } else {
                &incident_faces[fidx - 1]
            };

            let refi = delaunay_face_to_vertex[fi.id.id as usize];
            let refj = delaunay_face_to_vertex[fj.id.id as usize];
            if refi < 0 || refj < 0 {
                continue;
            }
            let refi = refi as usize;
            let refj = refj as usize;

            let vi_pos = v.vertices[refi].position;
            let mut eij = v.create_half_edge();
            eij.origin = Ref::new(refi as i32);
            v.update_edge(eij);

            vertex_edges[refi].push((refj, eij.id.id as usize));
        }
    }
}

/// 初始化顶点的关联边
///
/// 为每个 Voronoi 顶点设置其 `incident_edge` 字段，
/// 指向从该顶点出发的任意一条半边。
///
/// # 参数
/// * `v` - 正在构建的 Voronoi 图
/// * `vertex_edges` - 每个顶点的关联边列表
///
/// # 参考来源
/// - 原始 C++ 实现: src/voronoi.cpp, initVertexIncidentEdges()
fn init_vertex_incident_edges(v: &mut Dcel, vertex_edges: &Vec<Vec<(usize, usize)>>) {
    for i in 0..vertex_edges.len() {
        if vertex_edges[i].is_empty() {
            continue;
        }

        // 使用第一条边作为关联边
        let edge_id = vertex_edges[i][0].1;
        let mut vi = v.vertices[i];
        vi.incident_edge = Ref::new(edge_id as i32);
        v.update_vertex(vi);
    }
}

/// 获取 Voronoi 单元的边环
///
/// 对于给定的 Delaunay 顶点，获取围绕其对应 Voronoi 单元的所有边。
/// 这些边按逆时针顺序排列，形成一个闭合的环。
///
/// # 算法原理
/// 1. 获取围绕 Delaunay 顶点的所有三角形（按逆时针顺序）
/// 2. 每对相邻的三角形对应一条 Voronoi 边
/// 3. 这些边首尾相连，形成 Voronoi 单元的边界
///
/// # 参数
/// * `delaunay_vertex` - Delaunay 顶点
/// * `t` - Delaunay 三角剖分
/// * `v` - Voronoi 图
/// * `delaunay_face_to_vertex` - Delaunay 面 → Voronoi 顶点的映射
/// * `vertex_edges` - 顶点-边表
///
/// # 返回
/// 按逆时针顺序排列的边环
///
/// # 参考来源
/// - 原始 C++ 实现: src/voronoi.cpp, getVoronoiCellEdgeLoop()
fn get_voronoi_cell_edge_loop(
    delaunay_vertex: Vertex,
    t: &Dcel,
    v: &Dcel,
    delaunay_face_to_vertex: &[i32],
    vertex_edges: &Vec<Vec<(usize, usize)>>,
) -> Vec<HalfEdge> {
    // 获取围绕 Delaunay 顶点的所有三角形（按逆时针顺序）
    let incident_faces = t.get_incident_faces(delaunay_vertex);
    let mut edge_loop = Vec::new();

    // 为每对相邻的三角形找到对应的 Voronoi 边
    for fidx in 0..incident_faces.len() {
        let fi = &incident_faces[fidx];
        let fj = if fidx == 0 {
            incident_faces.last().unwrap()
        } else {
            &incident_faces[fidx - 1]
        };

        // 获取对应的 Voronoi 顶点索引
        let refi = delaunay_face_to_vertex[fi.id.id as usize];
        let refj = delaunay_face_to_vertex[fj.id.id as usize];
        if refi < 0 || refj < 0 {
            continue;
        }
        let refi = refi as usize;
        let refj = refj as usize;

        // 找到从 vi 到 vj 的边
        let mut found_edge = HalfEdge::new();
        for &(dest, eid) in &vertex_edges[refi] {
            if dest == refj {
                found_edge = v.edges[eid];
                break;
            }
        }
        edge_loop.push(found_edge);
    }

    edge_loop
}

/// 从边环初始化 Voronoi 面
///
/// 将一组边连接起来，形成一个完整的 Voronoi 单元（面）。
/// 设置每条边的 next、prev、twin、incident_face 等字段。
///
/// # 算法流程
/// 1. 创建一个新的面
/// 2. 遍历边环中的每条边
/// 3. 为每条边设置 next 和 prev 指针
/// 4. 为每条边找到或创建其孪生边（twin）
/// 5. 设置所有边的 incident_face 指向新创建的面
///
/// # 参数
/// * `edge_loop` - 按逆时针顺序排列的边环
/// * `v` - Voronoi 图
/// * `vertex_edges` - 顶点-边表（用于查找和创建孪生边）
///
/// # 参考来源
/// - 原始 C++ 实现: src/voronoi.cpp, initVoronoiFaceFromEdgeLoop()
fn init_voronoi_face_from_edge_loop(
    edge_loop: &[HalfEdge],
    v: &mut Dcel,
    vertex_edges: &mut Vec<Vec<(usize, usize)>>,
) {
    if edge_loop.is_empty() {
        return;
    }

    // 创建新的 Voronoi 单元（面）
    let mut cell_face = v.create_face();
    cell_face.outer_component = edge_loop[0].id;
    v.update_face(cell_face);

    let n = edge_loop.len();

    // 连接边环中的所有边
    for hidx in 0..n {
        let eij = edge_loop[hidx];

        // 前一条边（逆时针方向）
        let ejk = if hidx == 0 {
            *edge_loop.last().unwrap()
        } else {
            edge_loop[hidx - 1]
        };

        // 后一条边（逆时针方向）
        let eri = if hidx == n - 1 {
            edge_loop[0]
        } else {
            edge_loop[hidx + 1]
        };

        let vi = v.origin(eij);
        let vj = v.origin(ejk);

        // 找到或创建孪生边 eji（从 vj 到 vi）
        let eji = find_or_create_twin(
            v,
            vertex_edges,
            vj.id.id as usize,
            vi.id.id as usize,
            eij.id,
        );

        // 更新边的所有字段
        let mut eij2 = eij;
        eij2.origin = vi.id;
        eij2.twin = eji.id;
        eij2.incident_face = cell_face.id;
        eij2.next = ejk.id;
        eij2.prev = eri.id;
        v.update_edge(eij2);
    }
}

/// 查找或创建孪生边
///
/// 在 DCEL 结构中，每条边都有一条方向相反的孪生边（twin）。
/// 此函数尝试查找已存在的孪生边，如果不存在则创建一条新的。
///
/// # 参数
/// * `v` - Voronoi 图
/// * `vertex_edges` - 顶点-边表
/// * `from_vi` - 起点顶点索引
/// * `to_vj` - 终点顶点索引
/// * `twin_id` - 原边的 ID（用于设置新边的 twin 字段）
///
/// # 返回
/// 从 from_vi 到 to_vj 的半边
///
/// # 参考来源
/// - 原始 C++ 实现: src/voronoi.cpp, findOrCreateTwin()
fn find_or_create_twin(
    v: &mut Dcel,
    vertex_edges: &mut Vec<Vec<(usize, usize)>>,
    from_vi: usize,
    to_vj: usize,
    twin_id: Ref,
) -> HalfEdge {
    // 检查从 from_vi 到 to_vj 的边是否已存在
    for &(dest, eid) in &vertex_edges[from_vi] {
        if dest == to_vj {
            return v.edges[eid];
        }
    }

    // 不存在，创建新边
    let mut eji = v.create_half_edge();
    eji.origin = Ref::new(from_vi as i32);
    eji.twin = twin_id;
    v.update_edge(eji);

    // 添加到顶点-边表
    vertex_edges[from_vi].push((to_vj, eji.id.id as usize));
    eji
}
