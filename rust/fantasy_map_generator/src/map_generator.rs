#![allow(
    clippy::too_many_arguments,
    // Large map generator - not all helper methods are used from main, but
    // they constitute the full port and may be used in tests or future features
    dead_code,
    unused_variables
)]

use std::collections::VecDeque;
use serde_json::{json, Value};

use crate::dcel::{Dcel, Face, HalfEdge, Point, Ref, Vertex};
use crate::extents2d::Extents2d;
use crate::rand::GlibcRand;
use crate::vertex_map::VertexMap;
use crate::node_map::NodeMap;
use crate::font_face::{FontFace, TextExtents};
use crate::spatial_point_grid::SpatialPointGrid;
use crate::poisson_disc;
use crate::delaunay;
use crate::voronoi;

#[derive(Clone, Debug)]
struct City {
    city_name: String,
    territory_name: String,
    position: Point,
    face_id: usize,
    movement_costs: Vec<f64>,
}

#[derive(Clone, Debug)]
struct Town {
    town_name: String,
    position: Point,
    face_id: usize,
}

#[derive(Clone, Debug, Default)]
struct LabelCandidate {
    text: String,
    fontface: String,
    fontsize: i32,
    position: Point,
    extents: Extents2d,
    char_extents: Vec<Extents2d>,
    city_id: i32,

    orientation_score: f64,
    edge_score: f64,
    marker_score: f64,
    contour_score: f64,
    river_score: f64,
    border_score: f64,
    base_score: f64,

    parent_idx: usize,
    collision_idx: usize,
    collision_data: Vec<usize>, // collision_idx of overlapping candidates
}

#[derive(Clone, Debug, Default)]
struct Label {
    text: String,
    fontface: String,
    fontsize: i32,
    position: Point,
    candidates: Vec<LabelCandidate>,
    candidate_idx: usize,
    score: f64,
}

pub struct MapGenerator {
    extents: Extents2d,
    resolution: f64,
    img_width: u32,
    img_height: u32,

    voronoi: Dcel,
    vertex_map: VertexMap,
    neighbour_map: Vec<Vec<usize>>,
    face_neighbours: Vec<Vec<usize>>,
    face_vertices: Vec<Vec<usize>>,
    face_edges: Vec<usize>, // not really used by index, store edge ids per face
    face_edge_lists: Vec<Vec<usize>>,

    height_map: NodeMap<f64>,
    flux_map: NodeMap<f64>,
    flow_map: NodeMap<i32>,
    is_initialized: bool,
    is_height_map_eroded: bool,

    is_land_face_table: Vec<bool>,
    is_land_face_table_initialized: bool,

    cities: Vec<City>,
    towns: Vec<Town>,

    font_data: FontFace,
    draw_scale: f64,

    // Cached draw data
    contour_data: Vec<Vec<f64>>,
    river_data: Vec<Vec<f64>>,
    border_data: Vec<Vec<f64>>,
    territory_data: Vec<i32>,

    // Feature flags
    slopes_enabled: bool,
    rivers_enabled: bool,
    contour_enabled: bool,
    borders_enabled: bool,
    cities_enabled: bool,
    towns_enabled: bool,
    labels_enabled: bool,
    area_labels_enabled: bool,

    pub rng: GlibcRand,

    // Constants
    sample_pad_factor: f64,
    poisson_k: usize,
    flux_cap_percentile: f64,
    max_erosion_rate: f64,
    erosion_river_factor: f64,
    erosion_creep_factor: f64,
    river_flux_threshold: f64,
    river_smoothing_factor: f64,
    isolevel: f64,
    min_island_face_threshold: usize,

    min_slope_threshold: f64,
    min_slope: f64,
    max_slope: f64,
    min_slope_angle: f64,
    max_slope_angle: f64,
    min_slope_length: f64,
    max_slope_length: f64,
    min_vertical_slope: f64,
    max_vertical_slope: f64,

    flux_score_bonus: f64,
    near_edge_score_penalty: f64,
    near_city_score_penalty: f64,
    near_town_score_penalty: f64,
    max_penalty_distance: f64,

    land_distance_cost: f64,
    sea_distance_cost: f64,
    uphill_cost: f64,
    downhill_cost: f64,
    flux_cost: f64,
    land_transition_cost: f64,

    num_territory_border_smoothing_iterations: usize,

    city_marker_radius: f64,
    town_marker_radius: f64,
    city_label_font_face: String,
    town_label_font_face: String,
    area_label_font_face: String,
    city_label_font_size: i32,
    town_label_font_size: i32,
    area_label_font_size: i32,
    num_area_label_samples: usize,
    num_area_label_candidates: usize,
    spatial_grid_resolution_factor: f64,
    label_marker_radius_factor: f64,
    area_label_marker_radius_factor: f64,
    edge_score_penalty: f64,
    marker_score_penalty: f64,
    min_contour_score_penalty: f64,
    max_contour_score_penalty: f64,
    min_river_score_penalty: f64,
    max_river_score_penalty: f64,
    min_border_score_penalty: f64,
    max_border_score_penalty: f64,
    overlap_score_penalty: f64,
    territory_score: f64,
    enemy_score: f64,
    water_score: f64,

    initial_temperature: f64,
    annealing_factor: f64,
    max_temperature_changes: i32,
    successful_repositioning_factor: i32,
    total_repositioning_factor: i32,
}

impl MapGenerator {
    pub fn new(extents: Extents2d, resolution: f64, img_width: u32, img_height: u32, rng: GlibcRand) -> Self {
        let font_data_str = include_str!("fontdata/fontdata.json");
        let font_data = FontFace::new(font_data_str);
        MapGenerator {
            extents,
            resolution,
            img_width,
            img_height,
            voronoi: Dcel::new(),
            vertex_map: VertexMap::new_empty(),
            neighbour_map: Vec::new(),
            face_neighbours: Vec::new(),
            face_vertices: Vec::new(),
            face_edges: Vec::new(),
            face_edge_lists: Vec::new(),
            height_map: NodeMap::new(0),
            flux_map: NodeMap::new(0),
            flow_map: NodeMap::new(0),
            is_initialized: false,
            is_height_map_eroded: false,
            is_land_face_table: Vec::new(),
            is_land_face_table_initialized: false,
            cities: Vec::new(),
            towns: Vec::new(),
            font_data,
            draw_scale: 1.0,
            contour_data: Vec::new(),
            river_data: Vec::new(),
            border_data: Vec::new(),
            territory_data: Vec::new(),
            slopes_enabled: true,
            rivers_enabled: true,
            contour_enabled: true,
            borders_enabled: true,
            cities_enabled: true,
            towns_enabled: true,
            labels_enabled: true,
            area_labels_enabled: true,
            rng,
            sample_pad_factor: 3.5,
            poisson_k: 25,
            flux_cap_percentile: 0.995,
            max_erosion_rate: 50.0,
            erosion_river_factor: 500.0,
            erosion_creep_factor: 500.0,
            river_flux_threshold: 0.06,
            river_smoothing_factor: 0.5,
            isolevel: 0.0,
            min_island_face_threshold: 35,
            min_slope_threshold: 0.07,
            min_slope: 0.0,
            max_slope: 0.7,
            min_slope_angle: 0.2,
            max_slope_angle: 1.5,
            min_slope_length: 0.75,
            max_slope_length: 1.3,
            min_vertical_slope: -0.25,
            max_vertical_slope: 0.05,
            flux_score_bonus: 2.0,
            near_edge_score_penalty: 0.5,
            near_city_score_penalty: 2.0,
            near_town_score_penalty: 1.5,
            max_penalty_distance: 4.0,
            land_distance_cost: 0.2,
            sea_distance_cost: 0.4,
            uphill_cost: 0.1,
            downhill_cost: 1.0,
            flux_cost: 0.8,
            land_transition_cost: 0.0,
            num_territory_border_smoothing_iterations: 3,
            city_marker_radius: 10.0,
            town_marker_radius: 5.0,
            city_label_font_face: "Times New Roman".to_string(),
            town_label_font_face: "Times New Roman".to_string(),
            area_label_font_face: "Times New Roman".to_string(),
            city_label_font_size: 35,
            town_label_font_size: 25,
            area_label_font_size: 35,
            num_area_label_samples: 500,
            num_area_label_candidates: 120,
            spatial_grid_resolution_factor: 5.0,
            label_marker_radius_factor: 1.0,
            area_label_marker_radius_factor: 7.5,
            edge_score_penalty: 4.0,
            marker_score_penalty: 6.0,
            min_contour_score_penalty: 0.5,
            max_contour_score_penalty: 1.5,
            min_river_score_penalty: 0.7,
            max_river_score_penalty: 2.0,
            min_border_score_penalty: 0.8,
            max_border_score_penalty: 2.0,
            overlap_score_penalty: 4.0,
            territory_score: 0.0,
            enemy_score: 6.0,
            water_score: 0.2,
            initial_temperature: 0.91023922,
            annealing_factor: 0.9,
            max_temperature_changes: 100,
            successful_repositioning_factor: 5,
            total_repositioning_factor: 20,
        }
    }

    pub fn get_extents(&self) -> Extents2d {
        self.extents
    }

    pub fn set_draw_scale(&mut self, scale: f64) {
        if scale > 0.0 {
            let orig = self.draw_scale;
            self.draw_scale = scale;
            self.city_marker_radius *= scale / orig;
            self.town_marker_radius *= scale / orig;
        }
    }

    pub fn rng_mut(&mut self) -> &mut GlibcRand {
        &mut self.rng
    }

