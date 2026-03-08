//! 顶点映射数据结构
//!
//! VertexMap 用于管理和分类 Voronoi 图中的顶点。
//! 它将顶点分为两类：边界顶点和内部顶点。
//!
//! # 顶点分类
//! - 边界顶点（Edge）：位于地图边界附近，邻居数量少于 3 个
//! - 内部顶点（Interior）：位于地图内部，有 3 个或更多邻居
//!
//! # 在地图生成中的应用
//! 顶点分类用于：
//! 1. 地形生成：内部顶点参与地形计算，边界顶点保持固定
//! 2. 河流生成：只在内部顶点之间生成河流
//! 3. 城市放置：优先在内部顶点放置城市
//!
//! # 参考来源
//! - 原始 C++ 实现: src/vertexmap.h, src/vertexmap.cpp

use super::dcel::{Dcel, Ref, Vertex};
use super::extents2d::Extents2d;

/// 顶点类型
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VertexType {
    /// 边界顶点：位于地图边界附近
    Edge,
    /// 内部顶点：位于地图内部
    Interior,
}

/// 顶点映射
///
/// 管理 Voronoi 图中的顶点，提供快速查询和分类功能。
pub struct VertexMap {
    /// 所有有效顶点（在边界内且非边界顶点）
    pub vertices: Vec<Vertex>,
    /// 边界顶点列表
    pub edge: Vec<Vertex>,
    /// 内部顶点列表
    pub interior: Vec<Vertex>,
    /// 顶点 ID → 映射索引的转换表
    vertex_id_to_map_index: Vec<i32>,
    /// 顶点类型数组
    vertex_types: Vec<VertexType>,
}

impl VertexMap {
    pub fn new_empty() -> Self {
        VertexMap {
            vertices: Vec::new(),
            edge: Vec::new(),
            interior: Vec::new(),
            vertex_id_to_map_index: Vec::new(),
            vertex_types: Vec::new(),
        }
    }

    pub fn new(dcel: &Dcel, extents: Extents2d) -> Self {
        let n = dcel.vertices.len();
        let mut vertex_id_to_map_index = vec![-1i32; n];
        let mut vertices = Vec::new();
        let mut edge = Vec::new();
        let mut interior = Vec::new();
        let mut vertex_types = Vec::new();

        for i in 0..n {
            let v = dcel.vertices[i];
            if !extents.contains_point(v.position) || is_boundary_vertex(dcel, v) {
                continue;
            }

            vertices.push(v);
            vertex_id_to_map_index[v.id.id as usize] = (vertices.len() - 1) as i32;

            let vtype = get_vertex_type(dcel, v, extents);
            vertex_types.push(vtype);
            match vtype {
                VertexType::Interior => interior.push(v),
                VertexType::Edge => edge.push(v),
            }
        }

        VertexMap {
            vertices,
            edge,
            interior,
            vertex_id_to_map_index,
            vertex_types,
        }
    }

    pub fn size(&self) -> usize {
        self.vertices.len()
    }

    pub fn get_vertex_index(&self, v: Vertex) -> i32 {
        if v.id.id < 0 || v.id.id as usize >= self.vertex_id_to_map_index.len() {
            return -1;
        }
        self.vertex_id_to_map_index[v.id.id as usize]
    }

    pub fn get_vertex_index_by_id(&self, id: i32) -> i32 {
        if id < 0 || id as usize >= self.vertex_id_to_map_index.len() {
            return -1;
        }
        self.vertex_id_to_map_index[id as usize]
    }

    pub fn is_vertex(&self, v: Vertex) -> bool {
        self.get_vertex_index(v) != -1
    }

    pub fn is_edge_vertex(&self, v: Vertex) -> bool {
        let idx = self.get_vertex_index(v);
        if idx < 0 {
            return false;
        }
        self.vertex_types[idx as usize] == VertexType::Edge
    }

    pub fn is_interior_vertex(&self, v: Vertex) -> bool {
        let idx = self.get_vertex_index(v);
        if idx < 0 {
            return false;
        }
        self.vertex_types[idx as usize] == VertexType::Interior
    }

    pub fn get_neighbour_indices(&self, dcel: &Dcel, v: Vertex) -> Vec<usize> {
        let mut nbs = Vec::new();
        if !v.incident_edge.is_valid() {
            return nbs;
        }
        let h0 = dcel.incident_edge(v);
        let start = h0.id;
        let mut h = h0;
        loop {
            let tw = dcel.twin(h);
            let n = dcel.origin(tw);
            if self.is_vertex(n) {
                let idx = self.get_vertex_index(n);
                if idx >= 0 {
                    nbs.push(idx as usize);
                }
            }
            h = dcel.next(tw);
            if h.id == start {
                break;
            }
        }
        nbs
    }
}

/// 判断顶点是否为 Voronoi 边界顶点
///
/// Voronoi 边界顶点是指那些关联边中有无效孪生边的顶点。
/// 这些顶点位于 Voronoi 图的外边界上。
fn is_boundary_vertex(dcel: &Dcel, v: Vertex) -> bool {
    dcel.is_boundary_vertex_check(v)
}

/// 确定顶点类型（边界/内部）
///
/// 根据顶点在边界内的有效邻居数量来分类：
/// - 邻居数 < 3：边界顶点
/// - 邻居数 >= 3：内部顶点
///
/// # 算法原理
/// Voronoi 图中，每个顶点理论上应该有 3 个邻居（因为是三角剖分的对偶）。
/// 如果邻居数少于 3，说明该顶点靠近地图边界。
///
/// # 参数
/// * `dcel` - DCEL 数据结构
/// * `v` - 待分类的顶点
/// * `extents` - 地图边界
///
/// # 返回
/// 顶点类型（Edge 或 Interior）
///
/// # 参考来源
/// - 原始 C++ 实现: src/vertexmap.cpp, getVertexType()
fn get_vertex_type(dcel: &Dcel, v: Vertex, extents: Extents2d) -> VertexType {
    let h0 = dcel.incident_edge(v);
    let start = h0.id;
    let mut h = h0;
    let mut ncount = 0;

    // 统计在边界内的有效邻居数量
    loop {
        let tw = dcel.twin(h);
        let n = dcel.origin(tw);

        // 只计数在边界内且非边界顶点的邻居
        if extents.contains_point(n.position) && !is_boundary_vertex(dcel, n) {
            ncount += 1;
        }

        h = dcel.next(tw);
        if h.id == start {
            break;
        }
    }

    // 邻居数少于 3 的是边界顶点
    if ncount < 3 {
        VertexType::Edge
    } else {
        VertexType::Interior
    }
}
