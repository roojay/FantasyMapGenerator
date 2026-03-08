//! Poisson 圆盘采样算法
//!
//! 生成一组随机点，保证任意两点之间的距离不小于指定的最小距离。
//! 这种采样方法产生的点分布比纯随机采样更均匀，避免了点的聚集。
//!
//! # 算法原理
//! 使用 Robert Bridson 的快速 Poisson 圆盘采样算法。
//! 算法通过维护一个活跃点列表和一个背景网格来加速邻域查询，
//! 时间复杂度为 O(n)，其中 n 是生成的点数。
//!
//! # 在地图生成中的应用
//! Poisson 采样用于生成 Voronoi 图的种子点。
//! 相比纯随机采样，Poisson 采样生成的 Voronoi 单元大小更均匀，
//! 使得生成的地图看起来更自然。
//!
//! # 参考来源
//! - R. Bridson, "Fast Poisson Disk Sampling in Arbitrary Dimensions",
//!   ACM SIGGRAPH 2007 Sketches Program, 2007.
//! - M. O'Leary, "Generating fantasy maps", https://mewo2.com/notes/terrain/
//! - 原始 C++ 实现: src/poissondiscsampler.h, src/poissondiscsampler.cpp

use crate::data_structures::dcel::Point;
use crate::data_structures::extents2d::Extents2d;
use crate::utils::rand::GlibcRand;

/// 背景网格，用于加速 Poisson 采样中的邻域查询
///
/// 将采样区域划分为网格，每个网格单元最多包含一个采样点。
/// 这样在检查新点是否满足最小距离要求时，只需要检查周围的几个单元，
/// 而不需要检查所有已生成的点。
///
/// # 网格单元大小
/// 单元边长设为 min_distance / √2，这样可以保证：
/// - 每个单元最多包含一个点
/// - 检查一个点周围 5×5 的单元就足够了
struct SampleGrid {
    /// 采样区域的边界
    bounds: Extents2d,

    /// 网格的宽度（单元数）
    width: usize,

    /// 网格的高度（单元数）
    height: usize,

    /// 单元的边长
    dx: f64,

    /// 网格数据，存储每个单元中的采样点索引（-1 表示空）
    grid: Vec<i32>,
}

impl SampleGrid {
    /// 创建一个新的背景网格
    ///
    /// # 参数
    /// * `extents` - 采样区域的边界
    /// * `cellsize` - 网格单元的边长
    fn new(extents: Extents2d, cellsize: f64) -> Self {
        let bw = extents.maxx - extents.minx;
        let bh = extents.maxy - extents.miny;

        // 计算需要多少个单元来覆盖整个区域
        let width = (bw / cellsize).ceil() as usize;
        let height = (bh / cellsize).ceil() as usize;

        // 初始化网格，-1 表示单元为空
        let grid = vec![-1i32; width * height];

        SampleGrid {
            bounds: extents,
            width,
            height,
            dx: cellsize,
            grid,
        }
    }

    /// 将二维索引转换为一维数组索引
    fn flat_index(&self, i: usize, j: usize) -> usize {
        i + j * self.width
    }

    /// 获取指定单元中的采样点索引
    ///
    /// # 参数
    /// * `i` - 单元的 x 索引
    /// * `j` - 单元的 y 索引
    ///
    /// # 返回
    /// 采样点的索引，如果单元为空或越界则返回 -1
    fn get_sample(&self, i: i32, j: i32) -> i32 {
        if i < 0 || i >= self.width as i32 || j < 0 || j >= self.height as i32 {
            return -1;
        }
        self.grid[self.flat_index(i as usize, j as usize)]
    }

    /// 在指定单元中设置采样点索引
    fn set_sample(&mut self, i: usize, j: usize, s: i32) {
        let idx = self.flat_index(i, j);
        self.grid[idx] = s;
    }

