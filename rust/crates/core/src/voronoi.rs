use crate::dcel::{Dcel, Face, HalfEdge, Point, Ref, Vertex};
use crate::geometry::line_intersection;

pub fn delaunay_to_voronoi(t: &Dcel) -> Dcel {
    let mut v = Dcel::new();
    let mut voronoi_vertex_to_face_table: Vec<usize> = Vec::new();
    create_voronoi_vertices(t, &mut v, &mut voronoi_vertex_to_face_table);

    let mut delaunay_face_to_vertex_table = vec![-1i32; t.faces.len()];
    for (i, &fidx) in voronoi_vertex_to_face_table.iter().enumerate() {
        delaunay_face_to_vertex_table[fidx] = i as i32;
    }

    // vertexEdges[vi] = list of (vj, edge_id)
    let mut vertex_edges: Vec<Vec<(usize, usize)>> = vec![Vec::new(); v.vertices.len()];
    init_vertex_edge_table(t, &mut v, &delaunay_face_to_vertex_table, &mut vertex_edges);
    init_vertex_incident_edges(&mut v, &vertex_edges);

    for vidx in 0..t.vertices.len() {
        let dv = t.vertices[vidx];
        if t.is_boundary_vertex_check(dv) {
            continue;
        }

        let edge_loop = get_voronoi_cell_edge_loop(dv, t, &v, &delaunay_face_to_vertex_table, &vertex_edges);
        init_voronoi_face_from_edge_loop(&edge_loop, &mut v, &mut vertex_edges);
    }

    v
}

fn compute_voronoi_vertex(t: &Dcel, f: &Face) -> Point {
    let h = t.outer_component(f);
    let p0 = t.origin(h).position;
    let pi = t.origin(t.next(h)).position;
    let pj = t.origin(t.prev(h)).position;

    let p = Point::new(0.5 * (pi.x + pj.x), 0.5 * (pi.y + pj.y));
    let r = Point::new(-(pj.y - pi.y), pj.x - pi.x);
    let q = Point::new(0.5 * (pi.x + p0.x), 0.5 * (pi.y + p0.y));
    let s = Point::new(-(p0.y - pi.y), p0.x - pi.x);

    match line_intersection(p, r, q, s) {
        None => p0,
        Some(center) => center,
    }
}

fn create_voronoi_vertices(t: &Dcel, v: &mut Dcel, vertex_to_face: &mut Vec<usize>) {
    for i in 0..t.faces.len() {
        let f = &t.faces[i];
        if !f.outer_component.is_valid() {
            continue;
        }
        let p = compute_voronoi_vertex(t, f);
        v.create_vertex(p);
        vertex_to_face.push(i);
    }
}

fn init_vertex_edge_table(
    t: &Dcel,
    v: &mut Dcel,
    delaunay_face_to_vertex: &[i32],
    vertex_edges: &mut Vec<Vec<(usize, usize)>>,
) {
    for vidx in 0..t.vertices.len() {
        let dv = t.vertices[vidx];
        if t.is_boundary_vertex_check(dv) {
            continue;
        }

        let incident_faces = t.get_incident_faces(dv);

        for fidx in 0..incident_faces.len() {
            let fi = &incident_faces[fidx];
            let fj = if fidx == 0 { incident_faces.last().unwrap() } else { &incident_faces[fidx - 1] };

            let refi = delaunay_face_to_vertex[fi.id.id as usize];
            let refj = delaunay_face_to_vertex[fj.id.id as usize];
            if refi < 0 || refj < 0 { continue; }
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

fn init_vertex_incident_edges(v: &mut Dcel, vertex_edges: &Vec<Vec<(usize, usize)>>) {
    for i in 0..vertex_edges.len() {
        if vertex_edges[i].is_empty() { continue; }
        let edge_id = vertex_edges[i][0].1;
        let mut vi = v.vertices[i];
        vi.incident_edge = Ref::new(edge_id as i32);
        v.update_vertex(vi);
    }
}

fn get_voronoi_cell_edge_loop(
    delaunay_vertex: Vertex,
    t: &Dcel,
    v: &Dcel,
    delaunay_face_to_vertex: &[i32],
    vertex_edges: &Vec<Vec<(usize, usize)>>,
) -> Vec<HalfEdge> {
    let incident_faces = t.get_incident_faces(delaunay_vertex);
    let mut edge_loop = Vec::new();

    for fidx in 0..incident_faces.len() {
        let fi = &incident_faces[fidx];
        let fj = if fidx == 0 { incident_faces.last().unwrap() } else { &incident_faces[fidx - 1] };

        let refi = delaunay_face_to_vertex[fi.id.id as usize];
        let refj = delaunay_face_to_vertex[fj.id.id as usize];
        if refi < 0 || refj < 0 { continue; }
        let refi = refi as usize;
        let refj = refj as usize;

        // Find edge from vi to vj
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

fn init_voronoi_face_from_edge_loop(
    edge_loop: &[HalfEdge],
    v: &mut Dcel,
    vertex_edges: &mut Vec<Vec<(usize, usize)>>,
) {
    if edge_loop.is_empty() { return; }

    let mut cell_face = v.create_face();
    cell_face.outer_component = edge_loop[0].id;
    v.update_face(cell_face);

    let n = edge_loop.len();
    for hidx in 0..n {
        let eij = edge_loop[hidx];
        let ejk = if hidx == 0 { *edge_loop.last().unwrap() } else { edge_loop[hidx - 1] };
        let eri = if hidx == n - 1 { edge_loop[0] } else { edge_loop[hidx + 1] };

        let vi = v.origin(eij);
        let vj = v.origin(ejk);

        // Find or create twin eji (from vj to vi)
        let eji = find_or_create_twin(v, vertex_edges, vj.id.id as usize, vi.id.id as usize, eij.id);

        let mut eij2 = eij;
        eij2.origin = vi.id;
        eij2.twin = eji.id;
        eij2.incident_face = cell_face.id;
        eij2.next = ejk.id;
        eij2.prev = eri.id;
        v.update_edge(eij2);
    }
}

fn find_or_create_twin(
    v: &mut Dcel,
    vertex_edges: &mut Vec<Vec<(usize, usize)>>,
    from_vi: usize,
    to_vj: usize,
    twin_id: Ref,
) -> HalfEdge {
    // Check if edge from from_vi to to_vj already exists
    for &(dest, eid) in &vertex_edges[from_vi] {
        if dest == to_vj {
            return v.edges[eid];
        }
    }

    // Create new
    let mut eji = v.create_half_edge();
    eji.origin = Ref::new(from_vi as i32);
    eji.twin = twin_id;
    v.update_edge(eji);

    vertex_edges[from_vi].push((to_vj, eji.id.id as usize));
    eji
}