    pub fn disable_slopes(&mut self) { self.slopes_enabled = false; }
    pub fn disable_rivers(&mut self) { self.rivers_enabled = false; }
    pub fn disable_contour(&mut self) { self.contour_enabled = false; }
    pub fn disable_borders(&mut self) { self.borders_enabled = false; }
    pub fn disable_cities(&mut self) { self.cities_enabled = false; }
    pub fn disable_towns(&mut self) { self.towns_enabled = false; }
    pub fn disable_labels(&mut self) { self.labels_enabled = false; }
    pub fn disable_area_labels(&mut self) { self.area_labels_enabled = false; }

    pub fn initialize(&mut self) {
        self.init_voronoi_data();
        self.init_map_data();
        self.is_initialized = true;
    }

    fn init_voronoi_data(&mut self) {
        let pad = self.sample_pad_factor * self.resolution;
        let sample_extents = Extents2d::new(
            self.extents.minx - pad, self.extents.miny - pad,
            self.extents.maxx + pad, self.extents.maxy + pad,
        );
        let mut samples = poisson_disc::generate_samples(&mut self.rng, sample_extents, self.resolution, self.poisson_k);
        let triangulation = delaunay::triangulate(&mut samples);
        self.voronoi = voronoi::delaunay_to_voronoi(&triangulation);
    }

    fn init_map_data(&mut self) {
        self.vertex_map = VertexMap::new(&self.voronoi, self.extents);
        let n = self.vertex_map.size();
        self.height_map = NodeMap::new_filled(n, 0.0);
        self.flux_map = NodeMap::new_filled(n, 0.0);
        self.flow_map = NodeMap::new_filled(n, -1i32);

        // Build neighbour map
        self.neighbour_map = Vec::with_capacity(n);
        for i in 0..n {
            let v = self.vertex_map.vertices[i];
            let nbs = self.vertex_map.get_neighbour_indices(&self.voronoi, v);
            self.neighbour_map.push(nbs);
        }

        // Build face data
        let nf = self.voronoi.faces.len();
        self.face_neighbours = Vec::with_capacity(nf);
        self.face_vertices = Vec::with_capacity(nf);
        self.face_edge_lists = Vec::with_capacity(nf);

        for i in 0..nf {
            let f = self.voronoi.faces[i];

            if !f.outer_component.is_valid() {
                self.face_neighbours.push(Vec::new());
                self.face_vertices.push(Vec::new());
                self.face_edge_lists.push(Vec::new());
                continue;
            }

            // neighbours
            let edges = self.voronoi.get_outer_components(&f);
            let mut nbs = Vec::new();
            for e in &edges {
                let tw = self.voronoi.twin(*e);
                if tw.incident_face.is_valid() {
                    nbs.push(tw.incident_face.id as usize);
                }
            }
            self.face_neighbours.push(nbs);

            // vertices
            let verts: Vec<usize> = edges.iter()
                .map(|e| self.voronoi.origin(*e).id.id as usize)
                .collect();
            self.face_vertices.push(verts);

            // edges
            let eids: Vec<usize> = edges.iter().map(|e| e.id.id as usize).collect();
            self.face_edge_lists.push(eids);
        }
    }

    pub fn add_hill(&mut self, px: f64, py: f64, r: f64, height: f64) {
        let coef1 = (4.0 / 9.0) / (r * r * r * r * r * r);
        let coef2 = (17.0 / 9.0) / (r * r * r * r);
        let coef3 = (22.0 / 9.0) / (r * r);
        let rsq = r * r;
        for i in 0..self.vertex_map.size() {
            let v = self.vertex_map.vertices[i].position;
            let dx = v.x - px;
            let dy = v.y - py;
            let dsq = dx * dx + dy * dy;
            if dsq < rsq {
                let kernel = 1.0 - coef1 * dsq * dsq * dsq + coef2 * dsq * dsq - coef3 * dsq;
                let hval = *self.height_map.get(i);
                self.height_map.set(i, hval + height * kernel);
            }
        }
    }

    pub fn add_cone(&mut self, px: f64, py: f64, radius: f64, height: f64) {
        let inv_r = 1.0 / radius;
        let rsq = radius * radius;
        for i in 0..self.vertex_map.size() {
            let v = self.vertex_map.vertices[i].position;
            let dx = v.x - px;
            let dy = v.y - py;
            let dsq = dx * dx + dy * dy;
            if dsq < rsq {
                let dist = dsq.sqrt();
                let kernel = 1.0 - dist * inv_r;
                let hval = *self.height_map.get(i);
                self.height_map.set(i, hval + height * kernel);
            }
        }
    }

    pub fn add_slope(&mut self, px: f64, py: f64, dirx: f64, diry: f64, radius: f64, height: f64) {
        for i in 0..self.vertex_map.size() {
            let v = self.vertex_map.vertices[i].position;
            let dx = px - v.x;
            let dy = py - v.y;
            let dot = dx * dirx + dy * diry;
            let distx = dx - dot * dirx;
            let disty = dy - dot * diry;
            let dist = (distx * distx + disty * disty).sqrt().min(radius);
            let cross = dx * diry - dy * dirx;
            let (min, max) = if cross < 0.0 {
                (0.5 * height, 0.0)
            } else {
                (0.5 * height, height)
            };
            let fieldval = min + (dist / radius) * (max - min);
            let hval = *self.height_map.get(i);
            self.height_map.set(i, hval + fieldval);
        }
    }

    pub fn normalize(&mut self) {
        self.height_map.normalize();
    }

    pub fn round(&mut self) {
        self.height_map.round();
    }

    pub fn relax(&mut self) {
        self.height_map.relax(&self.vertex_map, &self.voronoi);
    }

    pub fn set_sea_level_to_median(&mut self) {
        self.height_map.set_level_to_median();
    }

    pub fn erode(&mut self, amount: f64) {
        let mut erosion_map = NodeMap::new_filled(self.vertex_map.size(), 0.0);
        self.calculate_erosion_map(&mut erosion_map);
        for i in 0..self.height_map.size() {
            let cur = *self.height_map.get(i);
            let new = cur - amount * erosion_map.get(i);
            self.height_map.set(i, new);
        }
        self.is_height_map_eroded = true;
    }

    fn calculate_erosion_map(&mut self, erosion_map: &mut NodeMap<f64>) {
        self.fill_depressions();
        self.calculate_flux_map();
        let mut slope_map = NodeMap::new_filled(self.vertex_map.size(), 0.0);
        self.calculate_slope_map(&mut slope_map);

        for i in 0..erosion_map.size() {
            let flux = *self.flux_map.get(i);
            let slope = *slope_map.get(i);
            let river = self.erosion_river_factor * flux.sqrt() * slope;
            let creep = self.erosion_creep_factor * slope * slope;
            let erosion = (river + creep).min(self.max_erosion_rate);
            erosion_map.set(i, erosion);
        }
        erosion_map.normalize();
    }

    fn fill_depressions(&mut self) {
        let max_h = self.height_map.max_val();
        let n = self.vertex_map.size();
        let mut final_hm = NodeMap::new_filled(n, max_h);

        for i in 0..self.vertex_map.edge.len() {
            let v = self.vertex_map.edge[i];
            let idx = self.vertex_map.get_vertex_index(v) as usize;
            let hval = *self.height_map.get(idx);
            final_hm.set(idx, hval);
        }

        let eps = 1e-5;
        loop {
            let mut updated = false;
            for i in 0..n {
                let h = *self.height_map.get(i);
                let fh = *final_hm.get(i);
                if h == fh { continue; }
                for &nb in &self.neighbour_map[i] {
                    let nval = *final_hm.get(nb);
                    if h >= nval + eps {
                        final_hm.set(i, h);
                        updated = true;
                        break;
                    }
                    let hval = nval + eps;
                    if fh > hval && hval > h {
                        final_hm.set(i, hval);
                        updated = true;
                    }
                }
            }
            if !updated { break; }
        }
        self.height_map = final_hm;
    }

    fn calculate_flow_map(&mut self) {
        let n = self.vertex_map.size();
        let mut flow_map = NodeMap::new_filled(n, -1i32);
        for i in 0..self.vertex_map.interior.len() {
            let v = self.vertex_map.interior[i];
            let vidx = self.vertex_map.get_vertex_index(v) as usize;
            let h = *self.height_map.get(vidx);
            let mut min_h = h;
            let mut min_idx = -1i32;
            for &nb in &self.neighbour_map[vidx] {
                let nh = *self.height_map.get(nb);
                if nh < min_h {
                    min_h = nh;
                    min_idx = nb as i32;
                }
            }
            flow_map.set(vidx, min_idx);
        }
        self.flow_map = flow_map;
    }

    fn calculate_flux_map(&mut self) {
        self.calculate_flow_map();
        let n = self.vertex_map.size();
        let mut flux_map = NodeMap::new_filled(n, 0.0f64);

        for i in 0..n {
            let mut next = i as i32;
            while next != -1 {
                let cur = *flux_map.get(next as usize);
                flux_map.set(next as usize, cur + 1.0);
                next = *self.flow_map.get(next as usize);
            }
        }

        // Cap at percentile
        let max_flux = self.calculate_flux_cap(&flux_map);
        for i in 0..n {
            let f = (*flux_map.get(i)).min(max_flux) / max_flux;
            flux_map.set(i, f);
        }
        self.flux_map = flux_map;
    }

    fn calculate_flux_cap(&self, flux_map: &NodeMap<f64>) -> f64 {
        let max = flux_map.max_val();
        let nbins = 1000;
        let mut bins = vec![0usize; nbins];
        let step = max / nbins as f64;
        let inv_step = 1.0 / step;
        for i in 0..flux_map.size() {
            let f = *flux_map.get(i);
            let bin = ((f * inv_step).floor() as usize).min(nbins - 1);
            bins[bin] += 1;
        }
        let mut acc = 0.0;
        for i in 0..nbins {
            let pct = bins[i] as f64 / flux_map.size() as f64;
            acc += pct;
            if acc > self.flux_cap_percentile {
                return (i + 1) as f64 * step;
            }
        }
        max
    }

