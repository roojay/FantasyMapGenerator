use crate::dcel::{Dcel, Ref, Vertex};
use crate::extents2d::Extents2d;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VertexType {
    Edge,
    Interior,
}

pub struct VertexMap {
    pub vertices: Vec<Vertex>,
    pub edge: Vec<Vertex>,
    pub interior: Vec<Vertex>,
    vertex_id_to_map_index: Vec<i32>,
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

        VertexMap { vertices, edge, interior, vertex_id_to_map_index, vertex_types }
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
        if idx < 0 { return false; }
        self.vertex_types[idx as usize] == VertexType::Edge
    }

    pub fn is_interior_vertex(&self, v: Vertex) -> bool {
        let idx = self.get_vertex_index(v);
        if idx < 0 { return false; }
        self.vertex_types[idx as usize] == VertexType::Interior
    }

    pub fn get_neighbour_indices(&self, dcel: &Dcel, v: Vertex) -> Vec<usize> {
        let mut nbs = Vec::new();
        if !v.incident_edge.is_valid() { return nbs; }
        let h0 = dcel.incident_edge(v);
        let start = h0.id;
        let mut h = h0;
        loop {
            let tw = dcel.twin(h);
            let n = dcel.origin(tw);
            if self.is_vertex(n) {
                let idx = self.get_vertex_index(n);
                if idx >= 0 { nbs.push(idx as usize); }
            }
            h = dcel.next(tw);
            if h.id == start { break; }
        }
        nbs
    }
}

fn is_boundary_vertex(dcel: &Dcel, v: Vertex) -> bool {
    dcel.is_boundary_vertex_check(v)
}

fn get_vertex_type(dcel: &Dcel, v: Vertex, extents: Extents2d) -> VertexType {
    let h0 = dcel.incident_edge(v);
    let start = h0.id;
    let mut h = h0;
    let mut ncount = 0;
    loop {
        let tw = dcel.twin(h);
        let n = dcel.origin(tw);
        if extents.contains_point(n.position) && !is_boundary_vertex(dcel, n) {
            ncount += 1;
        }
        h = dcel.next(tw);
        if h.id == start { break; }
    }
    if ncount < 3 {
        VertexType::Edge
    } else {
        VertexType::Interior
    }
}
