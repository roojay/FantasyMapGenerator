# fantasy-map-cli

A developer-friendly command-line tool (`fmg`) for the Fantasy Map Generator.
Unlike the lower-level `map_generation` binary in `crates/core`, this tool lets
you choose the output format, inspect statistics, and is ideal for CI pipelines
and manual testing.

---

## Installation

```bash
# Build and install into ~/.cargo/bin
cargo install --path rust/crates/cli

# Or run directly from the workspace
cargo run -p fantasy-map-cli -- [OPTIONS]
```

---

## Usage

```
fmg [OPTIONS]

Options:
  -s, --seed <SEED>             Random seed [default: 0]
      --timeseed                Use current time as seed
  -o, --output <OUTPUT>         Output path ("-" for stdout) [default: -]
  -f, --format <FORMAT>         Output format [default: json]
                                  json | json-pretty | svg | stats
  -r, --resolution <RESOLUTION> Poisson-disc resolution [default: 0.08]
  -c, --cities <CITIES>         Number of cities (-1 = random) [default: -1]
  -t, --towns <TOWNS>           Number of towns (-1 = random) [default: -1]
  -e, --erosion-amount <AMT>    Erosion amount per step (-1 = random)
      --erosion-steps <N>       Erosion iterations [default: 3]
      --width <WIDTH>           Image width px [default: 1920]
      --height <HEIGHT>         Image height px [default: 1080]
      --draw-scale <SCALE>      Label scale factor [default: 1.0]
      --no-slopes               Disable slope rendering
      --no-rivers               Disable rivers
      --no-contour              Disable contour lines
      --no-borders              Disable territory borders
      --no-cities               Disable cities
      --no-towns                Disable towns
      --no-labels               Disable all labels
      --no-area-labels          Disable area labels only
      --svg-precision <N>       SVG coordinate decimals [default: 2]
  -q, --quiet                   Suppress progress messages
  -h, --help                    Print help
```

---

## Examples

```bash
# Quick statistics overview
fmg --seed 42 --format stats

# Compact JSON to stdout (pipe-friendly)
fmg --seed 42 --format json | jq '.label | length'

# Human-readable JSON to file
fmg --seed 42 --format json-pretty --output map.json

# SVG map to file
fmg --seed 42 --format svg --output map.svg

# Minimal map (no labels, no towns) for faster iteration
fmg --seed 1 --format stats --no-labels --no-towns

# Reproducible test with exact city/town counts
fmg --seed 7 --cities 3 --towns 8 --format stats

# Use time-based seed (unique every run)
fmg --timeseed --format json --output $(date +%s).json
```

---

## Output formats

### `json` (default)

Compact JSON blob; identical schema to the web app's `MapData` TypeScript
interface.  Suitable for machine consumption and diffs.

```json
{
  "image_width": 1920,
  "image_height": 1080,
  "draw_scale": 1.0,
  "contour": [[…], …],
  "river": [[…], …],
  "slope": […],
  "city": […],
  "town": […],
  "territory": [[…], …],
  "label": [{"text": "Ironhold", …}, …]
}
```

### `json-pretty`

Same as `json` but indented (4 spaces) for readability.

### `svg`

A production-quality Scalable Vector Graphics document.  Features:
* Layer-based `<g>` grouping (`contour`, `rivers`, `slopes`, `territory`,
  `cities`, `towns`, `labels`)
* Adjacent polylines of the same type merged into a single `<path>` (fewer DOM
  nodes, smaller file)
* Configurable coordinate precision via `--svg-precision`

### `stats`

Human-readable summary printed to stdout:

```
Fantasy Map Generator — Statistics
===================================
Seed             : 42
Image size       : 1920×1080 px
Draw scale       : 1.00

Contour lines    : 20 paths  (2779 pts)
Rivers           : 73 paths  (2042 pts)
Slope segments   : 6798
Territory borders: 10 paths  (1745 pts)
Cities           : 6
Towns            : 21
Labels           : 33
```

---

## Build commands

```bash
# Build only
cargo build -p fantasy-map-cli

# Build with optimisations
cargo build -p fantasy-map-cli --release

# Run tests (CLI is mostly tested via integration with core/svg)
cargo test -p fantasy-map-cli

# Build documentation
cargo doc -p fantasy-map-cli --open
```

---

## Design notes

* `--output -` (the default) writes to stdout, making the tool composable in
  shell pipelines.
* Progress messages are always written to **stderr** so they do not pollute
  stdout when piping JSON.
* The `--quiet` flag silences all stderr output for use in scripts.
* City and town names are loaded from the same bundled JSON as the core
  library (`CITY_DATA` constant).