    fn calculate_slope_map(&self, slope_map: &mut NodeMap<f64>) {
        for i in 0..slope_map.size() {
            let s = self.calculate_slope(i);
            slope_map.set(i, s);
        }
    }

    fn calculate_slope(&self, i: usize) -> f64 {
        let v = self.vertex_map.vertices[i];
        if !self.vertex_map.is_interior_vertex(v) { return 0.0; }
        let (nx, ny, _) = self.calculate_vertex_normal(i);
        (nx * nx + ny * ny).sqrt()
    }

    fn calculate_vertex_normal(&self, vidx: usize) -> (f64, f64, f64) {
        let nbs = &self.neighbour_map[vidx];
        if nbs.len() < 3 { return (0.0, 0.0, 1.0); }
        let p0 = self.vertex_map.vertices[nbs[0]].position;
        let p1 = self.vertex_map.vertices[nbs[1]].position;
        let p2 = self.vertex_map.vertices[nbs[2]].position;
        let v0x = p1.x - p0.x;
        let v0y = p1.y - p0.y;
        let v0z = *self.height_map.get(nbs[1]) - *self.height_map.get(nbs[0]);
        let v1x = p2.x - p0.x;
        let v1y = p2.y - p0.y;
        let v1z = *self.height_map.get(nbs[2]) - *self.height_map.get(nbs[0]);
        let vnx = v0y * v1z - v0z * v1y;
        let vny = v0z * v1x - v0x * v1z;
        let vnz = v0x * v1y - v0y * v1x;
        let len = (vnx * vnx + vny * vny + vnz * vnz).sqrt();
        if len < 1e-12 { return (0.0, 0.0, 1.0); }
        (vnx / len, vny / len, vnz / len)
    }

    fn compute_face_values(&self, node_map: &NodeMap<f64>) -> Vec<f64> {
        let nf = self.voronoi.faces.len();
        let mut result = Vec::with_capacity(nf);
        for j in 0..nf {
            let verts = &self.face_vertices[j];
            if verts.is_empty() {
                result.push(0.0);
                continue;
            }
            let mut sum = 0.0;
            let mut count = 0;
            for &vid in verts {
                let v = if vid < self.voronoi.vertices.len() {
                    self.voronoi.vertices[vid]
                } else { continue; };
                let idx = self.vertex_map.get_vertex_index(v);
                if idx >= 0 {
                    sum += node_map.get(idx as usize);
                    count += 1;
                }
            }
            result.push(if count > 0 { sum / count as f64 } else { 0.0 });
        }
        result
    }

    fn compute_face_positions(&self) -> Vec<Point> {
        let nf = self.voronoi.faces.len();
        let mut result = Vec::with_capacity(nf);
        for j in 0..nf {
            result.push(self.compute_face_position(j));
        }
        result
    }

    fn compute_face_position(&self, fidx: usize) -> Point {
        let verts = &self.face_vertices[fidx];
        if verts.is_empty() { return Point::new(0.0, 0.0); }
        let mut sx = 0.0;
        let mut sy = 0.0;
        let mut cnt = 0;
        for &vid in verts {
            if vid < self.voronoi.vertices.len() {
                let p = self.voronoi.vertices[vid].position;
                sx += p.x;
                sy += p.y;
                cnt += 1;
            }
        }
        if cnt > 0 { Point::new(sx / cnt as f64, sy / cnt as f64) } else { Point::new(0.0, 0.0) }
    }

    fn is_edge_in_map(&self, h: HalfEdge) -> bool {
        let v1 = self.voronoi.origin(h);
        let tw = self.voronoi.twin(h);
        let v2 = self.voronoi.origin(tw);
        self.vertex_map.is_vertex(v1) && self.vertex_map.is_vertex(v2)
    }

    fn is_land_face(&mut self, fidx: usize) -> bool {
        if !self.is_land_face_table_initialized {
            self.init_land_face_table();
        }
        if fidx < self.is_land_face_table.len() {
            self.is_land_face_table[fidx]
        } else {
            false
        }
    }

    fn init_land_face_table(&mut self) {
        let face_heights = self.compute_face_values(&self.height_map.clone());
        let nf = self.voronoi.faces.len();
        let mut is_land = vec![false; nf];
        for i in 0..nf {
            is_land[i] = face_heights[i] >= self.isolevel;
        }
        self.cleanup_land_faces(&mut is_land);
        self.is_land_face_table = is_land;
        self.is_land_face_table_initialized = true;
    }

    fn cleanup_land_faces(&self, is_land: &mut Vec<bool>) {
        let nf = is_land.len();
        let mut processed = vec![false; nf];
        let mut islands: Vec<Vec<usize>> = Vec::new();
        for i in 0..nf {
            if processed[i] { continue; }
            let mut connected = Vec::new();
            self.get_connected_faces(i, is_land, &mut processed, &mut connected);
            islands.push(connected);
        }
        for island in &islands {
            if island.len() >= self.min_island_face_threshold { continue; }
            for &fidx in island {
                is_land[fidx] = !is_land[fidx];
            }
        }
    }

    fn get_connected_faces(&self, seed: usize, is_land: &[bool], processed: &mut Vec<bool>, faces: &mut Vec<usize>) {
        let mut queue = vec![seed];
        processed[seed] = true;
        while let Some(fidx) = queue.pop() {
            let ftype = is_land[fidx];
            for &nfidx in &self.face_neighbours[fidx] {
                if nfidx < is_land.len() && is_land[nfidx] == ftype && !processed[nfidx] {
                    queue.push(nfidx);
                    processed[nfidx] = true;
                }
            }
            faces.push(fidx);
        }
    }

    fn is_contour_edge(&mut self, h: HalfEdge) -> bool {
        let f1 = self.voronoi.incident_face(h);
        let f2 = self.voronoi.incident_face(self.voronoi.twin(h));
        if !f1.id.is_valid() || !f2.id.is_valid() { return false; }
        let land1 = self.is_land_face(f1.id.id as usize);
        let land2 = self.is_land_face(f2.id.id as usize);
        (land1 && !land2) || (!land1 && land2)
    }

    fn is_contour_edge_between(&mut self, v1_id: i32, v2_id: i32) -> bool {
        if v1_id < 0 || v1_id as usize >= self.voronoi.vertices.len() { return false; }
        let v1 = self.voronoi.vertices[v1_id as usize];
        let edges = self.voronoi.get_incident_edges(v1);
        for h in edges {
            let tw = self.voronoi.twin(h);
            let v = self.voronoi.origin(tw);
            if v.id.id == v2_id {
                return self.is_contour_edge(h);
            }
        }
        false
    }

    fn is_land_vertex(&mut self, vidx: usize) -> bool {
        if vidx >= self.vertex_map.vertices.len() { return false; }
        let v = self.vertex_map.vertices[vidx];
        let faces = self.voronoi.get_incident_faces(v);
        for f in &faces {
            if self.is_land_face(f.id.id as usize) { return true; }
        }
        false
    }

    fn is_coast_vertex(&mut self, vidx: usize) -> bool {
        if vidx >= self.vertex_map.vertices.len() { return false; }
        let v = self.vertex_map.vertices[vidx];
        let faces = self.voronoi.get_incident_faces(v);
        let mut has_land = false;
        let mut has_sea = false;
        for f in &faces {
            if self.is_land_face(f.id.id as usize) { has_land = true; }
            else { has_sea = true; }
        }
        has_land && has_sea
    }

    // ----- contour paths -----
    fn get_contour_draw_data(&mut self) -> Vec<Vec<f64>> {
        let paths = self.get_contour_paths();
        let inv_w = 1.0 / (self.extents.maxx - self.extents.minx);
        let inv_h = 1.0 / (self.extents.maxy - self.extents.miny);
        paths.iter().map(|path| {
            let mut out = Vec::with_capacity(path.len() * 2);
            for &vidx in path {
                let v = self.vertex_map.vertices[vidx];
                out.push((v.position.x - self.extents.minx) * inv_w);
                out.push((v.position.y - self.extents.miny) * inv_h);
            }
            out
        }).collect()
    }

    fn get_contour_paths(&mut self) -> Vec<Vec<usize>> {
        let n = self.vertex_map.vertices.len();
        let mut adj_counts = vec![0usize; n];
        let mut edge_visited = vec![false; self.voronoi.edges.len()];

        for i in 0..self.voronoi.edges.len() {
            let h = self.voronoi.edges[i];
            if h.id.id as usize != i { continue; }
            if edge_visited[i] { continue; }
            if !self.is_edge_in_map(h) { continue; }
            if !self.is_contour_edge(h) { continue; }
            let v1 = self.voronoi.origin(h);
            let v2 = self.voronoi.origin(self.voronoi.twin(h));
            let idx1 = self.vertex_map.get_vertex_index(v1);
            let idx2 = self.vertex_map.get_vertex_index(v2);
            if idx1 >= 0 && idx2 >= 0 {
                adj_counts[idx1 as usize] += 1;
                adj_counts[idx2 as usize] += 1;
            }
            edge_visited[i] = true;
            if h.twin.is_valid() {
                edge_visited[h.twin.id as usize] = true;
            }
        }

        let mut is_contour_vertex = vec![false; n];
        let mut is_end_vertex = vec![false; n];
        for i in 0..n {
            if adj_counts[i] == 1 {
                is_end_vertex[i] = true;
                is_contour_vertex[i] = true;
            } else if adj_counts[i] == 2 {
                is_contour_vertex[i] = true;
            }
        }

        let mut in_contour = vec![false; n];
        let mut paths = Vec::new();

        // Start from end vertices first
        for i in 0..n {
            if is_end_vertex[i] && !in_contour[i] {
                let path = self.get_contour_path(i, &is_contour_vertex, &is_end_vertex, &mut in_contour);
                if !path.is_empty() { paths.push(path); }
            }
        }
        for i in 0..n {
            if is_contour_vertex[i] && !in_contour[i] {
                let path = self.get_contour_path(i, &is_contour_vertex, &is_end_vertex, &mut in_contour);
                if !path.is_empty() { paths.push(path); }
            }
        }
        paths
    }

