#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Ref {
    pub id: i32,
}

impl Ref {
    pub fn new(id: i32) -> Self {
        Ref { id }
    }
    pub fn invalid() -> Self {
        Ref { id: -1 }
    }
    pub fn is_valid(&self) -> bool {
        self.id >= 0
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct HalfEdge {
    pub origin: Ref,
    pub twin: Ref,
    pub incident_face: Ref,
    pub next: Ref,
    pub prev: Ref,
    pub id: Ref,
}

impl HalfEdge {
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

#[derive(Clone, Copy, Debug, Default)]
pub struct Face {
    pub outer_component: Ref,
    pub id: Ref,
}

impl Face {
    pub fn new() -> Self {
        Face {
            outer_component: Ref::invalid(),
            id: Ref::invalid(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Vertex {
    pub position: Point,
    pub incident_edge: Ref,
    pub id: Ref,
}

impl Vertex {
    pub fn new(x: f64, y: f64) -> Self {
        Vertex {
            position: Point::new(x, y),
            incident_edge: Ref::invalid(),
            id: Ref::invalid(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Dcel {
    pub vertices: Vec<Vertex>,
    pub edges: Vec<HalfEdge>,
    pub faces: Vec<Face>,
}

impl Dcel {
    pub fn new() -> Self {
        Dcel { vertices: Vec::new(), edges: Vec::new(), faces: Vec::new() }
    }

    pub fn create_vertex(&mut self, p: Point) -> Vertex {
        let mut v = Vertex::new(p.x, p.y);
        v.id = Ref::new(self.vertices.len() as i32);
        self.vertices.push(v);
        v
    }

    pub fn create_half_edge(&mut self) -> HalfEdge {
        let mut e = HalfEdge::new();
        e.id = Ref::new(self.edges.len() as i32);
        self.edges.push(e);
        e
    }

    pub fn create_face(&mut self) -> Face {
        let mut f = Face::new();
        f.id = Ref::new(self.faces.len() as i32);
        self.faces.push(f);
        f
    }

    pub fn vertex(&self, id: Ref) -> Vertex {
        self.vertices[id.id as usize]
    }

    pub fn edge(&self, id: Ref) -> HalfEdge {
        self.edges[id.id as usize]
    }

    pub fn face(&self, id: Ref) -> Face {
        self.faces[id.id as usize]
    }

    pub fn update_vertex(&mut self, v: Vertex) {
        self.vertices[v.id.id as usize] = v;
    }

    pub fn update_edge(&mut self, e: HalfEdge) {
        self.edges[e.id.id as usize] = e;
    }

    pub fn update_face(&mut self, f: Face) {
        self.faces[f.id.id as usize] = f;
    }

    pub fn origin(&self, h: HalfEdge) -> Vertex {
        self.vertex(h.origin)
    }

    pub fn twin(&self, h: HalfEdge) -> HalfEdge {
        self.edge(h.twin)
    }

    pub fn next(&self, h: HalfEdge) -> HalfEdge {
        self.edge(h.next)
    }

    pub fn prev(&self, h: HalfEdge) -> HalfEdge {
        self.edge(h.prev)
    }

    pub fn outer_component(&self, f: &Face) -> HalfEdge {
        self.edge(f.outer_component)
    }

    pub fn incident_edge(&self, v: Vertex) -> HalfEdge {
        self.edge(v.incident_edge)
    }

    pub fn incident_face(&self, h: HalfEdge) -> Face {
        self.face(h.incident_face)
    }

    pub fn is_boundary(&self, h: HalfEdge) -> bool {
        h.incident_face.id == -1
    }

    pub fn get_outer_components(&self, f: &Face) -> Vec<HalfEdge> {
        let mut edges = Vec::new();
        let h0 = self.outer_component(f);
        let start = h0.id;
        let mut h = h0;
        loop {
            edges.push(h);
            h = self.next(h);
            if h.id == start { break; }
        }
        edges
    }

    pub fn get_incident_edges(&self, v: Vertex) -> Vec<HalfEdge> {
        let mut edges = Vec::new();
        let h0 = self.incident_edge(v);
        let start = h0.id;
        let mut h = h0;
        loop {
            edges.push(h);
            let tw = self.twin(h);
            h = self.next(tw);
            if h.id == start { break; }
        }
        edges
    }

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
            if h.id == start { break; }
        }
        faces
    }

    pub fn is_boundary_vertex_check(&self, v: Vertex) -> bool {
        if !v.incident_edge.is_valid() {
            return true;
        }
        let h0 = self.incident_edge(v);
        let start = h0.id;
        let mut h = h0;
        let mut count = 0;
        loop {
            if h.incident_face.id == -1 {
                return true;
            }
            if !h.twin.is_valid() {
                return true;
            }
            let tw = self.twin(h);
            if !tw.next.is_valid() {
                return true;
            }
            h = self.next(tw);
            if h.id == start { break; }
            count += 1;
            if count > 1000 { return true; } // safety guard
        }
        false
    }
}
