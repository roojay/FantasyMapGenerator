//! 双连边表 (Doubly Connected Edge List, DCEL) 数据结构
//!
//! DCEL 是一种用于表示平面细分的数据结构，广泛应用于计算几何中。
//! 它能够高效地表示和遍历平面图的拓扑结构。
//!
//! # 参考来源
//! - M. Berg, "Computational geometry", Berlin: Springer, 2000.
//! - 原始 C++ 实现: src/dcel.h

/// 二维空间中的点
///
/// 表示平面上的一个位置，用于存储顶点坐标。
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    /// X 坐标
    pub x: f64,
    /// Y 坐标
    pub y: f64,
}

impl Point {
    /// 创建一个新的点
    ///
    /// # 参数
    /// * `x` - X 坐标
    /// * `y` - Y 坐标
    pub fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }
}

/// 引用类型，用于在 DCEL 中索引顶点、半边和面
///
/// 使用整数 ID 来引用 DCEL 中的元素，而不是直接使用指针。
/// ID 为 -1 表示无效引用（类似于 null 指针）。
///
/// # 设计原因
/// 使用整数索引而非指针的原因：
/// 1. 避免 Rust 的借用检查器问题
/// 2. 便于序列化和持久化
/// 3. 更容易进行数组索引操作
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Ref {
    /// 元素的索引 ID，-1 表示无效引用
    pub id: i32,
}

impl Ref {
    /// 创建一个新的引用
    pub fn new(id: i32) -> Self {
        Ref { id }
    }

    /// 创建一个无效引用
    pub fn invalid() -> Self {
        Ref { id: -1 }
    }

    /// 检查引用是否有效
    pub fn is_valid(&self) -> bool {
        self.id >= 0
    }
}

/// 半边 (Half-Edge)
///
/// DCEL 的核心组成部分。每条边被分解为两条方向相反的半边。
/// 半边存储了丰富的拓扑信息，使得图的遍历非常高效。
///
/// # DCEL 结构说明
/// 在 DCEL 中，每条边由两条半边表示：
/// - 一条半边从顶点 A 指向顶点 B
/// - 另一条半边（twin）从顶点 B 指向顶点 A
///
/// 这种表示方法使得我们可以：
/// 1. 快速访问边的两侧面
/// 2. 高效地遍历面的边界
/// 3. 轻松地在图中导航
///
/// # 参考来源
/// - M. Berg, "Computational geometry", Chapter 2
#[derive(Clone, Copy, Debug, Default)]
pub struct HalfEdge {
    /// 半边的起点顶点
    pub origin: Ref,

    /// 对偶半边（方向相反的另一条半边）
    pub twin: Ref,

    /// 半边左侧的面
    pub incident_face: Ref,

    /// 沿着面边界的下一条半边
    pub next: Ref,

    /// 沿着面边界的上一条半边
    pub prev: Ref,

    /// 半边自身的 ID
    pub id: Ref,
}

impl HalfEdge {
    /// 创建一个新的半边，所有引用初始化为无效
    pub fn new() -> Self {
        HalfEdge {
            origin: Ref::invalid(),
            twin: Ref::invalid(),
            incident_face: Ref::invalid(),
            next: Ref::invalid(),
            prev: Ref::invalid(),
            id: Ref::invalid(),
        }
    }
}

/// 面 (Face)
///
/// 表示平面细分中的一个面（多边形区域）。
/// 在 Voronoi 图中，每个面对应原始 Delaunay 三角剖分中的一个顶点。
///
/// # 边界表示
/// 面的边界由一系列首尾相连的半边组成。
/// `outer_component` 指向边界上的任意一条半边，
/// 通过 `next` 指针可以遍历整个边界。
#[derive(Clone, Copy, Debug, Default)]
pub struct Face {
    /// 外边界的一条半边（任意一条即可）
    pub outer_component: Ref,

    /// 面自身的 ID
    pub id: Ref,
}

impl Face {
    /// 创建一个新的面，所有引用初始化为无效
    pub fn new() -> Self {
        Face {
            outer_component: Ref::invalid(),
            id: Ref::invalid(),
        }
    }
}

/// 顶点 (Vertex)
///
/// 表示平面细分中的一个顶点。
/// 在 Voronoi 图中，每个顶点是 Delaunay 三角剖分中三个点的外接圆圆心。
///
/// # 关联信息
/// 每个顶点存储一条关联的半边，通过这条半边可以访问：
/// - 所有从该顶点出发的边
/// - 所有与该顶点相邻的面
#[derive(Clone, Copy, Debug, Default)]
pub struct Vertex {
    /// 顶点在平面上的位置
    pub position: Point,