    fn get_contour_path(&mut self, seed: usize, is_contour: &[bool], is_end: &[bool], in_contour: &mut Vec<bool>) -> Vec<usize> {
        let mut path = Vec::new();
        let mut v_idx = seed;
        let mut last_idx = usize::MAX;

        loop {
            path.push(v_idx);
            in_contour[v_idx] = true;
            let v = self.vertex_map.vertices[v_idx];
            let nbs = self.neighbour_map[v_idx].clone();
            let mut found = false;
            for &nb in &nbs {
                if nb != last_idx && is_contour[nb] && self.is_contour_edge_between(v.id.id, self.vertex_map.vertices[nb].id.id) {
                    last_idx = v_idx;
                    v_idx = nb;
                    found = true;
                    break;
                }
            }
            if !found { break; }
            if is_end[v_idx] || in_contour[v_idx] {
                path.push(v_idx);
                in_contour[v_idx] = true;
                break;
            }
        }
        path
    }

    // ----- river paths -----
    fn get_river_draw_data(&mut self) -> Vec<Vec<f64>> {
        let paths = self.get_river_paths();
        let inv_w = 1.0 / (self.extents.maxx - self.extents.minx);
        let inv_h = 1.0 / (self.extents.maxy - self.extents.miny);
        let factor = self.river_smoothing_factor;
        paths.iter().map(|path| {
            let raw: Vec<(f64, f64)> = path.iter().map(|&vidx| {
                let v = self.vertex_map.vertices[vidx];
                (
                    (v.position.x - self.extents.minx) * inv_w,
                    (v.position.y - self.extents.miny) * inv_h,
                )
            }).collect();
            let smooth = smooth_positions(raw, factor);
            let mut out = Vec::with_capacity(smooth.len() * 2);
            for (x, y) in smooth {
                out.push(x);
                out.push(y);
            }
            out
        }).collect()
    }

    fn get_river_paths(&mut self) -> Vec<Vec<usize>> {
        let n = self.vertex_map.size();
        let mut is_river_vertex = vec![false; n];

        // Mark river vertices
        for i in 0..n {
            let v = self.vertex_map.vertices[i];
            if *self.flux_map.get(i) < self.river_flux_threshold || self.is_coast_vertex(i) {
                continue;
            }
            let mut next = *self.flow_map.get(i);
            let mut path_verts = Vec::new();
            while next != -1 {
                let ni = next as usize;
                path_verts.push(ni);
                if self.is_coast_vertex(ni) { break; }
                if !self.is_land_vertex(ni) { path_verts.clear(); break; }
                next = *self.flow_map.get(ni);
            }
            for &pv in &path_verts {
                is_river_vertex[pv] = true;
            }
        }

        // Count adjacent river edges
        let mut adj_counts = vec![0usize; n];
        let mut edge_processed = vec![false; self.voronoi.edges.len()];
        for i in 0..self.voronoi.edges.len() {
            let h = self.voronoi.edges[i];
            if h.id.id as usize != i { continue; }
            if edge_processed[i] { continue; }
            if !self.is_edge_in_map(h) { continue; }
            let v1 = self.voronoi.origin(h);
            let v2 = self.voronoi.origin(self.voronoi.twin(h));
            let idx1 = self.vertex_map.get_vertex_index(v1);
            let idx2 = self.vertex_map.get_vertex_index(v2);
            if idx1 >= 0 && idx2 >= 0 {
                let i1 = idx1 as usize;
                let i2 = idx2 as usize;
                if is_river_vertex[i1] && is_river_vertex[i2] {
                    adj_counts[i1] += 1;
                    adj_counts[i2] += 1;
                }
            }
            edge_processed[i] = true;
            if h.twin.is_valid() { edge_processed[h.twin.id as usize] = true; }
        }

        let mut is_fixed = vec![false; n];
        for i in 0..n {
            if adj_counts[i] == 1 || adj_counts[i] == 3 {
                is_fixed[i] = true;
            }
        }

        let mut paths = Vec::new();
        for i in 0..n {
            if !is_fixed[i] { continue; }
            let mut path = Vec::new();
            let mut next = i as i32;
            while next != -1 {
                let ni = next as usize;
                if !is_river_vertex.get(ni).copied().unwrap_or(false) { break; }
                path.push(ni);
                next = *self.flow_map.get(ni);
                if next == -1 { break; }
                if is_fixed.get(next as usize).copied().unwrap_or(false) {
                    path.push(next as usize);
                    break;
                }
            }
            if path.len() >= 2 {
                paths.push(path);
            }
        }
        paths
    }

    // ----- slope segments -----
    fn get_slope_draw_data(&mut self) -> Vec<f64> {
        let mut h_slope = NodeMap::new_filled(self.vertex_map.size(), 0.0);
        let mut v_slope = NodeMap::new_filled(self.vertex_map.size(), 0.0);
        for i in 0..self.vertex_map.size() {
            let v = self.vertex_map.vertices[i];
            if !self.vertex_map.is_interior_vertex(v) { continue; }
            let (nx, ny, _) = self.calculate_vertex_normal(i);
            h_slope.set(i, nx);
            v_slope.set(i, ny);
        }

        let face_slopes = self.compute_face_values(&h_slope);
        let near_slopes = self.compute_face_values(&v_slope);
        let face_positions = self.compute_face_positions();

        let inv_w = 1.0 / (self.extents.maxx - self.extents.minx);
        let inv_h = 1.0 / (self.extents.maxy - self.extents.miny);
        let mut data = Vec::new();

        for i in 0..self.voronoi.faces.len() {
            let slope = face_slopes[i];
            if !self.is_land_face(i) || slope.abs() < self.min_slope_threshold { continue; }

            let factor = ((slope.abs() - self.min_slope) / (self.max_slope - self.min_slope)).clamp(0.0, 1.0);
            let angle = self.min_slope_angle + factor * (self.max_slope_angle - self.min_slope_angle);
            let angle = if slope < 0.0 { angle } else { -angle };
            let dirx = angle.cos();
            let diry = angle.sin();

            let min_len = self.min_slope_length * self.resolution;
            let max_len = self.max_slope_length * self.resolution;
            let vslope = near_slopes[i];
            let nf = ((vslope - self.min_vertical_slope) / (self.max_vertical_slope - self.min_vertical_slope)).clamp(0.0, 1.0);
            let length = min_len + nf * (max_len - min_len);

            let p1 = face_positions[i];
            let p2 = Point::new(p1.x + dirx * length, p1.y + diry * length);

            data.push((p1.x - self.extents.minx) * inv_w);
            data.push((p1.y - self.extents.miny) * inv_h);
            data.push((p2.x - self.extents.minx) * inv_w);
            data.push((p2.y - self.extents.miny) * inv_h);
        }
        data
    }

    // ----- city/town placement -----
    fn ensure_eroded(&mut self) {
        if !self.is_height_map_eroded {
            self.erode(0.0);
        }
    }

    pub fn add_city(&mut self, city_name: String, territory_name: String) {
        self.ensure_eroded();
        let loc = self.get_city_location();
        let mut city = City {
            city_name,
            territory_name,
            position: loc.0,
            face_id: loc.1,
            movement_costs: Vec::new(),
        };
        self.update_city_movement_cost(&mut city);
        self.cities.push(city);
    }

    pub fn add_town(&mut self, town_name: String) {
        self.ensure_eroded();
        let loc = self.get_city_location();
        self.towns.push(Town {
            town_name,
            position: loc.0,
            face_id: loc.1,
        });
    }

    fn get_city_location(&mut self) -> (Point, usize) {
        let city_scores = self.get_city_scores();
        let face_scores = self.compute_face_values(&city_scores);
        let face_positions = self.compute_face_positions();
        let mut max_score = f64::NEG_INFINITY;
        let mut best_fidx = 0usize;
        for i in 0..face_scores.len() {
            let fp = face_positions[i];
            if self.extents.contains_point(fp) && face_scores[i] > max_score {
                max_score = face_scores[i];
                best_fidx = i;
            }
        }
        (self.compute_face_position(best_fidx), best_fidx)
    }

    fn get_city_scores(&mut self) -> NodeMap<f64> {
        let mut relaxed_flux = self.flux_map.clone();
        relaxed_flux.relax(&self.vertex_map, &self.voronoi);

        let n = self.vertex_map.size();
        let mut scores = NodeMap::new_filled(n, 0.0f64);
        let neg_inf = -1e2;
        let eps = 1e-6;

        let city_positions: Vec<Point> = self.cities.iter().map(|c| c.position).collect();
        let town_positions: Vec<Point> = self.towns.iter().map(|t| t.position).collect();

        for i in 0..n {
            let mut score = 0.0;
            if !self.is_land_vertex(i) || self.is_coast_vertex(i) {
                score += neg_inf;
            }
            score += self.flux_score_bonus * relaxed_flux.get(i).sqrt();
            let p = self.vertex_map.vertices[i].position;
            let edge_dist = f64::max(0.0, self.point_to_edge_distance(p));
            score -= self.near_edge_score_penalty * (1.0 / (edge_dist + eps));
            for &cp in &city_positions {
                let dist = f64::min(point_distance(p, cp), self.max_penalty_distance);
                let df = 1.0 - dist / self.max_penalty_distance;
                score -= self.near_city_score_penalty * df;
            }
            for &tp in &town_positions {
                let dist = f64::min(point_distance(p, tp), self.max_penalty_distance);
                let df = 1.0 - dist / self.max_penalty_distance;
                score -= self.near_town_score_penalty * df;
            }
            score = score.max(neg_inf);
            scores.set(i, score);
        }
        scores
    }