    /// 获取点所在的网格单元
    ///
    /// # 参数
    /// * `p` - 要查询的点
    ///
    /// # 返回
    /// 单元的 (i, j) 索引
    fn get_cell(&self, p: Point) -> (usize, usize) {
        let x = p.x - self.bounds.minx;
        let y = p.y - self.bounds.miny;
        let i = (x / self.dx).floor() as usize;
        let j = (y / self.dx).floor() as usize;
        (i, j)
    }
}

/// 生成指定范围内的随机浮点数
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
            if sid == -1 {
                continue;
            }
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

/// 生成 Poisson 圆盘采样点
///
/// 在指定的矩形区域内生成一组随机点，保证任意两点之间的距离
/// 不小于 `r`（最小距离）。
///
/// # 算法步骤
/// 1. 初始化背景网格，单元边长为 r/√2
/// 2. 选择一个随机起始点，加入活跃列表
/// 3. 当活跃列表不为空时：
///    a. 从活跃列表中随机选择一个点 P
///    b. 在 P 周围的环形区域（半径 r 到 2r）尝试生成 k 个候选点
///    c. 对每个候选点，检查其周围是否有距离小于 r 的已有点
///    d. 如果找到合法的候选点，将其加入点集和活跃列表
///    e. 如果 k 次尝试都失败，将 P 从活跃列表中移除
/// 4. 返回生成的所有点
///
/// # 为什么在环形区域生成候选点
/// 候选点必须距离 P 至少 r（满足最小距离要求），
/// 但不能太远（否则会在点集中产生大的空隙）。
/// 因此在半径 r 到 2r 的环形区域内生成候选点是最优的。
///
/// # 参数
/// * `rng` - 随机数生成器
/// * `bounds` - 采样区域的边界
/// * `r` - 点之间的最小距离
/// * `k` - 每个点周围尝试生成新点的次数（推荐值：30）
///
/// # 返回
/// 生成的点的集合
///
/// # 性能
/// 时间复杂度：O(n)，其中 n 是生成的点数
/// 空间复杂度：O(n)
///
/// # 参考来源
/// - R. Bridson, "Fast Poisson Disk Sampling in Arbitrary Dimensions",
///   ACM SIGGRAPH 2007 Sketches Program, 2007.
/// - 原始 C++ 实现: src/poissondiscsampler.cpp, generateSamples()
pub fn generate_samples(rng: &mut GlibcRand, bounds: Extents2d, r: f64, k: usize) -> Vec<Point> {
    // ===================================
    // 1. 初始化背景网格
    // ===================================
    // 单元边长为 r/√2，这样可以保证每个单元最多包含一个点
    let dx = r / 2.0f64.sqrt();
    let mut grid = SampleGrid::new(bounds, dx);

    // ===================================
    // 2. 生成起始点
    // ===================================
    let seed = random_point(rng, &bounds);
    let mut points = vec![seed];
    let mut active_list: Vec<usize> = vec![0];

    let (gi, gj) = grid.get_cell(seed);
    grid.set_sample(gi, gj, 0);

    // ===================================
    // 3. 主循环：从活跃点生成新点
    // ===================================
    while !active_list.is_empty() {
        // 随机选择一个活跃点
        let rand_idx = random_range(rng, 0, active_list.len());
        let pidx = active_list[rand_idx];
        let p = points[pidx];

        // 尝试在该点周围生成新点
        match find_disc_point(rng, p, r, k, &points, &grid) {
            None => {
                // k 次尝试都失败，将该点从活跃列表移除
                active_list.remove(rand_idx);
            }
            Some(new_point) => {
                // 找到合法的新点，加入点集和活跃列表
                let new_idx = points.len();
                active_list.push(new_idx);
                points.push(new_point);

                // 在网格中记录新点的位置
                let (ni, nj) = grid.get_cell(new_point);
                grid.set_sample(ni, nj, new_idx as i32);
            }
        }
    }

    points
}
