# C++ to Rust (WebGPU) Migration Documentation

## Overview

This document records the process of migrating the Fantasy Map Generator from C++ with Python/Cairo rendering to native Rust with WebGPU rendering.

## Migration Approach

Following a Test-Driven Development (TDD) approach:

1. **Baseline Establishment**: Generate reference outputs from C++ implementation
2. **Test Harness**: Create automated comparison infrastructure
3. **Iterative Implementation**: Migrate components one at a time, validating against baseline
4. **Rendering Migration**: Convert Cairo rendering commands to WebGPU

## Technology Stack Mapping

### C++ → Rust Core Dependencies

| C++ Component | Rust Equivalent | Notes |
|--------------|-----------------|-------|
| `jsoncons` | `serde` + `serde_json` | JSON serialization |
| `Argtable3` | `clap` | CLI argument parsing |
| `std::rand()` | `rand` + `rand_chacha` | Deterministic RNG with ChaCha8 |
| Python/PyCairo | `wgpu` + `image` | WebGPU rendering |
| Manual memory | Rust ownership | Memory safety by design |
| `std::vector` | `Vec<T>` | Dynamic arrays |
| `std::map` | `std::collections::HashMap` | Hash maps |

### Rendering Architecture

| C++ (Cairo) | Rust (WebGPU) | Mapping |
|------------|---------------|---------|
| `cairo_context` | `wgpu::RenderPass` | Drawing context |
| `cairo_surface` | `wgpu::Texture` | Render target |
| `cairo_line_to` | Custom vertex buffers | Line rendering |
| `cairo_stroke` | Fragment shader | Stroke operations |
| `cairo_fill` | Fragment shader | Fill operations |
| `cairo_set_source_rgb` | Uniform buffers | Color values |
| `cairo_set_line_width` | Vertex shader | Line width |
| `cairo_arc` | Geometry generation | Circle primitives |
| `cairo_show_text` | Texture atlas + instancing | Text rendering |

## Data Structure Migration

### Core Geometry

**C++ (`geometry.h`)**
```cpp
namespace dcel {
    struct Point {
        double x, y;
        Point(double x, double y) : x(x), y(y) {}
    };
}
```

**Rust (`geometry.rs`)**
```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
```

### DCEL (Doubly Connected Edge List)

**Key Concepts to Migrate:**
- Half-edge data structure
- Vertex, Edge, Face relationships
- Circular linked lists
- Graph traversal operations

**Memory Management:**
- C++: Manual pointer management with raw pointers
- Rust: Arena allocation with indices instead of pointers (safe alternative)

### Algorithm Mappings

#### 1. Poisson Disc Sampling