    fn point_to_edge_distance(&self, p: Point) -> f64 {
        let d1 = p.x - self.extents.minx;
        let d2 = self.extents.maxx - p.x;
        let d3 = p.y - self.extents.miny;
        let d4 = self.extents.maxy - p.y;
        d1.min(d2).min(d3).min(d4)
    }

    fn update_city_movement_cost(&mut self, city: &mut City) {
        let face_heights = self.compute_face_values(&self.height_map.clone());
        let face_flux = self.compute_face_values(&self.flux_map.clone());
        let face_positions = self.compute_face_positions();
        let nf = self.voronoi.faces.len();
        let is_in_map: Vec<bool> = (0..nf).map(|i| self.extents.contains_point(face_positions[i])).collect();

        let inf = f64::INFINITY;
        let mut costs = vec![inf; nf];
        costs[city.face_id] = 0.0;
        let mut queue = VecDeque::new();
        queue.push_back(city.face_id);

        let land_table = if self.is_land_face_table_initialized {
            self.is_land_face_table.clone()
        } else {
            vec![false; nf]
        };

        while let Some(fidx) = queue.pop_front() {
            for &nidx in &self.face_neighbours[fidx].clone() {
                if !is_in_map[nidx] || costs[nidx] != inf { continue; }
                let hdist = point_distance(face_positions[nidx], face_positions[fidx]);
                let hcost = if nidx < land_table.len() && land_table[nidx] {
                    self.land_distance_cost
                } else {
                    self.sea_distance_cost
                };
                let mut cost = hcost * hdist;
                if nidx < land_table.len() && land_table[nidx] {
                    let udist = face_heights[nidx] - face_heights[fidx];
                    let ucost = if udist > 0.0 { self.uphill_cost } else { self.downhill_cost };
                    let slope = udist / hdist.max(1e-12);
                    cost += slope * slope * ucost;
                    cost += face_flux[nidx].sqrt() * self.flux_cost;
                }
                costs[nidx] = costs[fidx] + cost;
                queue.push_back(nidx);
            }
        }
        city.movement_costs = costs;
    }

    // ----- territory borders -----
    fn get_territory_draw_data(&mut self) -> Vec<Vec<f64>> {
        if self.cities.is_empty() { return Vec::new(); }
        let borders = self.get_territory_borders();
        let inv_w = 1.0 / (self.extents.maxx - self.extents.minx);
        let inv_h = 1.0 / (self.extents.maxy - self.extents.miny);
        let factor = 0.5f64;
        borders.iter().map(|path| {
            let raw: Vec<(f64, f64)> = path.iter().map(|&vidx| {
                let v = self.vertex_map.vertices[vidx];
                (
                    (v.position.x - self.extents.minx) * inv_w,
                    (v.position.y - self.extents.miny) * inv_h,
                )
            }).collect();
            let smooth = smooth_positions(raw, factor);
            let mut out = Vec::with_capacity(smooth.len() * 2);
            for (x, y) in smooth {
                out.push(x);
                out.push(y);
            }
            out
        }).collect()
    }

    fn get_territory_borders(&mut self) -> Vec<Vec<usize>> {
        let nf = self.voronoi.faces.len();
        let mut face_territories = vec![-1i32; nf];
        self.get_face_territories(&mut face_territories);
        self.territory_data = face_territories.clone();
        self.get_border_paths(&face_territories)
    }

    fn get_face_territories(&mut self, face_territories: &mut Vec<i32>) {
        let face_positions = self.compute_face_positions();
        let nf = self.voronoi.faces.len();
        let is_in_map: Vec<bool> = (0..nf).map(|i| self.extents.contains_point(face_positions[i])).collect();

        for i in 0..nf {
            if !is_in_map[i] || !self.is_land_face(i) { continue; }
            let mut min_cost = f64::INFINITY;
            let mut min_cidx = -1i32;
            for (j, city) in self.cities.iter().enumerate() {
                if city.movement_costs.len() > i && city.movement_costs[i] < min_cost {
                    min_cost = city.movement_costs[i];
                    min_cidx = j as i32;
                }
            }
            face_territories[i] = min_cidx;
        }

        let ncities = self.cities.len();
        // Smooth borders
        for _ in 0..self.num_territory_border_smoothing_iterations {
            let old = face_territories.clone();
            let mut counts = vec![0i32; ncities];
            for fidx in 0..nf {
                if old[fidx] == -1 { continue; }
                for i in 0..ncities { counts[i] = 0; }
                for &nidx in &self.face_neighbours[fidx].clone() {
                    if old[nidx] >= 0 && (old[nidx] as usize) < ncities {
                        counts[old[nidx] as usize] += 1;
                    }
                }
                let cur = old[fidx] as usize;
                let mut majority = old[fidx];
                let mut majority_count = counts[cur];
                for (c, &cnt) in counts.iter().enumerate() {
                    if cnt > majority_count {
                        majority_count = cnt;
                        majority = c as i32;
                    }
                }
                face_territories[fidx] = majority;
            }
        }

        // Fix disjoint territories
        self.fix_disjoint_territories(face_territories);
    }

    fn fix_disjoint_territories(&self, face_territories: &mut Vec<i32>) {
        let nf = face_territories.len();
        let mut processed = vec![false; nf];
        let mut connected: Vec<Vec<usize>> = Vec::new();
        for i in 0..nf {
            if face_territories[i] == -1 || processed[i] { continue; }
            let tid = face_territories[i];
            let mut group = Vec::new();
            let mut stack = vec![i];
            processed[i] = true;
            while let Some(fidx) = stack.pop() {
                group.push(fidx);
                for &nidx in &self.face_neighbours[fidx] {
                    if !processed[nidx] && face_territories[nidx] == tid {
                        stack.push(nidx);
                        processed[nidx] = true;
                    }
                }
            }
            connected.push(group);
        }

        // Find disjoint groups (groups that don't contain the city face)
        let city_faces: Vec<usize> = self.cities.iter().map(|c| c.face_id).collect();
        for group in &connected {
            let cityid = face_territories[group[0]];
            if cityid < 0 { continue; }
            let cfidx = city_faces[cityid as usize];
            if group.contains(&cfidx) { continue; }
            // Find the majority neighbor
            let mut nb_counts = vec![0i32; self.cities.len()];
            let mut group_set: std::collections::HashSet<usize> = group.iter().copied().collect();
            for &fidx in group {
                for &nidx in &self.face_neighbours[fidx] {
                    if group_set.contains(&nidx) { continue; }
                    if face_territories[nidx] >= 0 {
                        nb_counts[face_territories[nidx] as usize] += 1;
                    }
                }
            }
            let majority = nb_counts.iter().enumerate()
                .max_by_key(|(_, &v)| v)
                .map(|(i, _)| i as i32)
                .unwrap_or(-1);
            for &fidx in group {
                face_territories[fidx] = majority;
            }
        }
    }

    fn is_border_edge_between_faces(&self, h: HalfEdge, face_territories: &[i32]) -> bool {
        let f1 = self.voronoi.incident_face(h);
        let f2 = self.voronoi.incident_face(self.voronoi.twin(h));
        if !f1.id.is_valid() || !f2.id.is_valid() { return false; }
        let c1 = face_territories.get(f1.id.id as usize).copied().unwrap_or(-1);
        let c2 = face_territories.get(f2.id.id as usize).copied().unwrap_or(-1);
        c1 != -1 && c2 != -1 && c1 != c2
    }

    fn is_border_edge_between_vertices(&self, v1_id: i32, v2_id: i32, face_territories: &[i32]) -> bool {
        if v1_id < 0 || v1_id as usize >= self.voronoi.vertices.len() { return false; }
        let v1 = self.voronoi.vertices[v1_id as usize];
        let edges = self.voronoi.get_incident_edges(v1);
        for h in edges {
            let tw = self.voronoi.twin(h);
            let v = self.voronoi.origin(tw);
            if v.id.id == v2_id {
                return self.is_border_edge_between_faces(h, face_territories);
            }
        }
        false
    }

    fn get_border_paths(&self, face_territories: &[i32]) -> Vec<Vec<usize>> {
        let n = self.vertex_map.vertices.len();
        let mut border_counts = vec![0usize; n];
        let mut edge_visited = vec![false; self.voronoi.edges.len()];

        for i in 0..self.voronoi.edges.len() {
            let h = self.voronoi.edges[i];
            if h.id.id as usize != i { continue; }
            if edge_visited[i] { continue; }
            if !self.is_edge_in_map(h) { continue; }
            if self.is_border_edge_between_faces(h, face_territories) {
                let v1 = self.voronoi.origin(h);
                let v2 = self.voronoi.origin(self.voronoi.twin(h));
                let idx1 = self.vertex_map.get_vertex_index(v1);
                let idx2 = self.vertex_map.get_vertex_index(v2);
                if idx1 >= 0 && idx2 >= 0 {
                    border_counts[idx1 as usize] += 1;
                    border_counts[idx2 as usize] += 1;
                }
                edge_visited[i] = true;
                if h.twin.is_valid() { edge_visited[h.twin.id as usize] = true; }
            }
        }

        let mut is_end_vertex = vec![false; n];
        for i in 0..n {
            if border_counts[i] == 1 || border_counts[i] == 3 {
                is_end_vertex[i] = true;
            }
        }

        let mut vertex_processed = vec![false; n];
        let mut paths = Vec::new();

        for i in 0..n {
            if !is_end_vertex[i] { continue; }
            for _ in 0..3 {
                let path = self.get_border_path(i, face_territories, &is_end_vertex, &mut vertex_processed);
                if !path.is_empty() { paths.push(path); }
            }
        }
        paths
    }

