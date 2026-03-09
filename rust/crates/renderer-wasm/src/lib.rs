//! WASM bridge adapter for Fantasy Map Generator.
//!
//! Exposes the core map-generation algorithm to JavaScript via
//! WebAssembly linear memory (zero-copy buffers).
//!
//! Build with:
//! ```sh
//! wasm-pack build crates/renderer-wasm --target web --features wasm
//! ```

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

use fantasy_map_core::{
    map_data::{MapAdapter, MapData},
    extents2d::Extents2d,
    rand::GlibcRand,
    map_generator::MapGenerator,
};

// ── Zero-Copy Buffer Layout ────────────────────────────────────────────────
//
// The WASM module exposes raw pointers into its linear memory so that the
// JS side can create typed-array views without copying.
//
// Vertex buffer:  Float32Array  [x0,y0, x1,y1, … ]  (cities + towns)
// Index buffer:   Uint32Array   [i0, i1, i2, … ]     (path point indices)
// Attrib buffer:  Float32Array  [r,g,b,a, … ]        (one rgba per vertex)

/// In-memory representation of a generated map's GPU-ready buffers.
pub struct WasmMapBuffers {
    /// Interleaved x,y positions for all geometry points (Float32).
    pub vertices: Vec<f32>,
    /// Point indices for all polyline segments (Uint32).
    pub indices: Vec<u32>,
    /// RGBA colour per vertex (Float32, 4 components each).
    pub colors: Vec<f32>,
    /// Original MapData for non-GPU uses.
    pub map_data: MapData,
}

/// WASM adapter that converts `MapData` into GPU-ready linear-memory buffers.
pub struct WasmAdapter;

impl MapAdapter for WasmAdapter {
    type Output = WasmMapBuffers;

    fn render(&self, data: &MapData) -> WasmMapBuffers {
        let mut vertices: Vec<f32> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut colors: Vec<f32> = Vec::new();

        let w = data.image_width as f32;
        let h = data.image_height as f32;

        let mut push_vertex = |x: f32, y: f32, r: f32, g: f32, b: f32, a: f32| {
            let idx = (vertices.len() / 2) as u32;
            vertices.push(x * w);
            vertices.push(y * h);
            colors.push(r);
            colors.push(g);
            colors.push(b);
            colors.push(a);
            idx
        };

        // ── Contour lines (teal) ────────────────────────────────────────
        for path in &data.contour {
            let mut i = 0;
            while i + 1 < path.len() {
                let vi = push_vertex(path[i] as f32, path[i + 1] as f32, 0.53, 0.60, 0.67, 1.0);
                if i > 0 {
                    indices.push(vi - 1);
                    indices.push(vi);
                }
                i += 2;
            }
        }

        // ── Rivers (blue) ────────────────────────────────────────────────
        for path in &data.river {
            let mut i = 0;
            while i + 1 < path.len() {
                let vi = push_vertex(path[i] as f32, path[i + 1] as f32, 0.27, 0.53, 0.80, 1.0);
                if i > 0 {
                    indices.push(vi - 1);
                    indices.push(vi);
                }
                i += 2;
            }
        }

        // ── Territory borders (red) ──────────────────────────────────────
        for path in &data.territory {
            let mut i = 0;
            while i + 1 < path.len() {
                let vi = push_vertex(path[i] as f32, path[i + 1] as f32, 0.80, 0.27, 0.27, 1.0);
                if i > 0 {
                    indices.push(vi - 1);
                    indices.push(vi);
                }
                i += 2;
            }
        }

        // ── City markers (dark) ──────────────────────────────────────────
        let mut ci = 0;
        while ci + 1 < data.city.len() {
            push_vertex(data.city[ci] as f32, data.city[ci + 1] as f32, 0.13, 0.13, 0.20, 1.0);
            ci += 2;
        }

        // ── Town markers ──────────────────────────────────────────────────
        let mut ti = 0;
        while ti + 1 < data.town.len() {
            push_vertex(data.town[ti] as f32, data.town[ti + 1] as f32, 0.27, 0.33, 0.40, 1.0);
            ti += 2;
        }

        WasmMapBuffers {
            vertices,
            indices,
            colors,
            map_data: data.clone(),
        }
    }
}

// ── WASM-bindgen public API ────────────────────────────────────────────────

