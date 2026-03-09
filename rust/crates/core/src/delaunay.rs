use crate::dcel::{Dcel, Face, HalfEdge, Point, Ref, Vertex};
use crate::geometry::{line_intersection, line_segment_intersection};

pub fn triangulate(points: &mut Vec<Point>) -> Dcel {
    if points.is_empty() {
        return Dcel::new();
    }

    let mut t = init_triangulation(points);

    while let Some(p) = points.pop() {
        let f = locate_triangle_at_point(p, &t);
        if f.id.is_valid() {
            insert_point_into_triangulation(p, f, &mut t);
        }
    }

    cleanup(&mut t);
    t
}

fn get_super_triangle(points: &[Point]) -> (Point, Point, Point) {
    let eps = 1e-3;
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
    let expand = f64::max(0.1 * (maxx - minx), 0.1 * (maxy - miny));
    minx -= expand;
    miny -= 5.0 * expand;
    maxx += expand;
    maxy += expand;

    let p1x = 0.5 * (minx + maxx);
    let p1y = maxy + 0.5 * (maxy - miny);
    let p1 = Point::new(p1x, p1y);

    let m = (maxy - p1y) / (maxx - p1x);
    let p2x = (1.0 / m) * (miny - p1y + m * p1x);
    let p2 = Point::new(p2x, miny);

    let m2 = (maxy - p1y) / (minx - p1x);
    let p3x = (1.0 / m2) * (miny - p1y + m2 * p1x);
    let p3 = Point::new(p3x, miny);

    (p1, p2, p3)
}

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

fn is_point_inside_triangle(p: Point, f: &Face, t: &Dcel) -> bool {
    let h = t.outer_component(f);
    let p0 = t.origin(h).position;
    let h2 = t.next(h);
    let p1 = t.origin(h2).position;
    let p2 = t.origin(t.next(h2)).position;

    let area = 0.5 * (-p1.y * p2.x + p0.y * (-p1.x + p2.x) + p0.x * (p1.y - p2.y) + p1.x * p2.y);
    let s = 1.0 / (2.0 * area) * (p0.y * p2.x - p0.x * p2.y + (p2.y - p0.y) * p.x + (p0.x - p2.x) * p.y);
    let t_val = 1.0 / (2.0 * area) * (p0.x * p1.y - p0.y * p1.x + (p0.y - p1.y) * p.x + (p1.x - p0.x) * p.y);

    s >= 0.0 && t_val >= 0.0 && 1.0 - s - t_val >= 0.0
}

fn compute_triangle_centroid(f: &Face, t: &Dcel) -> Point {
    let h = t.outer_component(f);
    let p0 = t.origin(h).position;
    let p1 = t.origin(t.next(h)).position;
    let p2 = t.origin(t.prev(h)).position;
    let frac = 1.0 / 3.0;
    Point::new(frac * (p0.x + p1.x + p2.x), frac * (p0.y + p1.y + p2.y))
}

fn is_segment_intersecting_edge(p0: Point, p1: Point, h: HalfEdge, t: &Dcel) -> bool {
    let c = t.origin(h).position;
    let d = t.origin(t.twin(h)).position;
    line_segment_intersection(p0, p1, c, d)
}

fn locate_triangle_at_point(p: Point, t: &Dcel) -> Face {
    if t.faces.is_empty() {
        return Face::new();
    }
    // Start with face 0
    let mut f = t.face(Ref::new(0));
    let max_count = (2.0 * (t.faces.len() as f64).sqrt()) as i32;
    let mut count = 0;
    let mut face_history = [-1i32; 3];

    loop {
        if is_point_inside_triangle(p, &f, t) {
            return f;
        }
        let p0 = compute_triangle_centroid(&f, t);
        let mut neighbour_found = false;
        let h0 = t.outer_component(&f);
        let mut h = h0;
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

        face_history[2] = face_history[1];
        face_history[1] = face_history[0];
        face_history[0] = f.id.id;
        if face_history[0] == face_history[2] { break; }

        count += 1;
        if count > max_count { break; }
    }
    Face::new()
}

fn point_to_edge_distance(p0: Point, h: HalfEdge, t: &Dcel) -> f64 {
    let p1 = t.origin(h).position;
    let p2 = t.origin(t.twin(h)).position;
    let vx = p2.x - p1.x;
    let vy = p2.y - p1.y;
    let len = (vx * vx + vy * vy).sqrt();
    if len < 1e-12 { return f64::INFINITY; }
    ((vx * (p1.y - p0.y) - (p1.x - p0.x) * vy) / len).abs()
}

fn insert_point_into_triangulation(p: Point, f: Face, t: &mut Dcel) {
    let eps = 1e-9;
    let mut close_edge_count = 0;
    let mut close_edge = HalfEdge::new();

    let h0 = t.outer_component(&f);
    let mut h = h0;
    for _ in 0..3 {
        let dist = point_to_edge_distance(p, h, t);
        if dist < eps {
            close_edge = h;
            close_edge_count += 1;
            if close_edge_count == 2 {
                return;
            }
        }
        h = t.next(h);
    }

    if close_edge_count == 0 {
        insert_point_into_triangle(p, f, t);
    } else {
        insert_point_into_triangle_edge(p, f, close_edge, t);
    }
}

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

    let mut eij2 = eij;
    let mut ejk2 = ejk;
    let mut eki2 = eki;
    let mut eji2 = eji;
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

fn is_edge_legal(pr: Vertex, e: HalfEdge, t: &Dcel) -> bool {
    let tw = t.twin(e);
    if t.is_boundary(tw) {
        return true;
    }
    let p0 = pr.position;
    let pi = t.origin(e).position;
    let pj = t.origin(tw).position;
    let pk = t.origin(t.prev(tw)).position;

    let p = Point::new(0.5 * (pi.x + pj.x), 0.5 * (pi.y + pj.y));
    let r = Point::new(-(pj.y - pi.y), pj.x - pi.x);
    let q = Point::new(0.5 * (pi.x + p0.x), 0.5 * (pi.y + p0.y));
    let s = Point::new(-(p0.y - pi.y), p0.x - pi.x);

    match line_intersection(p, r, q, s) {
        None => false,
        Some(center) => {
            let dvx = p0.x - center.x;
            let dvy = p0.y - center.y;
            let crsq = dvx * dvx + dvy * dvy;
            let dkx = pk.x - center.x;
            let dky = pk.y - center.y;
            let distsq = dkx * dkx + dky * dky;
            distsq >= crsq
        }
    }
}

fn legalize_edge(pr: Vertex, eij: HalfEdge, t: &mut Dcel) {
    if is_edge_legal(pr, eij, t) {
        return;
    }

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

    // replacement: eij -> erk, eji -> ekr
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

fn cleanup(t: &mut Dcel) {
    // Find the 3 super-triangle vertices (first 3 vertices)
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
    let remap_ref = |r: crate::dcel::Ref, new_idx: &[i32]| -> crate::dcel::Ref {
        if !r.is_valid() { return r; }
        let new = new_idx[r.id as usize];
        crate::dcel::Ref::new(new)
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