    /// 从该顶点出发的任意一条半边
    pub incident_edge: Ref,

    /// 顶点自身的 ID
    pub id: Ref,
}

impl Vertex {
    /// 创建一个新的顶点
    ///
    /// # 参数
    /// * `x` - X 坐标
    /// * `y` - Y 坐标
    pub fn new(x: f64, y: f64) -> Self {
        Vertex {
            position: Point::new(x, y),
            incident_edge: Ref::invalid(),
            id: Ref::invalid(),
        }
    }
}

/// 双连边表 (Doubly Connected Edge List)
///
/// DCEL 是一种用于表示平面细分的数据结构，它存储了顶点、半边和面的集合，
/// 以及它们之间的拓扑关系。
///
/// # 数据结构优势
/// 1. **高效遍历**: 可以快速访问顶点的邻居、面的边界等
/// 2. **拓扑完整**: 完整保存了平面图的拓扑信息
/// 3. **双向导航**: 可以从任意元素导航到相关的其他元素
///
/// # 在地图生成中的应用
/// 在本项目中，DCEL 用于表示：
/// - Delaunay 三角剖分
/// - Voronoi 图（Delaunay 的对偶图）
///
/// Voronoi 图的顶点用作地图生成的不规则网格节点。
///
/// # 参考来源
/// - M. Berg, "Computational geometry", Berlin: Springer, 2000.
/// - 原始 C++ 实现: src/dcel.h, src/dcel.cpp
#[derive(Clone, Debug, Default)]
pub struct Dcel {
    /// 所有顶点的集合
    pub vertices: Vec<Vertex>,

    /// 所有半边的集合
    pub edges: Vec<HalfEdge>,

    /// 所有面的集合
    pub faces: Vec<Face>,
}

impl Dcel {
    /// 创建一个空的 DCEL
    pub fn new() -> Self {
        Dcel {
            vertices: Vec::new(),
            edges: Vec::new(),
            faces: Vec::new(),
        }
    }

    // ===================================
    // 创建元素的方法
    // ===================================

    /// 创建一个新顶点并添加到 DCEL 中
    ///
    /// # 参数
    /// * `p` - 顶点的位置
    ///
    /// # 返回
    /// 新创建的顶点，其 ID 已自动分配
    pub fn create_vertex(&mut self, p: Point) -> Vertex {
        let mut v = Vertex::new(p.x, p.y);
        v.id = Ref::new(self.vertices.len() as i32);
        self.vertices.push(v);
        v
    }

    /// 创建一个新半边并添加到 DCEL 中
    ///
    /// # 返回
    /// 新创建的半边，其 ID 已自动分配，其他字段为无效引用
    pub fn create_half_edge(&mut self) -> HalfEdge {
        let mut e = HalfEdge::new();
        e.id = Ref::new(self.edges.len() as i32);
        self.edges.push(e);
        e
    }

    /// 创建一个新面并添加到 DCEL 中
    ///
    /// # 返回
    /// 新创建的面，其 ID 已自动分配
    pub fn create_face(&mut self) -> Face {
        let mut f = Face::new();
        f.id = Ref::new(self.faces.len() as i32);
        self.faces.push(f);
        f
    }

    // ===================================
    // 访问元素的方法
    // ===================================

    /// 通过引用获取顶点
    pub fn vertex(&self, id: Ref) -> Vertex {
        self.vertices[id.id as usize]
    }

    /// 通过引用获取半边
    pub fn edge(&self, id: Ref) -> HalfEdge {
        self.edges[id.id as usize]
    }

    /// 通过引用获取面
    pub fn face(&self, id: Ref) -> Face {
        self.faces[id.id as usize]
    }

    // ===================================
    // 更新元素的方法
    // ===================================

    /// 更新顶点的信息
    ///
    /// # 注意
    /// 必须保证顶点的 ID 与其在数组中的位置一致
    pub fn update_vertex(&mut self, v: Vertex) {
        self.vertices[v.id.id as usize] = v;
    }

    /// 更新半边的信息
    pub fn update_edge(&mut self, e: HalfEdge) {
        self.edges[e.id.id as usize] = e;
    }

    /// 更新面的信息
    pub fn update_face(&mut self, f: Face) {
        self.faces[f.id.id as usize] = f;
    }

