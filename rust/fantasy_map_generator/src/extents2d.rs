use crate::dcel::Point;

#[derive(Clone, Copy, Debug, Default)]
pub struct Extents2d {
    pub minx: f64,
    pub miny: f64,
    pub maxx: f64,
    pub maxy: f64,
}

impl Extents2d {
    pub fn new(minx: f64, miny: f64, maxx: f64, maxy: f64) -> Self {
        Extents2d { minx, miny, maxx, maxy }
    }

    pub fn contains_point(&self, p: Point) -> bool {
        p.x >= self.minx && p.x < self.maxx && p.y >= self.miny && p.y < self.maxy
    }

    pub fn contains_xy(&self, x: f64, y: f64) -> bool {
        x >= self.minx && x < self.maxx && y >= self.miny && y < self.maxy
    }
}