    fn get_border_path(&self, start: usize, face_territories: &[i32], is_end: &[bool], processed: &mut Vec<bool>) -> Vec<usize> {
        let mut path = Vec::new();
        let mut v_idx = start;
        let mut last_idx = usize::MAX;

        loop {
            path.push(v_idx);
            processed[v_idx] = true;
            let v = self.vertex_map.vertices[v_idx];
            let nbs = &self.neighbour_map[v_idx];
            let mut found = false;
            for &nb in nbs {
                if nb != last_idx && self.is_border_edge_between_vertices(v.id.id, self.vertex_map.vertices[nb].id.id, face_territories) && !processed[nb] {
                    last_idx = v_idx;
                    v_idx = nb;
                    found = true;
                    break;
                }
            }
            if !found {
                // Try to find an end vertex neighbor
                for &nb in nbs {
                    if is_end[nb] {
                        path.push(nb);
                        processed[nb] = true;
                    }
                }
                if path.len() < 2 { path.clear(); }
                break;
            }
            if is_end[v_idx] {
                path.push(v_idx);
                processed[v_idx] = true;
                break;
            }
        }
        path
    }

    // ----- labels -----
    fn get_label_draw_data(&mut self) -> Vec<Value> {
        let mut labels = self.initialize_labels();
        if labels.is_empty() { return Vec::new(); }
        self.generate_label_placements(&mut labels);

        labels.iter().map(|label| {
            let c = &label.candidates[label.candidate_idx];
            self.label_to_json(c, label.score)
        }).collect()
    }

    fn initialize_labels(&mut self) -> Vec<Label> {
        let mut marker_labels = self.initialize_marker_labels();
        let area_labels = if self.area_labels_enabled {
            self.initialize_area_labels()
        } else {
            Vec::new()
        };

        // Initialize marker scores
        if !marker_labels.is_empty() {
            self.initialize_marker_label_scores(&mut marker_labels);
        }

        let mut all = marker_labels;
        let mut area_with_scores = area_labels;
        if !area_with_scores.is_empty() {
            self.initialize_area_label_scores(&mut area_with_scores);
        }
        all.extend(area_with_scores);
        all
    }

    fn initialize_marker_labels(&mut self) -> Vec<Label> {
        let mut labels = Vec::new();
        let map_h = self.extents.maxy - self.extents.miny;
        let city_r = (self.city_marker_radius / self.img_height as f64) * map_h;
        let town_r = (self.town_marker_radius / self.img_height as f64) * map_h;

        for i in 0..self.cities.len() {
            let city = self.cities[i].clone();
            let text = city.city_name.clone();
            let pos = city.position;
            let candidates = self.get_marker_label_candidates(&text, pos, city_r, "Times New Roman", self.city_label_font_size);
            labels.push(Label {
                text,
                fontface: "Times New Roman".to_string(),
                fontsize: self.city_label_font_size,
                position: pos,
                candidates,
                candidate_idx: 0,
                score: 0.0,
            });
        }
        for i in 0..self.towns.len() {
            let town = self.towns[i].clone();
            let text = town.town_name.clone();
            let pos = town.position;
            let candidates = self.get_marker_label_candidates(&text, pos, town_r, "Times New Roman", self.town_label_font_size);
            labels.push(Label {
                text,
                fontface: "Times New Roman".to_string(),
                fontsize: self.town_label_font_size,
                position: pos,
                candidates,
                candidate_idx: 0,
                score: 0.0,
            });
        }
        labels
    }

    fn initialize_area_labels(&mut self) -> Vec<Label> {
        let mut labels = Vec::new();
        for i in 0..self.cities.len() {
            let city = self.cities[i].clone();
            let text = city.territory_name.clone();
            let pos = city.position;
            let city_id = i as i32;
            let candidates = self.get_area_label_candidates(&text, pos, &city, city_id);
            labels.push(Label {
                text,
                fontface: "Times New Roman".to_string(),
                fontsize: self.area_label_font_size,
                position: pos,
                candidates,
                candidate_idx: 0,
                score: 0.0,
            });
        }
        labels
    }

    fn get_marker_label_candidates(&self, text: &str, pos: Point, radius: f64, fontface: &str, fontsize: i32) -> Vec<LabelCandidate> {
        let offsets = self.get_label_offsets(text, pos, radius);
        offsets.into_iter().map(|(offset, score)| {
            let cpos = Point::new(pos.x + offset.x, pos.y + offset.y);
            let ext = self.get_text_extents(text, cpos);
            let char_ext = self.get_character_extents(text, cpos);
            LabelCandidate {
                text: text.to_string(),
                fontface: fontface.to_string(),
                fontsize,
                position: cpos,
                extents: ext,
                char_extents: char_ext,
                orientation_score: score,
                ..Default::default()
            }
        }).collect()
    }

    fn get_area_label_candidates(&self, text: &str, pos: Point, city: &City, city_id: i32) -> Vec<LabelCandidate> {
        let origin = Point::new(0.0, 0.0);
        let base_ext = self.get_text_extents(text, origin);
        let base_char = self.get_character_extents(text, origin);

        let center_x = 0.5 * (base_ext.minx + base_ext.maxx);
        let center_y = 0.5 * (base_ext.miny + base_ext.maxy);

        // Get territory samples
        let city_id_usize = city_id as usize;
        let samples = self.get_area_label_samples(city_id_usize);

        samples.into_iter().map(|p| {
            let tx = p.x - center_x;
            let ty = p.y - center_y;
            let mut ext = base_ext;
            ext.minx += tx; ext.miny += ty; ext.maxx += tx; ext.maxy += ty;
            let char_ext: Vec<Extents2d> = base_char.iter().map(|&ce| {
                let mut e = ce;
                e.minx += tx; e.miny += ty; e.maxx += tx; e.maxy += ty;
                e
            }).collect();
            LabelCandidate {
                text: text.to_string(),
                fontface: "Times New Roman".to_string(),
                fontsize: self.area_label_font_size,
                position: Point::new(tx, ty),
                extents: ext,
                char_extents: char_ext,
                city_id,
                ..Default::default()
            }
        }).collect()
    }

    fn get_area_label_samples(&self, city_id: usize) -> Vec<Point> {
        // Only works after territory_data is set
        if self.territory_data.is_empty() { return Vec::new(); }
        let nf = self.voronoi.faces.len();
        let mut territory_faces: Vec<usize> = (0..nf)
            .filter(|&i| i < self.territory_data.len() && self.territory_data[i] == city_id as i32)
            .collect();
        // Use territory_data counts for all cities
        let max_count = self.territory_data.iter()
            .filter(|&&id| id >= 0)
            .fold(std::collections::HashMap::new(), |mut acc, &id| {
                *acc.entry(id).or_insert(0usize) += 1;
                acc
            })
            .values()
            .copied()
            .max()
            .unwrap_or(1);

        let num_faces = territory_faces.len();
        let num_samples = ((num_faces as f64 / max_count as f64) * self.num_area_label_samples as f64)
            .min(territory_faces.len() as f64) as usize;

        territory_faces.truncate(num_samples);
        territory_faces.iter().map(|&fidx| self.compute_face_position(fidx)).collect()
    }

    fn get_label_offsets(&self, text: &str, pos: Point, r: f64) -> Vec<(Point, f64)> {
        let ext = self.get_text_extents(text, pos);
        let char_ext = self.get_character_extents(text, pos);
        let text_width = ext.maxx - ext.minx;
        let text_height = ext.maxy - ext.miny;
        let first_h = if !char_ext.is_empty() { char_ext[0].maxy - char_ext[0].miny } else { text_height };
        let last_h = if !char_ext.is_empty() { let l = char_ext.len()-1; char_ext[l].maxy - char_ext[l].miny } else { text_height };

        let first_ext = if !char_ext.is_empty() { &char_ext[0] } else { &ext };
        let last_ext = if char_ext.len() > 1 { &char_ext[char_ext.len()-1] } else { &ext };
        let star_ty = first_ext.miny - ext.miny;
        let end_y = last_ext.miny - ext.miny;

        let offsets: Vec<(Point, f64)> = vec![
            (Point::new(1.0*r, -star_ty + 1.2*r), 0.41),
            (Point::new(1.2*r, -star_ty + 0.9*r), 0.33),
            (Point::new(1.4*r, -star_ty + 0.0*r), 0.00),
            (Point::new(1.4*r, -star_ty + 0.5*r - 0.5*first_h), 0.04),
            (Point::new(1.4*r, -star_ty - 0.5*r - 0.5*first_h), 0.30),
            (Point::new(1.4*r, -star_ty + 0.0*r - first_h), 0.12),
            (Point::new(1.0*r, -star_ty - 1.0*r - first_h), 0.59),
            (Point::new(-1.2*r - text_width, -end_y + 1.0*r), 0.63),
            (Point::new(-1.3*r - text_width, -end_y + 0.5*r), 0.44),
            (Point::new(-1.4*r - text_width, -end_y + 0.0*r), 0.07),
            (Point::new(-1.4*r - text_width, -end_y + 0.5*r - 0.5*last_h), 0.10),
            (Point::new(-1.3*r - text_width, -end_y - 0.5*r - 0.5*last_h), 0.02),
            (Point::new(-1.3*r - text_width, -end_y + 0.0*r - last_h), 0.37),
            (Point::new(-(1.0/3.0)*text_width, 1.4*r), 0.70),
            (Point::new(-(1.0/3.0)*text_width, -1.4*r - text_height), 0.74),
            (Point::new(-0.5*text_width, 1.4*r), 0.67),
            (Point::new(-0.5*text_width, -1.4*r - text_height), 0.89),
            (Point::new(-(2.0/3.0)*text_width, -1.4*r - text_height), 0.74),
            (Point::new(-(2.0/3.0)*text_width, -1.4*r - text_height), 1.0),
        ];
        offsets
    }