**C++**: `poissondiscsampler.cpp` (Bridson's algorithm)
- Grid-based spatial partitioning
- Active list for candidate points
- Radius constraint checking

**Rust**: To implement in `poisson_disc.rs`
- Use `Vec<Vec<Option<usize>>>` for grid
- Use `Vec<usize>` for active list
- Same algorithm, idiomatic Rust

#### 2. Delaunay Triangulation

**C++**: `delaunay.cpp` (Incremental construction)
- Point insertion
- Triangle flip operations
- Circumcircle tests

**Rust**: To implement in `delaunay.rs`
- Use indices instead of pointers
- Leverage Rust's type system for edge cases
- Maintain determinism with explicit ordering

#### 3. Voronoi Diagram

**C++**: `voronoi.cpp` (Dual of Delaunay)
- Circumcenter computation
- Face-to-vertex mapping

**Rust**: To implement in `voronoi.rs`
- Direct translation possible
- Use nalgebra for geometric computations

#### 4. Terrain Generation

**C++ Primitives** (`mapgenerator.cpp`):
- `addHill` - Smooth falloff
- `addCone` - Linear falloff
- `addSlope` - Directional gradient
- `erode` - Hydraulic erosion simulation

**Rust**: To implement in `map_generator.rs`
- Same mathematical operations
- Use iterators for efficiency
- Leverage SIMD where applicable

#### 5. Label Placement

**C++**: Simulated annealing optimization
- Candidate generation
- Overlap detection
- Temperature scheduling

**Rust**: Direct algorithm translation
- Use `rand_chacha` for deterministic randomness
- Same annealing parameters

## Shader Migration (Cairo → WGSL)

### Slope Shading

**Cairo Approach** (CPU-side):
```python
for segment in slope_segments:
    cairo_set_line_width(ctx, segment.width)
    cairo_set_source_rgba(ctx, 0, 0, 0, segment.alpha)
    cairo_line_to(ctx, segment.x1, segment.y1)
    cairo_line_to(ctx, segment.x2, segment.y2)
    cairo_stroke(ctx)
```

**WebGPU Approach** (GPU-side):
```wgsl
// Vertex Shader
@vertex
fn vs_main(@location(0) position: vec2<f32>) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(position, 0.0, 1.0);
    return out;
}

// Fragment Shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 0.0, alpha);
}
```

### Rivers

**Cairo**: Bezier curves with varying width
**WebGPU**: Tesselated polylines with custom vertex shader for width

### Contours

**Cairo**: Simple line drawing
**WebGPU**: Instanced line rendering

### Borders

**Cairo**: Dashed line patterns
**WebGPU**: Fragment shader with distance-based dashing

### City/Town Markers

**Cairo**: Concentric circles
**WebGPU**: Circle geometry or point sprites

### Text Labels

**Cairo**: Built-in text rendering
**WebGPU**: Texture atlas with glyph quads (requires font rasterization)

## Testing Strategy

### JSON Comparison

**Tool**: `serde_json` + `approx` crate

**Validation**:
```rust
use approx::assert_relative_eq;

fn compare_json_floats(baseline: f64, actual: f64) {
    assert_relative_eq!(baseline, actual, epsilon = 1e-5);
}
```

### Image Comparison

**Tool**: `image` crate + pixel-by-pixel diff

**Metrics**:
- Mean Squared Error (MSE)
- Structural Similarity Index (SSIM)
- Perceptual diff threshold

**Acceptance Criteria**:
- JSON structure: 100% match
- JSON float values: ≤ 0.00001 difference
- Image pixels: ≤ 1% different pixels, ≤ 5 color difference per channel

## Determinism Guarantees

### Random Number Generation

**C++**: `srand(seed)` + `rand()`
- Platform-dependent
- Not guaranteed deterministic

**Rust**: `ChaCha8Rng::seed_from_u64(seed)`
- Platform-independent
- Cryptographically deterministic

**Compatibility**:
- May need to match C++ rand() behavior
- Option 1: Port C++ rand() implementation
- Option 2: Accept slight differences, establish new baseline

### Floating-Point

**Issues**:
- IEEE 754 compliance
- Different compiler optimizations
- SIMD instruction differences

**Strategy**:
- Use same precision (f64)
- Avoid `fma` instruction differences
- Test on multiple platforms

## Build System

### C++ Build

```bash
mkdir build && cd build
cmake ..
make
```

### Rust Build

```bash
cargo build --release
```

### Cross-Compilation

Rust supports trivial cross-compilation:
```bash
cargo build --target wasm32-unknown-unknown  # WebAssembly
cargo build --target x86_64-pc-windows-gnu   # Windows
cargo build --target aarch64-apple-darwin    # Apple Silicon
```

## Performance Expectations

| Component | C++ Performance | Rust Expected | Notes |
|-----------|----------------|---------------|-------|
| Delaunay | Baseline | +0-10% | Similar algorithmic complexity |
| Erosion | Baseline | +5-15% | SIMD optimization potential |
| Rendering | Cairo (slow) | WebGPU (10-100x) | GPU acceleration |
| Memory | Manual (risky) | Safe (zero-cost) | No runtime overhead |
| Compilation | Fast | Slower | Rust compile times longer |

## WebGPU Rendering Pipeline

### Initialization

```rust
let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
let adapter = instance.request_adapter(...).await;
let device = adapter.request_device(...).await;
let queue = device.queue();
```

### Render Pass Flow

1. Create texture (render target)
2. Create render pass
3. Bind vertex buffers
4. Bind uniform buffers (colors, transforms)
5. Draw calls
6. Submit command buffer
7. Read texture to CPU
8. Encode as PNG

## Challenges and Solutions

### Challenge 1: DCEL Pointer Management

**C++ Approach**: Raw pointers with manual lifecycle
**Rust Solution**: Arena with indices (`Vec<Edge>` with `EdgeId` type aliases)

### Challenge 2: Cairo's Stateful API

**C++ Issue**: Cairo maintains graphics state
**Rust Solution**: Explicit state management, batch rendering

### Challenge 3: Text Rendering

**C++ Solution**: Cairo's built-in text
**Rust Solution**:
- Option 1: Pre-rendered font atlas
- Option 2: Use `rusttype` or `ab_glyph` crate
- Option 3: SDF (Signed Distance Field) fonts

### Challenge 4: JSON Float Precision

**Issue**: Floating-point arithmetic may differ
**Solution**: Tolerance-based comparison (epsilon = 1e-5)

## Migration Status

- [x] Project scaffolding
- [x] Baseline generation (C++)
- [ ] Geometry primitives
- [ ] DCEL implementation
- [ ] Poisson Disc Sampling
- [ ] Delaunay Triangulation
- [ ] Voronoi Diagram
- [ ] Terrain Generation
- [ ] Erosion Simulation
- [ ] City/Town Generation
- [ ] Label Placement
- [ ] WebGPU Rendering
- [ ] JSON Validation Tests
- [ ] Image Validation Tests

## References

- [wgpu Tutorial](https://sotrh.github.io/learn-wgpu/)
- [WebGPU Specification](https://www.w3.org/TR/webgpu/)
- [Delaunay Triangulation (Berg et al.)](https://www.springer.com/gp/book/9783540779735)
- [Poisson Disc Sampling (Bridson)](https://www.cs.ubc.ca/~rbridson/docs/bridson-siggraph07-poissondisk.pdf)
- [Martin O'Leary's Fantasy Map Generation](https://mewo2.com/notes/terrain/)
