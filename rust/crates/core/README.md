# fantasy-map-core

The **core** crate is the heart of the Fantasy Map Generator.  It contains the
complete terrain-generation algorithm (ported from the original C++ codebase)
and defines the data contracts (`MapData`, `MapAdapter`) used by every renderer
plug-in.

---

## What this crate provides

| Item | Description |
|---|---|
| `MapData` | Typed output struct with normalised `[0,1]` coordinates |
| `LabelData` | Per-label layout data (position, font, extents, score) |
| `MapAdapter` | Trait that all renderer plug-ins must implement |
| `MapGenerator` | The stateful terrain generation engine |
| `CITY_DATA` | Bundled city-name JSON for label generation |
| `map_generation` binary | Legacy CLI that outputs JSON (kept for backward compat) |

---

## Algorithm overview

1. **Voronoi grid** — Poisson-disc sampling → Delaunay triangulation → dual
   Voronoi diagram (DCEL representation).
2. **Height map** — Random hills, cones and slope primitives; normalised and
   relaxed.
3. **Erosion** — Flux-based erosion over multiple iterations, finishing with
   sea-level adjustment.
4. **Hydrology** — Rivers traced from high-flux cells to the sea using the
   Planchon-Darboux depression-filling algorithm.
5. **Settlements** — Cities and towns placed on scored land cells; territory
   borders computed via movement-cost Dijkstra.
6. **Labels** — Simulated-annealing placement optimiser minimises overlap with
   contours, rivers, and borders.
7. **Output** — All geometry is normalised to `[0, 1]` and packaged into
   `MapData`.

---

## Key types

```rust
// The data contract — produced by MapGenerator, consumed by adapters
pub struct MapData {
    pub image_width:  u32,
    pub image_height: u32,
    pub draw_scale:   f64,
    pub contour:    Vec<Vec<f64>>,   // polylines [x0,y0, x1,y1, …]
    pub river:      Vec<Vec<f64>>,
    pub slope:      Vec<f64>,        // flat line segments [x0,y0,x1,y1, …]
    pub city:       Vec<f64>,        // flat points [x0,y0, x1,y1, …]
    pub town:       Vec<f64>,
    pub territory:  Vec<Vec<f64>>,
    pub label:      Vec<LabelData>,
}

// The plugin contract
pub trait MapAdapter {
    type Output;
    fn render(&self, data: &MapData) -> Self::Output;
}
```

---

## Build commands

```bash
# Build the library
cargo build -p fantasy-map-core

# Run unit + integration tests
cargo test -p fantasy-map-core

# Run the legacy JSON binary
cargo run -p fantasy-map-core --bin map_generation -- --seed 42 --output out.json

# Build documentation
cargo doc -p fantasy-map-core --open
```

---

## Public API

### `MapGenerator`

```rust
// Create and configure
let mut gen = MapGenerator::new(extents, resolution, width, height, rng);
gen.set_draw_scale(1.0);
gen.disable_rivers();   // toggle any feature before generating

// Build terrain
gen.initialize();
gen.add_hill(px, py, radius, height);
gen.normalize();
gen.erode(amount);
gen.set_sea_level_to_median();

// Add settlements
gen.add_city("Ironhold".into(), "NORDMARK".into());
gen.add_town("Millbrook".into());

// Retrieve output
let map_data: MapData = gen.get_map_data();   // typed struct (preferred)
let json_str: String  = gen.get_draw_data();  // legacy JSON string
```

### `CITY_DATA`

```rust
// Access the bundled city-name JSON at runtime
let json: serde_json::Value = serde_json::from_str(fantasy_map_core::CITY_DATA)?;
```

---

## Testing

The crate has two test layers:

| Layer | Location | What it checks |
|---|---|---|
| Unit tests | Inside each source file | Individual algorithm steps |
| Snapshot test | `tests/snapshot_test.rs` | Full binary output (JSON schema + defaults) |

Run all tests:

```bash
cargo test -p fantasy-map-core
```

---

## Design constraints

* **Pure library** — no I/O, no `std::process::exit`, no printing to stdout.
* **Deterministic** — given the same seed, the output must be byte-identical
  across platforms.  The `GlibcRand` implementation matches the C++ reference.
* **No WASM dependencies** — the core crate compiles to native and WASM without
  any platform-specific code.
