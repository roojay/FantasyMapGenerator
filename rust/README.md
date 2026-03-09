# Fantasy Map Generator — Rust Workspace

This directory contains the complete Rust implementation of the Fantasy Map
Generator, organised as a Cargo workspace.

---

## Project Structure

```
rust/
├── Cargo.toml                   # Workspace manifest
├── Cargo.lock
│
├── crates/
│   ├── core/                    # 🗺  Core generation library + legacy binary
│   ├── cli/                     # 🖥  Developer CLI tool (`fmg`)
│   ├── renderer-svg/            # 🖼  SVG rendering adapter
│   └── renderer-wasm/           # 🌐  WebAssembly bridge adapter
│
└── apps/
    └── web/                     # ⚛️  React + Three.js web frontend
```

---

## Crates at a Glance

| Crate | Package name | Binary | Purpose |
|---|---|---|---|
| `crates/core` | `fantasy-map-core` | `map_generation` | Core algorithms + types |
| `crates/cli` | `fantasy-map-cli` | `fmg` | Rich developer CLI |
| `crates/renderer-svg` | `fantasy-map-renderer-svg` | — | SVG output adapter |
| `crates/renderer-wasm` | `fantasy-map-renderer-wasm` | — | WASM/WebGPU bridge |

---

## Quick Start

### Prerequisites

* **Rust** ≥ 1.75 — [rustup.rs](https://rustup.rs)
* **Node.js** ≥ 20 — [nodejs.org](https://nodejs.org) (for the web frontend)
* **wasm-pack** (optional, for WASM builds) — `cargo install wasm-pack`

### Build all Rust crates

```bash
cd rust
cargo build --workspace
```

### Run all tests

```bash
cd rust
cargo test --workspace
```

### Generate a map (CLI)

```bash
# Print statistics
cargo run -p fantasy-map-cli -- --seed 42 --format stats

# Generate JSON
cargo run -p fantasy-map-cli -- --seed 42 --format json --output map.json

# Generate SVG
cargo run -p fantasy-map-cli -- --seed 42 --format svg --output map.svg

# Pipe JSON to another tool
cargo run -p fantasy-map-cli -- --seed 42 --format json | jq '.label | length'
```

### Start the web app

```bash
cd apps/web
npm install
npm run dev        # http://localhost:5173
```

### Build and integrate WASM (optional)

```bash
# From repo root
./scripts/build-wasm.sh
# Then start the web app — it will use live WASM generation instead of static data
cd apps/web && npm run dev
```

---

## Architecture

```
            ┌──────────────────────────────┐
            │       fantasy-map-core       │
            │  MapData  +  MapAdapter      │
            └──────────┬───────────────────┘
                       │ implements
          ┌────────────┼─────────────────────┐
          │            │                     │
   ┌──────▼──────┐  ┌──▼──────────┐  ┌──────▼───────┐
   │ renderer-   │  │ renderer-   │  │   fmg (cli)  │
   │    svg      │  │   wasm      │  │  JSON / SVG  │
   │  SvgAdapter │  │ WasmAdapter │  │    stats     │
   └─────────────┘  └──────┬──────┘  └──────────────┘
                           │ wasm-pack --target web
                    ┌──────▼──────────────┐
                    │     apps/web        │
                    │  React + Three.js   │
                    │  wasm-bridge.ts     │
                    └─────────────────────┘
```

The **plugin (adapter) pattern** is the central design principle:

1. `MapData` is the single output contract of the core algorithm — it is a
   plain serialisable Rust struct with normalised `[0,1]` coordinates.
2. `MapAdapter` is the trait every renderer implements.  Adding a new output
   format (PNG, GeoJSON, …) is as simple as adding a new crate that implements
   the trait.
3. Feature flags (`[features]`) in the WASM crate keep native and WASM builds
   completely separate, minimising the compiled `.wasm` binary size.

---

## Development Workflow

1. **Edit core algorithms** in `crates/core/src/map_generator.rs`.
2. **Run snapshot test** to verify output correctness:
   ```bash
   cargo test -p fantasy-map-core
   ```
3. **Iterate quickly** using the CLI:
   ```bash
   cargo run -p fantasy-map-cli -- --seed 1 --format stats
   ```
4. **Rebuild WASM** and test in the browser:
   ```bash
   ./scripts/build-wasm.sh --dev   # fast debug build
   cd apps/web && npm run dev
   ```
5. **Commit** following [Conventional Commits](https://www.conventionalcommits.org/).

---

## CI / CD

GitHub Actions workflows live in `.github/workflows/`:

| Workflow | Trigger | Steps |
|---|---|---|
| `rust.yml` | Push/PR touching `rust/**` | fmt · clippy · build · test |
| `web.yml` | Push/PR touching `rust/apps/web/**` | typecheck · build · deploy gh-pages |