    // ===================================
    // 拓扑导航方法
    // ===================================

    /// 获取半边的起点顶点
    pub fn origin(&self, h: HalfEdge) -> Vertex {
        self.vertex(h.origin)
    }

    /// 获取半边的对偶半边（方向相反）
    pub fn twin(&self, h: HalfEdge) -> HalfEdge {
        self.edge(h.twin)
    }

    /// 获取半边的下一条半边（沿着面边界）
    pub fn next(&self, h: HalfEdge) -> HalfEdge {
        self.edge(h.next)
    }

    /// 获取半边的上一条半边（沿着面边界）
    pub fn prev(&self, h: HalfEdge) -> HalfEdge {
        self.edge(h.prev)
    }

    /// 获取面的外边界上的一条半边
    pub fn outer_component(&self, f: &Face) -> HalfEdge {
        self.edge(f.outer_component)
    }

    /// 获取从顶点出发的一条半边
    pub fn incident_edge(&self, v: Vertex) -> HalfEdge {
        self.edge(v.incident_edge)
    }

    /// 获取半边左侧的面
    pub fn incident_face(&self, h: HalfEdge) -> Face {
        self.face(h.incident_face)
    }

    /// 检查半边是否在边界上
    ///
    /// # 边界定义
    /// 如果半边的 incident_face 为 -1，则该半边在边界上
    /// （即该半边的左侧没有面）
    pub fn is_boundary(&self, h: HalfEdge) -> bool {
        h.incident_face.id == -1
    }

    // ===================================
    // 遍历和查询方法
    // ===================================

    /// 获取面的所有外边界半边
    ///
    /// 从 `outer_component` 开始，沿着 `next` 指针遍历，
    /// 直到回到起点，收集所有经过的半边。
    ///
    /// # 参数
    /// * `f` - 要查询的面
    ///
    /// # 返回
    /// 按顺序排列的边界半边列表
    pub fn get_outer_components(&self, f: &Face) -> Vec<HalfEdge> {
        let mut edges = Vec::new();
        let h0 = self.outer_component(f);
        let start = h0.id;
        let mut h = h0;
        loop {
            edges.push(h);
            h = self.next(h);
            if h.id == start {
                break;
            }
        }
        edges
    }

    /// 获取从顶点出发的所有半边
    ///
    /// 从 `incident_edge` 开始，通过 twin 和 next 指针环绕顶点一周，
    /// 收集所有从该顶点出发的半边。
    ///
    /// # 遍历方式
    /// 对于每条半边 h：
    /// 1. 记录 h
    /// 2. 跳到 h 的 twin（到达对面）
    /// 3. 跳到 twin 的 next（绕到下一条边）
    /// 4. 重复直到回到起点
    ///
    /// # 参数
    /// * `v` - 要查询的顶点
    ///
    /// # 返回
    /// 从该顶点出发的所有半边
    pub fn get_incident_edges(&self, v: Vertex) -> Vec<HalfEdge> {
        let mut edges = Vec::new();
        let h0 = self.incident_edge(v);
        let start = h0.id;
        let mut h = h0;
        loop {
            edges.push(h);
            let tw = self.twin(h);
            h = self.next(tw);
            if h.id == start {
                break;
            }
        }
        edges
    }

    /// 获取与顶点相邻的所有面
    ///
    /// 遍历从顶点出发的所有半边，收集每条半边左侧的面。
    /// 跳过边界半边（没有关联面的半边）。
    ///
    /// # 参数
    /// * `v` - 要查询的顶点
    ///
    /// # 返回
    /// 与该顶点相邻的所有面
    pub fn get_incident_faces(&self, v: Vertex) -> Vec<Face> {
        let mut faces = Vec::new();
        let h0 = self.incident_edge(v);
        let start = h0.id;
        let mut h = h0;
        loop {
            if !self.is_boundary(h) {
                faces.push(self.incident_face(h));
            }
            let tw = self.twin(h);
            h = self.next(tw);
            if h.id == start {
                break;
            }
        }
        faces
    }