/// Handle to a generated map, kept alive on the WASM heap.
///
/// JS can call the pointer-getter methods to create zero-copy typed-array views.
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct WasmMapHandle {
    buffers: WasmMapBuffers,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WasmMapHandle {
    /// Number of f32 values in the vertex buffer.
    pub fn vertex_count(&self) -> u32 {
        self.buffers.vertices.len() as u32
    }

    /// Number of u32 values in the index buffer.
    pub fn index_count(&self) -> u32 {
        self.buffers.indices.len() as u32
    }

    /// Pointer to the vertex buffer (f32 array, length = vertex_count()).
    ///
    /// **SAFETY**: Valid only while this `WasmMapHandle` is alive.
    pub fn vertex_ptr(&self) -> *const f32 {
        self.buffers.vertices.as_ptr()
    }

    /// Pointer to the index buffer (u32 array, length = index_count()).
    ///
    /// **SAFETY**: Valid only while this `WasmMapHandle` is alive.
    pub fn index_ptr(&self) -> *const u32 {
        self.buffers.indices.as_ptr()
    }

    /// Pointer to the colour buffer (f32 array, length = vertex_count() * 4).
    ///
    /// **SAFETY**: Valid only while this `WasmMapHandle` is alive.
    pub fn color_ptr(&self) -> *const f32 {
        self.buffers.colors.as_ptr()
    }

    /// Image width from the map data.
    pub fn image_width(&self) -> u32 {
        self.buffers.map_data.image_width
    }

    /// Image height from the map data.
    pub fn image_height(&self) -> u32 {
        self.buffers.map_data.image_height
    }

    /// Serialise the full `MapData` as a JSON string (for non-GPU use).
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.buffers.map_data).unwrap_or_default()
    }
}

/// Generate a map and return a handle to its WASM-memory buffers.
///
/// # Arguments
/// * `seed` - Random seed for map generation.
/// * `img_width` / `img_height` - Output image dimensions in pixels.
/// * `resolution` - Poisson-disc sampling distance (0.05–0.15 typical).
/// * `num_cities` / `num_towns` - Entity counts (-1 for random defaults).
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn generate_map(
    seed: u32,
    img_width: u32,
    img_height: u32,
    resolution: f64,
    num_cities: i32,
    num_towns: i32,
) -> WasmMapHandle {
    let mut rng = GlibcRand::new(seed);
    for _ in 0..1000 {
        rng.rand();
    }

    let default_height = 20.0f64;
    let aspect = img_width as f64 / img_height as f64;
    let extents = Extents2d::new(0.0, 0.0, aspect * default_height, default_height);

    let mut map = MapGenerator::new(extents, resolution, img_width, img_height, rng);
    map.initialize();

    // Minimal heightmap
    let pad = 5.0;
    let ext = map.get_extents();
    let n = map.rng_mut().random_double(100.0, 200.0) as i32;
    for _ in 0..n {
        let _discard = map.rng_mut().random_double(ext.minx - pad, ext.maxx + pad); // advance RNG state
        let px = map.rng_mut().random_double(ext.minx - pad, ext.maxx + pad);
        let py = map.rng_mut().random_double(ext.miny - pad, ext.maxy + pad);
        let r = map.rng_mut().random_double(1.0, 8.0);
        let s = map.rng_mut().random_double(0.5, 1.5);
        map.add_hill(px, py, r, s);
    }
    map.normalize();

    for _ in 0..3 {
        map.erode(0.1);
    }
    map.set_sea_level_to_median();

    let nc = if num_cities >= 0 { num_cities } else { 4 };
    let nt = if num_towns >= 0 { num_towns } else { 12 };

    for i in 0..nc {
        map.add_city(format!("City {}", i + 1), format!("REGION {}", i + 1));
    }
    for i in 0..nt {
        map.add_town(format!("Town {}", i + 1));
    }

    let map_data = map.get_map_data();
    let adapter = WasmAdapter;
    let buffers = adapter.render(&map_data);

    WasmMapHandle { buffers }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fantasy_map_core::map_data::{LabelData, MapData};

    fn sample_data() -> MapData {
        MapData {
            image_width: 800,
            image_height: 600,
            draw_scale: 1.0,
            contour: vec![vec![0.1, 0.1, 0.2, 0.2]],
            river: vec![vec![0.4, 0.5, 0.5, 0.6]],
            slope: vec![0.1, 0.2, 0.15, 0.25],
            city: vec![0.5, 0.5],
            town: vec![0.3, 0.3],
            territory: vec![vec![0.0, 0.0, 1.0, 1.0]],
            label: vec![],
        }
    }

    #[test]
    fn test_wasm_adapter_produces_buffers() {
        let adapter = WasmAdapter;
        let buffers = adapter.render(&sample_data());
        assert!(!buffers.vertices.is_empty(), "Vertex buffer must not be empty");
        assert!(!buffers.indices.is_empty(), "Index buffer must not be empty");
        assert_eq!(buffers.colors.len(), buffers.vertices.len() * 2,
            "Color buffer should have 4 components per vertex (vertices has 2 floats per point)");
    }

    #[test]
    fn test_vertex_index_consistency() {
        let adapter = WasmAdapter;
        let buffers = adapter.render(&sample_data());
        let vertex_point_count = (buffers.vertices.len() / 2) as u32;
        for &idx in &buffers.indices {
            assert!(idx < vertex_point_count, "Index {} out of bounds (max {})", idx, vertex_point_count);
        }
    }
}
