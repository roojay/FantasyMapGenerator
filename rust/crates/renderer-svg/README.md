# fantasy-map-renderer-svg

High-performance SVG rendering adapter for the Fantasy Map Generator.

Implements `MapAdapter<Output = String>` and converts a `MapData` value into a
production-quality, fully-layered SVG document.

---

## Features

* **Path merging** — adjacent polylines of the same type are joined into a
  single `<path>` element, dramatically reducing DOM node count and file size.
* **Layer groups** — every feature type lives in its own `<g id="…">` element,
  making it easy to hide/show layers in any SVG viewer or editor.
* **Coordinate precision** — configurable number of decimal places (default 2)
  via `SvgConfig::coord_precision`.
* **Zero dependencies** — only depends on `fantasy-map-core` (no image or
  encoding libraries needed).

---

## Usage

```rust
use fantasy_map_core::map_data::MapAdapter;
use fantasy_map_renderer_svg::{SvgAdapter, SvgConfig};

let config = SvgConfig {
    coord_precision: 2,   // 2 decimal places for coordinates
    viewport_width:  1920,
    viewport_height: 1080,
};
let adapter = SvgAdapter::new(config);
let svg_string: String = adapter.render(&map_data);

std::fs::write("map.svg", svg_string)?;
```

Or via the `fmg` CLI:

```bash
fmg --seed 42 --format svg --output map.svg
```

---

## SVG layer structure

```xml
<svg xmlns="…" width="1920" height="1080" viewBox="0 0 1920 1080">
  <rect …/>                        <!-- background fill -->
  <g id="contour"   …><path …/></g>
  <g id="rivers"    …><path …/></g>
  <g id="slopes"    …><line …/>…</g>
  <g id="territory" …><path …/></g>
  <g id="cities"    …><circle …/>…</g>
  <g id="towns"     …><circle …/>…</g>
  <g id="labels"    …><text …/>…</g>
</svg>
```

---

## `SvgConfig`

| Field | Type | Default | Description |
|---|---|---|---|
| `coord_precision` | `usize` | `2` | Decimal places for x/y coordinates |
| `viewport_width` | `u32` | `1920` | SVG `width` / `viewBox` width |
| `viewport_height` | `u32` | `1080` | SVG `height` / `viewBox` height |

---

## Build commands

```bash
# Build
cargo build -p fantasy-map-renderer-svg

# Run tests (includes path-merging and precision tests)
cargo test -p fantasy-map-renderer-svg

# Build documentation
cargo doc -p fantasy-map-renderer-svg --open
```

---

## Adding this crate as a dependency

```toml
[dependencies]
fantasy-map-renderer-svg = { path = "../renderer-svg" }
```

---

## Design constraints

* **Allocation-friendly** — the SVG string is built into a single pre-allocated
  `String` (1 MB initial capacity) to avoid repeated re-allocations.
* **No WASM** — this crate is native-only; WASM rendering is handled by
  `crates/renderer-wasm`.
* **XML safety** — label text is XML-escaped before insertion into `<text>`
  elements.
