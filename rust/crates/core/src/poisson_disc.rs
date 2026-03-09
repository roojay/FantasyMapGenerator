use crate::dcel::Point;
use crate::extents2d::Extents2d;
use crate::rand::GlibcRand;

struct SampleGrid {
    bounds: Extents2d,
    width: usize,
    height: usize,
    dx: f64,
    grid: Vec<i32>,
}

impl SampleGrid {
    fn new(extents: Extents2d, cellsize: f64) -> Self {
        let bw = extents.maxx - extents.minx;
        let bh = extents.maxy - extents.miny;
        let width = (bw / cellsize).ceil() as usize;
        let height = (bh / cellsize).ceil() as usize;
        let grid = vec![-1i32; width * height];
        SampleGrid { bounds: extents, width, height, dx: cellsize, grid }
    }

    fn flat_index(&self, i: usize, j: usize) -> usize {
        i + j * self.width
    }

    fn get_sample(&self, i: i32, j: i32) -> i32 {
        if i < 0 || i >= self.width as i32 || j < 0 || j >= self.height as i32 {
            return -1;
        }
        self.grid[self.flat_index(i as usize, j as usize)]
    }

    fn set_sample(&mut self, i: usize, j: usize, s: i32) {
        let idx = self.flat_index(i, j);
        self.grid[idx] = s;
    }

    fn get_cell(&self, p: Point) -> (usize, usize) {
        let x = p.x - self.bounds.minx;
        let y = p.y - self.bounds.miny;
        let i = (x / self.dx).floor() as usize;
        let j = (y / self.dx).floor() as usize;
        (i, j)
    }
}

fn random_double(rng: &mut GlibcRand, min: f64, max: f64) -> f64 {
    rng.random_double(min, max)
}

fn random_range(rng: &mut GlibcRand, min: usize, max: usize) -> usize {
    (min as i32 + (rng.rand() % (max as i32 - min as i32))) as usize
}

// NOTE: double-assignment for px matches C++ behavior where randomDouble() is called twice
// for x coordinate, discarding first value to preserve RNG state compatibility.
fn random_point(rng: &mut GlibcRand, extents: &Extents2d) -> Point {
    let _px_discard = random_double(rng, extents.minx, extents.maxx); // consumed for RNG state
    let px = random_double(rng, extents.minx, extents.maxx);
    let py = random_double(rng, extents.miny, extents.maxy);
    Point::new(px, py)
}

fn random_disc_point(rng: &mut GlibcRand, center: Point, r: f64) -> Point {
    let angle = random_double(rng, 0.0, 2.0 * std::f64::consts::PI);
    let nx = angle.sin();
    let ny = angle.cos();
    let rl = random_double(rng, r, 2.0 * r);
    Point::new(center.x + nx * rl, center.y + ny * rl)
}

fn is_sample_valid(p: Point, r: f64, points: &[Point], grid: &SampleGrid) -> bool {
    let (gi, gj) = grid.get_cell(p);
    if grid.get_sample(gi as i32, gj as i32) != -1 {
        return false;
    }

    let mini = (gi as i32 - 2).max(0);
    let minj = (gj as i32 - 2).max(0);
    let maxi = (gi as i32 + 2).min(grid.width as i32 - 1);
    let maxj = (gj as i32 + 2).min(grid.height as i32 - 1);

    let rsq = r * r;
    for j in minj..=maxj {
        for i in mini..=maxi {
            let sid = grid.get_sample(i, j);
            if sid == -1 { continue; }
            let o = points[sid as usize];
            let dx = p.x - o.x;
            let dy = p.y - o.y;
            if dx * dx + dy * dy < rsq {
                return false;
            }
        }
    }
    true
}

fn find_disc_point(
    rng: &mut GlibcRand,
    center: Point,
    r: f64,
    k: usize,
    points: &[Point],
    grid: &SampleGrid,
) -> Option<Point> {
    for _ in 0..k {
        let sample = random_disc_point(rng, center, r);
        if !grid.bounds.contains_xy(sample.x, sample.y) {
            continue;
        }
        if is_sample_valid(sample, r, points, grid) {
            return Some(sample);
        }
    }
    None
}

pub fn generate_samples(rng: &mut GlibcRand, bounds: Extents2d, r: f64, k: usize) -> Vec<Point> {
    let dx = r / 2.0f64.sqrt();
    let mut grid = SampleGrid::new(bounds, dx);

    let seed = random_point(rng, &bounds);
    let mut points = vec![seed];
    let mut active_list: Vec<usize> = vec![0];

    let (gi, gj) = grid.get_cell(seed);
    grid.set_sample(gi, gj, 0);

    while !active_list.is_empty() {
        let rand_idx = random_range(rng, 0, active_list.len());
        let pidx = active_list[rand_idx];
        let p = points[pidx];

        match find_disc_point(rng, p, r, k, &points, &grid) {
            None => {
                active_list.remove(rand_idx);
            }
            Some(new_point) => {
                let new_idx = points.len();
                active_list.push(new_idx);
                points.push(new_point);
                let (ni, nj) = grid.get_cell(new_point);
                grid.set_sample(ni, nj, new_idx as i32);
            }
        }
    }

    points
}