    /// 检查顶点是否在边界上
    ///
    /// 如果顶点的任意一条关联半边是边界半边，则该顶点在边界上。
    /// 同时检查拓扑结构的完整性（twin 和 next 指针是否有效）。
    ///
    /// # 边界顶点的特征
    /// - 至少有一条关联半边没有左侧面
    /// - 或者拓扑结构不完整（缺少 twin 或 next）
    ///
    /// # 参数
    /// * `v` - 要检查的顶点
    ///
    /// # 返回
    /// 如果顶点在边界上返回 true
    pub fn is_boundary_vertex_check(&self, v: Vertex) -> bool {
        if !v.incident_edge.is_valid() {
            return true;
        }
        let h0 = self.incident_edge(v);
        let start = h0.id;
        let mut h = h0;
        let mut count = 0;
        loop {
            // 检查是否为边界半边
            if h.incident_face.id == -1 {
                return true;
            }
            // 检查拓扑完整性
            if !h.twin.is_valid() {
                return true;
            }
            let tw = self.twin(h);
            if !tw.next.is_valid() {
                return true;
            }
            h = self.next(tw);
            if h.id == start {
                break;
            }

            // 安全检查：防止无限循环
            count += 1;
            if count > 1000 {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_new_stores_coordinates() {
        let p = Point::new(3.0, 4.0);
        assert_eq!(p.x, 3.0);
        assert_eq!(p.y, 4.0);
    }

    #[test]
    fn point_default_is_zero() {
        let p = Point::default();
        assert_eq!(p.x, 0.0);
        assert_eq!(p.y, 0.0);
    }

    #[test]
    fn ref_valid_and_invalid() {
        let valid = Ref::new(5);
        assert!(valid.is_valid());
        assert_eq!(valid.id, 5);

        let invalid = Ref::invalid();
        assert!(!invalid.is_valid());
        assert_eq!(invalid.id, -1);

        let zero = Ref::new(0);
        assert!(zero.is_valid());
    }

    #[test]
    fn dcel_create_vertex() {
        let mut dcel = Dcel::new();
        let v = dcel.create_vertex(Point::new(1.0, 2.0));
        assert_eq!(v.id.id, 0);
        assert_eq!(v.position.x, 1.0);
        assert_eq!(v.position.y, 2.0);
        assert_eq!(dcel.vertices.len(), 1);
    }

    #[test]
    fn dcel_create_multiple_vertices() {
        let mut dcel = Dcel::new();
        let v0 = dcel.create_vertex(Point::new(0.0, 0.0));
        let v1 = dcel.create_vertex(Point::new(1.0, 0.0));
        let v2 = dcel.create_vertex(Point::new(0.0, 1.0));
        assert_eq!(v0.id.id, 0);
        assert_eq!(v1.id.id, 1);
        assert_eq!(v2.id.id, 2);
        assert_eq!(dcel.vertices.len(), 3);
    }

    #[test]
    fn dcel_create_half_edge_pairs() {
        let mut dcel = Dcel::new();
        let e0 = dcel.create_half_edge();
        let e1 = dcel.create_half_edge();
        assert_eq!(e0.id.id, 0);
        assert_eq!(e1.id.id, 1);
        assert_eq!(dcel.edges.len(), 2);
    }

    #[test]
    fn dcel_create_face() {
        let mut dcel = Dcel::new();
        let f = dcel.create_face();
        assert_eq!(f.id.id, 0);
        assert_eq!(dcel.faces.len(), 1);
    }

    #[test]
    fn dcel_lookup_by_ref() {
        let mut dcel = Dcel::new();
        let v = dcel.create_vertex(Point::new(5.0, 7.0));
        let looked_up = dcel.vertex(v.id);
        assert_eq!(looked_up.position.x, 5.0);
        assert_eq!(looked_up.position.y, 7.0);
    }

    #[test]
    fn dcel_update_vertex() {
        let mut dcel = Dcel::new();
        let mut v = dcel.create_vertex(Point::new(0.0, 0.0));
        v.position = Point::new(10.0, 20.0);
        dcel.update_vertex(v);
        let updated = dcel.vertex(v.id);
        assert_eq!(updated.position.x, 10.0);
        assert_eq!(updated.position.y, 20.0);
    }

    #[test]
    fn dcel_update_edge_twin_linkage() {
        let mut dcel = Dcel::new();
        let mut e0 = dcel.create_half_edge();
        let mut e1 = dcel.create_half_edge();
        e0.twin = e1.id;
        e1.twin = e0.id;
        dcel.update_edge(e0);
        dcel.update_edge(e1);

        let twin_of_0 = dcel.twin(dcel.edge(e0.id));
        assert_eq!(twin_of_0.id, e1.id);
        let twin_of_1 = dcel.twin(dcel.edge(e1.id));
        assert_eq!(twin_of_1.id, e0.id);
    }

    #[test]
    fn dcel_boundary_edge_detection() {
        let mut dcel = Dcel::new();
        let mut e = dcel.create_half_edge();
        // incident_face defaults to invalid (-1), so it's a boundary edge
        assert!(dcel.is_boundary(e));

        let f = dcel.create_face();
        e.incident_face = f.id;
        dcel.update_edge(e);
        assert!(!dcel.is_boundary(dcel.edge(e.id)));
    }

    /// Build a minimal valid triangle DCEL for testing traversal methods.
    fn build_triangle_dcel() -> Dcel {
        let mut d = Dcel::new();
        let v0 = d.create_vertex(Point::new(0.0, 0.0));
        let v1 = d.create_vertex(Point::new(1.0, 0.0));
        let v2 = d.create_vertex(Point::new(0.5, 1.0));

        // Inner half-edges (counter-clockwise around face)
        let mut e01 = d.create_half_edge(); // v0 -> v1
        let mut e12 = d.create_half_edge(); // v1 -> v2
        let mut e20 = d.create_half_edge(); // v2 -> v0

        // Outer half-edges (boundary, clockwise)
        let mut e10 = d.create_half_edge(); // v1 -> v0
        let mut e21 = d.create_half_edge(); // v2 -> v1
        let mut e02 = d.create_half_edge(); // v0 -> v2

        let f = d.create_face();

        // Set origins
        e01.origin = v0.id;
        e12.origin = v1.id;
        e20.origin = v2.id;
        e10.origin = v1.id;
        e21.origin = v2.id;
        e02.origin = v0.id;

        // Set twins
        e01.twin = e10.id;
        e10.twin = e01.id;
        e12.twin = e21.id;
        e21.twin = e12.id;
        e20.twin = e02.id;
        e02.twin = e20.id;

        // Set next/prev for inner face
        e01.next = e12.id;
        e12.next = e20.id;
        e20.next = e01.id;
        e01.prev = e20.id;
        e12.prev = e01.id;
        e20.prev = e12.id;

        // Set next/prev for outer boundary
        e10.next = e02.id;
        e02.next = e21.id;
        e21.next = e10.id;
        e10.prev = e21.id;
        e02.prev = e10.id;
        e21.prev = e02.id;

        // Set incident faces (inner edges belong to face, outer are boundary)
        e01.incident_face = f.id;
        e12.incident_face = f.id;
        e20.incident_face = f.id;
        // outer edges have no face (boundary)

        // Face outer component
        let mut f_up = f;
        f_up.outer_component = e01.id;
        d.update_face(f_up);

        // Vertex incident edges
        let mut v0_up = v0;
        v0_up.incident_edge = e01.id;
        let mut v1_up = v1;
        v1_up.incident_edge = e12.id;
        let mut v2_up = v2;
        v2_up.incident_edge = e20.id;
        d.update_vertex(v0_up);
        d.update_vertex(v1_up);
        d.update_vertex(v2_up);

        d.update_edge(e01);
        d.update_edge(e12);
        d.update_edge(e20);
        d.update_edge(e10);
        d.update_edge(e21);
        d.update_edge(e02);

        d
    }

    #[test]
    fn triangle_face_has_three_outer_components() {
        let d = build_triangle_dcel();
        let f = d.face(Ref::new(0));
        let edges = d.get_outer_components(&f);
        assert_eq!(edges.len(), 3);
    }

    #[test]
    fn triangle_vertex_incident_edges() {
        let d = build_triangle_dcel();
        // v0 should have edges going out from it
        let v0 = d.vertex(Ref::new(0));
        let edges = d.get_incident_edges(v0);
        assert_eq!(edges.len(), 2); // e01 and e02
    }

    #[test]
    fn triangle_vertex_incident_faces() {
        let d = build_triangle_dcel();
        let v0 = d.vertex(Ref::new(0));
        let faces = d.get_incident_faces(v0);
        // v0 touches the single inner face (boundary edges skipped)
        assert_eq!(faces.len(), 1);
        assert_eq!(faces[0].id.id, 0);
    }

    #[test]
    fn half_edge_new_defaults_to_invalid() {
        let e = HalfEdge::new();
        assert!(!e.origin.is_valid());
        assert!(!e.twin.is_valid());
        assert!(!e.incident_face.is_valid());
        assert!(!e.next.is_valid());
        assert!(!e.prev.is_valid());
        assert!(!e.id.is_valid());
    }
}