    fn get_text_extents(&self, text: &str, pos: Point) -> Extents2d {
        let te = self.font_data.get_text_extents(text);
        let px = self.get_pixel_coordinates(pos);
        let minp = self.get_map_coordinates(Point::new(px.x + te.offx, px.y + te.offy));
        let maxp = self.get_map_coordinates(Point::new(px.x + te.offx + te.width, px.y + te.offy + te.height));
        Extents2d::new(minp.x, maxp.y, maxp.x, minp.y)
    }

    fn get_character_extents(&self, text: &str, pos: Point) -> Vec<Extents2d> {
        let ces = self.font_data.get_character_extents(text);
        let px = self.get_pixel_coordinates(pos);
        ces.iter().map(|ce| {
            let minp = self.get_map_coordinates(Point::new(px.x + ce.offx, px.y + ce.offy));
            let maxp = self.get_map_coordinates(Point::new(px.x + ce.offx + ce.width, px.y + ce.offy + ce.height));
            Extents2d::new(minp.x, maxp.y, maxp.x, minp.y)
        }).collect()
    }

    fn get_pixel_coordinates(&self, p: Point) -> Point {
        let nx = (p.x - self.extents.minx) / (self.extents.maxx - self.extents.minx);
        let ny = (p.y - self.extents.miny) / (self.extents.maxy - self.extents.miny);
        Point::new(self.img_width as f64 * nx, self.img_height as f64 * (1.0 - ny))
    }

    fn get_map_coordinates(&self, p: Point) -> Point {
        let nx = p.x / self.img_width as f64;
        let ny = 1.0 - p.y / self.img_height as f64;
        Point::new(
            self.extents.minx + nx * (self.extents.maxx - self.extents.minx),
            self.extents.miny + ny * (self.extents.maxy - self.extents.miny),
        )
    }

    fn normalize_map_coordinate(&self, p: Point) -> Point {
        Point::new(
            (p.x - self.extents.minx) / (self.extents.maxx - self.extents.minx),
            (p.y - self.extents.miny) / (self.extents.maxy - self.extents.miny),
        )
    }

    fn label_to_json(&self, c: &LabelCandidate, score: f64) -> Value {
        let npos = self.normalize_map_coordinate(c.position);
        let nmin = self.normalize_map_coordinate(Point::new(c.extents.minx, c.extents.miny));
        let nmax = self.normalize_map_coordinate(Point::new(c.extents.maxx, c.extents.maxy));
        let mut char_extents_flat = Vec::new();
        for ce in &c.char_extents {
            let cn_min = self.normalize_map_coordinate(Point::new(ce.minx, ce.miny));
            let cn_max = self.normalize_map_coordinate(Point::new(ce.maxx, ce.maxy));
            char_extents_flat.push(cn_min.x);
            char_extents_flat.push(cn_min.y);
            char_extents_flat.push(cn_max.x);
            char_extents_flat.push(cn_max.y);
        }
        json!({
            "text": c.text,
            "fontface": c.fontface,
            "fontsize": c.fontsize,
            "position": [npos.x, npos.y],
            "extents": [nmin.x, nmin.y, nmax.x, nmax.y],
            "charextents": char_extents_flat,
            "score": score,
        })
    }

    fn initialize_marker_label_scores(&mut self, labels: &mut Vec<Label>) {
        self.initialize_label_edge_scores(labels);
        self.initialize_label_marker_scores(labels, false);
        self.initialize_label_contour_scores(labels);
        self.initialize_label_river_scores(labels);
        self.initialize_label_border_scores(labels);
        self.initialize_label_base_scores(labels);
    }

    fn initialize_area_label_scores(&mut self, labels: &mut Vec<Label>) {
        self.initialize_area_orientation_scores(labels);
        self.initialize_label_edge_scores(labels);
        self.initialize_label_marker_scores(labels, true);
        self.initialize_label_contour_scores(labels);
        self.initialize_label_river_scores(labels);
        self.initialize_label_border_scores(labels);
        self.initialize_label_base_scores(labels);

        for label in labels.iter_mut() {
            label.candidates.sort_by(|a, b| a.base_score.partial_cmp(&b.base_score).unwrap());
            label.candidates.truncate(self.num_area_label_candidates);
        }
    }

    fn initialize_label_edge_scores(&self, labels: &mut Vec<Label>) {
        for label in labels.iter_mut() {
            for c in label.candidates.iter_mut() {
                c.edge_score = self.get_edge_score(c.extents);
            }
        }
    }

    fn get_edge_score(&self, ext: Extents2d) -> f64 {
        if !self.extents.contains_point(Point::new(ext.minx, ext.miny)) { return self.edge_score_penalty; }
        if !self.extents.contains_point(Point::new(ext.maxx, ext.maxy)) { return self.edge_score_penalty; }
        0.0
    }

    fn initialize_label_marker_scores(&self, labels: &mut Vec<Label>, is_area: bool) {
        let map_h = self.extents.maxy - self.extents.miny;
        let city_r_factor = if is_area { self.area_label_marker_radius_factor } else { self.label_marker_radius_factor };
        let town_r_factor = city_r_factor;
        let city_r = (self.city_marker_radius / self.img_height as f64) * map_h * city_r_factor;
        let town_r = (self.town_marker_radius / self.img_height as f64) * map_h * town_r_factor;

        let city_positions: Vec<Point> = self.cities.iter().map(|c| c.position).collect();
        let town_positions: Vec<Point> = self.towns.iter().map(|t| t.position).collect();

        for label in labels.iter_mut() {
            for c in label.candidates.iter_mut() {
                let mut count = 0;
                for &cp in &city_positions {
                    let me = Extents2d::new(cp.x-city_r, cp.y-city_r, cp.x+city_r, cp.y+city_r);
                    if extents_overlap(c.extents, me) { count += 1; }
                }
                for &tp in &town_positions {
                    let me = Extents2d::new(tp.x-town_r, tp.y-town_r, tp.x+town_r, tp.y+town_r);
                    if extents_overlap(c.extents, me) { count += 1; }
                }
                c.marker_score = count as f64 * self.marker_score_penalty;
            }
        }
    }

    fn initialize_label_contour_scores(&self, labels: &mut Vec<Label>) {
        let mut points = Vec::new();
        self.data_to_points(&self.contour_data.clone(), &mut points);
        if points.is_empty() { return; }
        let dx = self.spatial_grid_resolution_factor * self.resolution;
        let grid = SpatialPointGrid::new(&points, dx);
        for label in labels.iter_mut() {
            compute_penalty_scores(&mut label.candidates, &grid, self.min_contour_score_penalty, self.max_contour_score_penalty, |c: &mut LabelCandidate, s| c.contour_score = s);
        }
    }

    fn initialize_label_river_scores(&self, labels: &mut Vec<Label>) {
        let mut points = Vec::new();
        self.data_to_points(&self.river_data.clone(), &mut points);
        if points.is_empty() { return; }
        let dx = self.spatial_grid_resolution_factor * self.resolution;
        let grid = SpatialPointGrid::new(&points, dx);
        for label in labels.iter_mut() {
            compute_penalty_scores(&mut label.candidates, &grid, self.min_river_score_penalty, self.max_river_score_penalty, |c: &mut LabelCandidate, s| c.river_score = s);
        }
    }

    fn initialize_label_border_scores(&self, labels: &mut Vec<Label>) {
        let mut points = Vec::new();
        self.data_to_points(&self.border_data.clone(), &mut points);
        if points.is_empty() { return; }
        let dx = self.spatial_grid_resolution_factor * self.resolution;
        let grid = SpatialPointGrid::new(&points, dx);
        for label in labels.iter_mut() {
            compute_penalty_scores(&mut label.candidates, &grid, self.min_border_score_penalty, self.max_border_score_penalty, |c: &mut LabelCandidate, s| c.border_score = s);
        }
    }

    fn initialize_area_orientation_scores(&self, labels: &mut Vec<Label>) {
        let face_positions = self.compute_face_positions();
        let nf = self.voronoi.faces.len();
        let is_in_map: Vec<bool> = (0..nf).map(|i| self.extents.contains_point(face_positions[i])).collect();

        for label in labels.iter_mut() {
            if label.candidates.is_empty() { continue; }
            let territory_id = label.candidates[0].city_id;
            if territory_id < 0 { continue; }

            let mut territory_pts = Vec::new();
            let mut water_pts = Vec::new();
            let mut enemy_pts = Vec::new();
            let mut com = Point::new(0.0, 0.0);

            for i in 0..nf {
                if !is_in_map[i] { continue; }
                let id = if i < self.territory_data.len() { self.territory_data[i] } else { -1 };
                if id == territory_id {
                    territory_pts.push(face_positions[i]);
                    com.x += face_positions[i].x;
                    com.y += face_positions[i].y;
                } else if id == -1 {
                    water_pts.push(face_positions[i]);
                } else {
                    enemy_pts.push(face_positions[i]);
                }
            }

            if territory_pts.is_empty() { continue; }
            com.x /= territory_pts.len() as f64;
            com.y /= territory_pts.len() as f64;
            let max_dist_sq = territory_pts.iter()
                .map(|p| { let dx = com.x-p.x; let dy = com.y-p.y; dx*dx+dy*dy })
                .fold(0.0f64, f64::max);
            let territory_radius = max_dist_sq.sqrt().max(1e-6);

            let dx = self.spatial_grid_resolution_factor * self.resolution;
            let terr_grid = SpatialPointGrid::new(&territory_pts, dx);
            let enemy_grid = SpatialPointGrid::new(&enemy_pts, dx);
            let water_grid = SpatialPointGrid::new(&water_pts, dx);

            for c in label.candidates.iter_mut() {
                let tc = get_label_point_count(c, &terr_grid);
                let ec = get_label_point_count(c, &enemy_grid);
                let wc = get_label_point_count(c, &water_grid);
                let total = (tc + ec + wc) as f64;
                let score = if total > 0.0 {
                    (tc as f64 / total) * self.territory_score +
                    (ec as f64 / total) * self.enemy_score +
                    (wc as f64 / total) * self.water_score
                } else { 0.0 };
                let ext = c.extents;
                let center = Point::new(0.5*(ext.maxx+ext.minx), 0.5*(ext.maxy+ext.miny));
                let dist = point_distance(center, com) / territory_radius;
                c.orientation_score = score + dist;
            }
        }
    }

