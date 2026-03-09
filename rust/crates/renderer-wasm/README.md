# fantasy-map-renderer-wasm

WebAssembly bridge adapter for the Fantasy Map Generator.

Exposes the core map-generation algorithm to JavaScript via WebAssembly linear
memory, enabling **zero-copy** typed-array views from the browser's
`Float32Array` / `Uint32Array` API.

---

## How it works

```
Rust (MapData)
    │
    ▼
WasmAdapter::render()  →  WasmMapBuffers
    │                       ├── vertices:  Vec<f32>  (x,y pairs, pixel space)
    │                       ├── indices:   Vec<u32>  (line-segment index pairs)
    │                       └── colors:    Vec<f32>  (RGBA per vertex)
    │
    ▼
WasmMapHandle  (kept alive in WASM heap)
    ├── vertex_ptr() → *const f32   ─┐
    ├── index_ptr()  → *const u32    ├─ JS creates typed-array views here
    └── color_ptr()  → *const f32   ─┘
```

The generated buffer layout is designed for direct submission to WebGPU
`BufferAttribute` objects without any copying.

---

## Build commands

### Native (unit tests only — no wasm-bindgen needed)

```bash
cargo build -p fantasy-map-renderer-wasm
cargo test  -p fantasy-map-renderer-wasm
```

### WebAssembly (requires `wasm-pack`)

```bash
# Install wasm-pack (once)
cargo install wasm-pack

# Release build → outputs to apps/web/public/wasm/
./scripts/build-wasm.sh

# Debug build (faster, larger output)
./scripts/build-wasm.sh --dev

# Or via npm inside apps/web
npm run wasm:build
npm run wasm:build:dev
```

---

## wasm-pack output

After a successful build the following files appear in `apps/web/public/wasm/`:

| File | Description |
|---|---|
| `fantasy_map_renderer_wasm.js` | ES-module glue (import + call `init()`) |
| `fantasy_map_renderer_wasm_bg.wasm` | Compiled WebAssembly binary |
| `fantasy_map_renderer_wasm.d.ts` | TypeScript type declarations |
| `package.json` | wasm-pack metadata (informational) |

These files are **excluded from git** (see `apps/web/.gitignore`).

---

## JavaScript API

```typescript
import init, { generate_map } from '/wasm/fantasy_map_renderer_wasm.js';

// Initialise once at startup
await init('/wasm/fantasy_map_renderer_wasm_bg.wasm');

// Generate a map (returns a handle into WASM linear memory)
const handle = generate_map(
  42,       // seed
  1920,     // imgWidth
  1080,     // imgHeight
  0.08,     // resolution
  5,        // numCities  (-1 = random)
  15,       // numTowns   (-1 = random)
);

// ── Option A: get the full MapData as JSON ─────────────────────
const mapData = JSON.parse(handle.to_json());

// ── Option B: zero-copy GPU buffer views ──────────────────────
const { buffer } = WebAssembly.instance.exports.memory as WebAssembly.Memory;
const vertices = new Float32Array(buffer, handle.vertex_ptr(), handle.vertex_count());
const indices  = new Uint32Array (buffer, handle.index_ptr(),  handle.index_count());

// Always free the handle when done
handle.free();
```

The web app integrates this through `src/wasm-bridge.ts`, which also provides
automatic fallback to static JSON when the WASM artefacts are not present.

---

## Cargo features

| Feature | Default | Description |
|---|---|---|
| `wasm` | ❌ | Enables `wasm-bindgen` exports (`generate_map`, `WasmMapHandle`) |

The `wasm` feature must be passed explicitly when building with `wasm-pack`:

```toml
# Cargo.toml excerpt
[features]
wasm = ["wasm-bindgen", "js-sys", "web-sys"]
```

This ensures that `wasm-bindgen` is **not** compiled into native binaries,
keeping the native test build fast and dependency-light.

---

## `WasmMapHandle` methods

| Method | Return | Description |
|---|---|---|
| `vertex_count()` | `u32` | Number of `f32` values in vertex buffer |
| `index_count()` | `u32` | Number of `u32` values in index buffer |
| `vertex_ptr()` | `*const f32` | Pointer into linear memory |
| `index_ptr()` | `*const u32` | Pointer into linear memory |
| `color_ptr()` | `*const f32` | Pointer into linear memory (4 components/vertex) |
| `image_width()` | `u32` | Map image width |
| `image_height()` | `u32` | Map image height |
| `to_json()` | `String` | Full `MapData` as a JSON string |
| `free()` | — | Release WASM heap allocation |

---

## Buffer layout

### Vertex buffer (`Float32Array`)

```
[ x0, y0,  x1, y1,  x2, y2, … ]
```

Coordinates are in **pixel space** (multiplied by `image_width` / `image_height`).

### Index buffer (`Uint32Array`)

```
[ a0, b0,  a1, b1, … ]   ← pairs of vertex indices forming line segments
```

### Colour buffer (`Float32Array`)

```
[ r0, g0, b0, a0,  r1, g1, b1, a1, … ]   ← RGBA in [0, 1] per vertex
```

Colour encodes feature type (contour = teal, river = blue, territory = red,
city = dark, town = mid-grey).
