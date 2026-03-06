use crate::dcel::Point;

pub fn line_intersection(p: Point, r: Point, q: Point, s: Point) -> Option<Point> {
    let cross = r.x * s.y - r.y * s.x;
    let eps = 1e-9;
    if cross.abs() < eps {
        return None;
    }
    let vx = q.x - p.x;
    let vy = q.y - p.y;
    let t = (vx * s.y - vy * s.x) / cross;
    Some(Point::new(p.x + t * r.x, p.y + t * r.y))
}

pub fn line_segment_intersection(a: Point, b: Point, c: Point, d: Point) -> bool {
    let c1 = (d.y - a.y) * (c.x - a.x) > (c.y - a.y) * (d.x - a.x);
    let c2 = (d.y - b.y) * (c.x - b.x) > (c.y - b.y) * (d.x - b.x);
    let c3 = (c.y - a.y) * (b.x - a.x) > (b.y - a.y) * (c.x - a.x);
    let c4 = (d.y - a.y) * (b.x - a.x) > (b.y - a.y) * (d.x - a.x);
    (c1 != c2) && (c3 != c4)
}
