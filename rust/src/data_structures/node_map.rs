//! 节点映射数据结构
//!
//! NodeMap 是一个泛型容器，用于存储与 VertexMap 中顶点关联的数据。
//! 它提供了按顶点索引快速访问和修改数据的功能。
//!
//! # 在地图生成中的应用
//! NodeMap 用于存储各种顶点属性：
//! - 地形高度（elevation）
//! - 水分值（moisture）
//! - 河流流量（flux）
//! - 温度、生物群系等其他属性
//!
//! # 特殊功能（针对 f64 类型）
//! 对于 `NodeMap<f64>`，提供了额外的数值处理功能：
//! - 归一化（normalize）：将值缩放到 [0, 1] 范围
//! - 平滑（relax）：用邻居的平均值替换每个值
//! - 水平调整（set_level）：整体偏移所有值
//!
//! # 参考来源
//! - 原始 C++ 实现: src/nodemap.h

use super::dcel::{Dcel, Vertex};
use super::vertex_map::VertexMap;

/// 节点映射
///
/// 存储与 VertexMap 中顶点关联的数据。
/// 索引与 VertexMap 中的顶点索引一一对应。
pub struct NodeMap<T: Clone + Default> {
    /// 节点数据数组
    nodes: Vec<T>,
    /// 节点数量
    size: usize,
}

impl<T: Clone + Default> Clone for NodeMap<T> {
    fn clone(&self) -> Self {
        NodeMap {
            nodes: self.nodes.clone(),
            size: self.size,
        }
    }
}

impl<T: Clone + Default> NodeMap<T> {
    /// 创建指定大小的节点映射，所有值初始化为默认值
    pub fn new(size: usize) -> Self {
        NodeMap {
            nodes: vec![T::default(); size],
            size,
        }
    }

    /// 创建指定大小的节点映射，所有值初始化为指定值
    pub fn new_filled(size: usize, val: T) -> Self {
        NodeMap {
            nodes: vec![val; size],
            size,
        }
    }

    /// 获取节点数量
    pub fn size(&self) -> usize {
        self.size
    }

    /// 获取指定索引的节点值（不可变引用）
    pub fn get(&self, idx: usize) -> &T {
        &self.nodes[idx]
    }

    /// 获取指定索引的节点值（可变引用）
    pub fn get_mut(&mut self, idx: usize) -> &mut T {
        &mut self.nodes[idx]
    }

    /// 设置指定索引的节点值
    pub fn set(&mut self, idx: usize, val: T) {
        self.nodes[idx] = val;
    }

    /// 将所有节点值设置为指定值
    pub fn fill(&mut self, val: T) {
        for n in self.nodes.iter_mut() {
            *n = val.clone();
        }
    }
}

/// 针对 f64 类型的特殊方法
impl NodeMap<f64> {
    /// 获取最小值
    pub fn min_val(&self) -> f64 {
        self.nodes.iter().cloned().fold(f64::INFINITY, f64::min)
    }

    /// 获取最大值
    pub fn max_val(&self) -> f64 {
        self.nodes.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
    }

    /// 归一化到 [0, 1] 范围
    ///
    /// 将所有值线性缩放到 [0, 1] 范围。
    /// 最小值映射到 0，最大值映射到 1。
    pub fn normalize(&mut self) {
        let mn = self.min_val();
        let mx = self.max_val();
        let range = mx - mn;

        // 如果所有值相同，不进行归一化
        if range < 1e-12 {
            return;
        }

        for v in self.nodes.iter_mut() {
            *v = (*v - mn) / range;
        }
    }

    /// 圆滑处理
    ///
    /// 先归一化，然后对每个值取平方根。
    /// 这会使较小的值增大，较大的值减小，产生更平滑的分布。
    ///
    /// # 在地图生成中的应用
    /// 用于地形高度的平滑处理，避免过于陡峭的地形。
    pub fn round(&mut self) {
        self.normalize();
        for v in self.nodes.iter_mut() {
            *v = v.sqrt();
        }
    }

    /// 平滑处理（松弛）
    ///
    /// 将每个节点的值替换为其邻居的平均值。
    /// 这是一种简单的平滑滤波器，可以消除噪声和尖锐变化。
    ///
    /// # 算法原理
    /// 对于每个顶点 v：
    /// new_value(v) = average(value(neighbor) for neighbor in neighbors(v))
    ///
    /// # 在地图生成中的应用
    /// 用于平滑地形高度、水分值等，使地图看起来更自然。
    ///
    /// # 参数
    /// * `vertex_map` - 顶点映射（用于获取邻居关系）
    /// * `dcel` - DCEL 数据结构
    ///
    /// # 参考来源
    /// - 原始 C++ 实现: src/nodemap.h, relax()
    pub fn relax(&mut self, vertex_map: &VertexMap, dcel: &Dcel) {
        let mut averages = Vec::with_capacity(self.size);

        for i in 0..self.size {
            let v = vertex_map.vertices[i];
            let nbs = vertex_map.get_neighbour_indices(dcel, v);

            // 如果没有邻居，保持原值
            if nbs.is_empty() {
                averages.push(self.nodes[i]);
                continue;
            }

            // 计算邻居的平均值
            let sum: f64 = nbs.iter().map(|&nb| self.nodes[nb]).sum();
            averages.push(sum / nbs.len() as f64);
        }

        self.nodes = averages;
    }

    /// 整体偏移所有值
    ///
    /// 从所有值中减去指定的水平值。
    /// 用于调整地形的整体高度。
    ///
    /// # 参数
    /// * `level` - 要减去的值
    pub fn set_level(&mut self, level: f64) {
        for v in self.nodes.iter_mut() {
            *v -= level;
        }
    }

    /// 将水平调整到中位数
    ///
    /// 计算所有值的中位数，然后从所有值中减去该中位数。
    /// 这样可以使正值和负值的数量大致相等。
    ///
    /// # 在地图生成中的应用
    /// 用于调整地形高度，使海平面（0 值）位于合适的位置，
    /// 从而控制陆地和海洋的比例。
    pub fn set_level_to_median(&mut self) {
        if self.nodes.is_empty() {
            return;
        }

        // 排序以找到中位数
        let mut sorted = self.nodes.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let n = sorted.len();
        let median = if n % 2 == 0 {
            // 偶数个元素，取中间两个的平均值
            0.5 * (sorted[n / 2 - 1] + sorted[n / 2])
        } else {
            // 奇数个元素，取中间的元素
            sorted[n / 2]
        };

        self.set_level(median);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_default_filled() {
        let nm: NodeMap<f64> = NodeMap::new(5);
        assert_eq!(nm.size(), 5);
        for i in 0..5 {
            assert_eq!(*nm.get(i), 0.0);
        }
    }

    #[test]
    fn new_filled_creates_with_value() {
        let nm = NodeMap::new_filled(3, 42.0);
        assert_eq!(nm.size(), 3);
        assert_eq!(*nm.get(0), 42.0);
        assert_eq!(*nm.get(2), 42.0);
    }

    #[test]
    fn get_set_roundtrip() {
        let mut nm: NodeMap<i32> = NodeMap::new(3);
        nm.set(1, 99);
        assert_eq!(*nm.get(0), 0);
        assert_eq!(*nm.get(1), 99);
        assert_eq!(*nm.get(2), 0);
    }

    #[test]
    fn get_mut_allows_in_place_modification() {
        let mut nm = NodeMap::new_filled(2, 10.0);
        *nm.get_mut(0) += 5.0;
        assert_eq!(*nm.get(0), 15.0);
        assert_eq!(*nm.get(1), 10.0);
    }

    #[test]
    fn fill_overwrites_all() {
        let mut nm = NodeMap::new_filled(3, 1.0);
        nm.fill(7.0);
        for i in 0..3 {
            assert_eq!(*nm.get(i), 7.0);
        }
    }

    #[test]
    fn min_max_val() {
        let mut nm = NodeMap::new(4);
        nm.set(0, 3.0);
        nm.set(1, -1.0);
        nm.set(2, 5.0);
        nm.set(3, 2.0);
        assert_eq!(nm.min_val(), -1.0);
        assert_eq!(nm.max_val(), 5.0);
    }

    #[test]
    fn normalize_maps_to_zero_one() {
        let mut nm = NodeMap::new(3);
        nm.set(0, 10.0);
        nm.set(1, 20.0);
        nm.set(2, 30.0);
        nm.normalize();
        assert!((*nm.get(0) - 0.0).abs() < 1e-12);
        assert!((*nm.get(1) - 0.5).abs() < 1e-12);
        assert!((*nm.get(2) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn normalize_uniform_values_no_change() {
        let mut nm = NodeMap::new_filled(3, 5.0);
        nm.normalize();
        // Uniform values stay as-is (range < epsilon)
        assert_eq!(*nm.get(0), 5.0);
    }

    #[test]
    fn round_applies_sqrt_after_normalize() {
        let mut nm = NodeMap::new(3);
        nm.set(0, 0.0);
        nm.set(1, 0.25);
        nm.set(2, 1.0);
        nm.round();
        // After normalize: [0.0, 0.25, 1.0]
        // After sqrt:      [0.0, 0.5, 1.0]
        assert!((*nm.get(0)).abs() < 1e-12);
        assert!((*nm.get(1) - 0.5).abs() < 1e-12);
        assert!((*nm.get(2) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn set_level_subtracts_from_all() {
        let mut nm = NodeMap::new_filled(3, 10.0);
        nm.set_level(3.0);
        for i in 0..3 {
            assert_eq!(*nm.get(i), 7.0);
        }
    }

    #[test]
    fn set_level_to_median_centers_values() {
        let mut nm = NodeMap::new(5);
        nm.set(0, 1.0);
        nm.set(1, 2.0);
        nm.set(2, 3.0);
        nm.set(3, 4.0);
        nm.set(4, 5.0);
        nm.set_level_to_median();
        // median = 3.0, so values become [-2, -1, 0, 1, 2]
        assert!((*nm.get(2)).abs() < 1e-12);
        assert_eq!(*nm.get(0), -2.0);
        assert_eq!(*nm.get(4), 2.0);
    }

    #[test]
    fn clone_is_independent() {
        let mut nm = NodeMap::new_filled(2, 1.0);
        let cloned = nm.clone();
        nm.set(0, 99.0);
        assert_eq!(*cloned.get(0), 1.0);
    }
}