    fn initialize_label_base_scores(&self, labels: &mut Vec<Label>) {
        for label in labels.iter_mut() {
            for c in label.candidates.iter_mut() {
                c.base_score = (c.orientation_score + c.edge_score + c.marker_score +
                    c.contour_score + c.river_score + c.border_score) / 6.0;
            }
        }
    }

    fn data_to_points(&self, data: &Vec<Vec<f64>>, points: &mut Vec<Point>) {
        let w = self.extents.maxx - self.extents.minx;
        let h = self.extents.maxy - self.extents.miny;
        let eps = 1e-9;
        for path in data {
            if path.len() < 2 { continue; }
            let last_i = path.len() - 2;
            let is_loop = (path[0] - path[last_i]).abs() < eps && (path[1] - path[last_i + 1]).abs() < eps;
            let end = if is_loop { last_i } else { path.len() };
            let mut i = 0;
            while i + 1 < end {
                let x = self.extents.minx + path[i] * w;
                let y = self.extents.miny + path[i+1] * h;
                points.push(Point::new(x, y));
                i += 2;
            }
        }
    }

    fn generate_label_placements(&mut self, labels: &mut Vec<Label>) {
        // Randomize initial placements
        for label in labels.iter_mut() {
            if !label.candidates.is_empty() {
                label.candidate_idx = self.rng.rand() as usize % label.candidates.len();
            }
        }

        // Assign collision IDs
        let mut uid = 0usize;
        for label in labels.iter_mut() {
            for c in label.candidates.iter_mut() {
                c.parent_idx = 0; // will be set properly below
                c.collision_idx = uid;
                uid += 1;
            }
        }
        for (j, label) in labels.iter_mut().enumerate() {
            for c in label.candidates.iter_mut() {
                c.parent_idx = j;
            }
        }

        // Build collision data
        let all_candidates: Vec<(usize, usize, Extents2d)> = labels.iter().enumerate()
            .flat_map(|(j, label)| {
                label.candidates.iter().enumerate().map(move |(i, c)| (j, c.collision_idx, c.extents))
            }).collect();

        for label in labels.iter_mut() {
            for c in label.candidates.iter_mut() {
                c.collision_data.clear();
                for &(other_parent, other_coll_idx, other_ext) in &all_candidates {
                    if other_parent == c.parent_idx { continue; }
                    if extents_overlap(c.extents, other_ext) {
                        c.collision_data.push(other_coll_idx);
                    }
                }
            }
        }

        let max_id = uid;
        let mut score = self.calculate_placement_score(labels, max_id);

        let num_labels = labels.len();
        let mut temperature = self.initial_temperature;
        let mut temp_changes = 0;
        let mut repositionings = 0;
        let mut successful = 0;
        let max_successful = self.successful_repositioning_factor * num_labels as i32;
        let max_total = self.total_repositioning_factor * num_labels as i32;

        while temp_changes < self.max_temperature_changes {
            let rl = self.rng.rand() as usize % num_labels;
            let rc = self.rng.rand() as usize % labels[rl].candidates.len().max(1);
            let last_c = labels[rl].candidate_idx;
            labels[rl].candidate_idx = rc;

            let new_score = self.calculate_placement_score(labels, max_id);
            let diff = new_score - score;
            if diff < 0.0 && diff.abs() > 1e-9 {
                score = new_score;
                successful += 1;
            } else {
                let prob = 1.0 - (-diff / temperature).exp();
                let r = self.rng.random_double(0.0, 1.0);
                if r < prob {
                    labels[rl].candidate_idx = last_c;
                } else {
                    score = new_score;
                }
            }

            repositionings += 1;
            if successful > max_successful || repositionings > max_total {
                if successful == 0 { break; }
                temperature *= self.annealing_factor;
                temp_changes += 1;
                repositionings = 0;
                successful = 0;
            }
        }
    }

    fn calculate_placement_score(&self, labels: &[Label], max_id: usize) -> f64 {
        let mut is_active = vec![false; max_id + 1];
        for label in labels {
            if label.candidates.is_empty() { continue; }
            let cid = label.candidates[label.candidate_idx].collision_idx;
            if cid < is_active.len() { is_active[cid] = true; }
        }
        let mut sum = 0.0;
        let mut count = 0;
        for label in labels {
            if label.candidates.is_empty() { continue; }
            let c = &label.candidates[label.candidate_idx];
            let mut s = c.base_score;
            for &col_idx in &c.collision_data {
                if col_idx < is_active.len() && is_active[col_idx] {
                    s += self.overlap_score_penalty;
                }
            }
            sum += s;
            count += 1;
        }
        if count > 0 { sum / count as f64 } else { 0.0 }
    }

    // ----- main output -----
    pub fn get_draw_data(&mut self) -> String {
        self.ensure_eroded();

        let contour = if self.contour_enabled {
            let cd = self.get_contour_draw_data();
            self.contour_data = cd.clone();
            cd
        } else { Vec::new() };

        let river = if self.rivers_enabled {
            let rd = self.get_river_draw_data();
            self.river_data = rd.clone();
            rd
        } else { Vec::new() };

        let slope = if self.slopes_enabled {
            self.get_slope_draw_data()
        } else { Vec::new() };

        let city: Vec<f64> = if self.cities_enabled {
            let inv_w = 1.0 / (self.extents.maxx - self.extents.minx);
            let inv_h = 1.0 / (self.extents.maxy - self.extents.miny);
            self.cities.iter().flat_map(|c| {
                let nx = (c.position.x - self.extents.minx) * inv_w;
                let ny = (c.position.y - self.extents.miny) * inv_h;
                vec![nx, ny]
            }).collect()
        } else { Vec::new() };

        let town: Vec<f64> = if self.towns_enabled {
            let inv_w = 1.0 / (self.extents.maxx - self.extents.minx);
            let inv_h = 1.0 / (self.extents.maxy - self.extents.miny);
            self.towns.iter().flat_map(|t| {
                let nx = (t.position.x - self.extents.minx) * inv_w;
                let ny = (t.position.y - self.extents.miny) * inv_h;
                vec![nx, ny]
            }).collect()
        } else { Vec::new() };

        let territory = {
            let td = self.get_territory_draw_data();
            self.border_data = td.clone();
            if self.borders_enabled { td } else { Vec::new() }
        };

        let label: Vec<Value> = if self.labels_enabled {
            self.get_label_draw_data()
        } else { Vec::new() };

        let output = json!({
            "image_width": self.img_width,
            "image_height": self.img_height,
            "draw_scale": self.draw_scale,
            "contour": contour,
            "river": river,
            "slope": slope,
            "city": city,
            "town": town,
            "territory": territory,
            "label": label,
        });

        serde_json::to_string(&output).unwrap()
    }
}

fn point_distance(a: Point, b: Point) -> f64 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx*dx + dy*dy).sqrt()
}

fn extents_overlap(a: Extents2d, b: Extents2d) -> bool {
    a.minx < b.maxx && a.maxx > b.minx && a.miny < b.maxy && a.maxy > b.miny
}

/// Smooths a sequence of (x, y) positions using a Laplacian filter:
/// v1 = (1-factor)*v1 + factor*0.5*(v0 + v2)
fn smooth_positions(positions: Vec<(f64, f64)>, factor: f64) -> Vec<(f64, f64)> {
    if positions.len() < 3 {
        return positions;
    }
    let mut out = positions.clone();
    for i in 1..positions.len() - 1 {
        let (x0, y0) = positions[i - 1];
        let (x1, y1) = positions[i];
        let (x2, y2) = positions[i + 1];
        out[i] = (
            (1.0 - factor) * x1 + factor * 0.5 * (x0 + x2),
            (1.0 - factor) * y1 + factor * 0.5 * (y0 + y2),
        );
    }
    out
}

fn get_label_point_count(c: &LabelCandidate, grid: &SpatialPointGrid) -> usize {
    c.char_extents.iter().map(|&ce| grid.get_point_count(ce)).sum()
}

fn compute_penalty_scores(
    candidates: &mut Vec<LabelCandidate>,
    grid: &SpatialPointGrid,
    min_penalty: f64,
    max_penalty: f64,
    setter: impl Fn(&mut LabelCandidate, f64),
) {
    let counts: Vec<usize> = candidates.iter().map(|c| get_label_point_count(c, grid)).collect();
    let nonzero: Vec<usize> = counts.iter().filter(|&&c| c > 0).copied().collect();
    if nonzero.is_empty() {
        for c in candidates.iter_mut() { setter(c, 0.0); }
        return;
    }
    let min_c = *nonzero.iter().min().unwrap();
    let max_c = *nonzero.iter().max().unwrap();
    for (i, c) in candidates.iter_mut().enumerate() {
        let score = if counts[i] == 0 {
            0.0
        } else if max_c > min_c {
            let f = (counts[i] - min_c) as f64 / (max_c - min_c) as f64;
            min_penalty + f * (max_penalty - min_penalty)
        } else {
            max_penalty
        };
        setter(c, score);
    }
}

// Override smooth_path to return smoothed vertex position indices
// Since we're working with vertex indices, smoothing the actual positions in draw data
// The river/contour draw data already handles this. For territory borders, also same approach.
// The C++ does a smooth_path that takes vertex positions, we need to do this at the draw stage.
// Let me update the draw data functions to smooth the actual coordinates:
