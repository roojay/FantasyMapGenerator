use crate::dcel::{Dcel, Vertex};
use crate::vertex_map::VertexMap;

/// A map storing values indexed by vertex positions in the VertexMap.
pub struct NodeMap<T: Clone + Default> {
    nodes: Vec<T>,
    size: usize,
}

impl<T: Clone + Default> Clone for NodeMap<T> {
    fn clone(&self) -> Self {
        NodeMap { nodes: self.nodes.clone(), size: self.size }
    }
}

impl<T: Clone + Default> NodeMap<T> {
    pub fn new(size: usize) -> Self {
        NodeMap { nodes: vec![T::default(); size], size }
    }

    pub fn new_filled(size: usize, val: T) -> Self {
        NodeMap { nodes: vec![val; size], size }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn get(&self, idx: usize) -> &T {
        &self.nodes[idx]
    }

    pub fn get_mut(&mut self, idx: usize) -> &mut T {
        &mut self.nodes[idx]
    }

    pub fn set(&mut self, idx: usize, val: T) {
        self.nodes[idx] = val;
    }

    pub fn fill(&mut self, val: T) {
        for n in self.nodes.iter_mut() {
            *n = val.clone();
        }
    }
}

impl NodeMap<f64> {
    pub fn min_val(&self) -> f64 {
        self.nodes.iter().cloned().fold(f64::INFINITY, f64::min)
    }

    pub fn max_val(&self) -> f64 {
        self.nodes.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
    }

    pub fn normalize(&mut self) {
        let mn = self.min_val();
        let mx = self.max_val();
        let range = mx - mn;
        if range < 1e-12 { return; }
        for v in self.nodes.iter_mut() {
            *v = (*v - mn) / range;
        }
    }

    pub fn round(&mut self) {
        self.normalize();
        for v in self.nodes.iter_mut() {
            *v = v.sqrt();
        }
    }

    pub fn relax(&mut self, vertex_map: &VertexMap, dcel: &Dcel) {
        let mut averages = Vec::with_capacity(self.size);
        for i in 0..self.size {
            let v = vertex_map.vertices[i];
            let nbs = vertex_map.get_neighbour_indices(dcel, v);
            if nbs.is_empty() {
                averages.push(self.nodes[i]);
                continue;
            }
            let sum: f64 = nbs.iter().map(|&nb| self.nodes[nb]).sum();
            averages.push(sum / nbs.len() as f64);
        }
        self.nodes = averages;
    }

    pub fn set_level(&mut self, level: f64) {
        for v in self.nodes.iter_mut() {
            *v -= level;
        }
    }

    pub fn set_level_to_median(&mut self) {
        if self.nodes.is_empty() { return; }
        let mut sorted = self.nodes.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = sorted.len();
        let median = if n % 2 == 0 {
            0.5 * (sorted[n / 2 - 1] + sorted[n / 2])
        } else {
            sorted[n / 2]
        };
        self.set_level(median);
    }
}
