use crate::dcel::Point;
use crate::extents2d::Extents2d;

pub struct SpatialPointGrid {
    dx: f64,
    offset: Point,
    isize: usize,
    jsize: usize,
    grid: Vec<Vec<Point>>,
}

impl SpatialPointGrid {
    pub fn new(points: &[Point], dx: f64) -> Self {
        if points.is_empty() {
            return SpatialPointGrid {
                dx,
                offset: Point::new(0.0, 0.0),
                isize: 0,
                jsize: 0,
                grid: Vec::new(),
            };
        }

        let ext = get_extents(points);
        let width = ext.maxx - ext.minx;
        let height = ext.maxy - ext.miny;
        let isize = (width / dx).ceil() as usize;
        let jsize = (height / dx).ceil() as usize;
        let total = isize.max(1) * jsize.max(1);
        let mut grid = vec![Vec::new(); total];

        let inv_dx = 1.0 / dx;
        for &p in points {
            let i = ((p.x - ext.minx) * inv_dx).floor() as usize;
            let j = ((p.y - ext.miny) * inv_dx).floor() as usize;
            let idx = i + isize * j;
            if idx < grid.len() {
                grid[idx].push(p);
            }
        }

        SpatialPointGrid {
            dx,
            offset: Point::new(ext.minx, ext.miny),
            isize: isize.max(1),
            jsize: jsize.max(1),
            grid,
        }
    }

    pub fn get_point_count(&self, extents: Extents2d) -> usize {
        if self.isize == 0 || self.jsize == 0 { return 0; }
        let inv_dx = 1.0 / self.dx;
        let mini = ((extents.minx - self.offset.x) * inv_dx).floor() as i64;
        let minj = ((extents.miny - self.offset.y) * inv_dx).floor() as i64;
        let maxi = ((extents.maxx - self.offset.x) * inv_dx).floor() as i64;
        let maxj = ((extents.maxy - self.offset.y) * inv_dx).floor() as i64;

        let mini = mini.max(0) as usize;
        let minj = minj.max(0) as usize;
        let maxi = (maxi as usize).min(self.isize - 1);
        let maxj = (maxj as usize).min(self.jsize - 1);

        let mut count = 0;
        for j in minj..=maxj {
            for i in mini..=maxi {
                let idx = i + self.isize * j;
                for &p in &self.grid[idx] {
                    if extents.contains_point(p) {
                        count += 1;
                    }
                }
            }
        }
        count
    }
}

fn get_extents(points: &[Point]) -> Extents2d {
    let mut e = Extents2d::new(points[0].x, points[0].y, points[0].x, points[0].y);
    for p in points {
        e.minx = e.minx.min(p.x);
        e.miny = e.miny.min(p.y);
        e.maxx = e.maxx.max(p.x);
        e.maxy = e.maxy.max(p.y);
    }
    e
}
