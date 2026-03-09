# Fantasy Map Generator — Web App

A modern, responsive web interface for the Fantasy Map Generator built with
React 19, Three.js, Mantine and Tailwind CSS.

---

## Features

* **2D SVG renderer** — instant rendering with per-layer visibility toggles
* **3D Three.js viewer** — perspective view with `OrbitControls`
* **Live WASM generation** — generates new maps in the browser via WebAssembly
  (requires building the WASM module first; falls back to static demo data)
* **Config panel** — adjustable seed, resolution, city/town counts and erosion
* **Layer toolbar** — real-time toggle for contour, rivers, slopes, territory,
  cities, towns and labels
* **i18n** — English / 中文 switching
* **Dark / light mode** — synced across Mantine and Tailwind
* **Responsive design** — sidebar on desktop, drawer on mobile

---

## Quick Start

### Prerequisites

* **Node.js** ≥ 20

### Install and run

```bash
cd rust/apps/web
npm install
npm run dev        # http://localhost:5173
```

### Production build

```bash
npm run build      # outputs to dist/
npm run preview    # preview the production build locally
```

---

## Enabling Live WASM Generation

By default the app loads a static demo map (`public/map-data.json`).  To
enable live in-browser generation:

1. Make sure you have Rust and `wasm-pack` installed:
   ```bash
   cargo install wasm-pack
   ```

2. Build the WASM module (from the **repo root**):
   ```bash
   ./scripts/build-wasm.sh        # optimised release build
   # OR
   ./scripts/build-wasm.sh --dev  # fast debug build
   ```
   This writes the artefacts to `public/wasm/`.

3. Start (or restart) the dev server:
   ```bash
   npm run dev
   ```

The header badge now shows **WASM** (green) instead of **Static** (orange).
Click "Generate Map" with any seed to generate a unique map live in the browser.

---

## npm scripts

| Script | Description |
|---|---|
| `npm run dev` | Start Vite dev server |
| `npm run build` | TypeScript check + production build |
| `npm run preview` | Preview production build |
| `npm run typecheck` | Type-check only (no emit) |
| `npm run wasm:build` | Build WASM (release) and copy to `public/wasm/` |
| `npm run wasm:build:dev` | Build WASM (debug) |

---

## Project Structure

```
apps/web/
├── index.html
├── package.json
├── vite.config.ts
├── tailwind.config.js
├── tsconfig.json
│
├── public/
│   ├── map-data.json           # Static demo data (always present)
│   └── wasm/                   # WASM build output (gitignored, created by wasm:build)
│       ├── *.js
│       ├── *.wasm
│       └── *.d.ts
│
└── src/
    ├── main.tsx                # App entry point
    ├── App.tsx                 # Root component
    ├── index.css               # Tailwind base + global styles
    ├── wasm-bridge.ts          # WASM loader + fallback logic
    │
    ├── types/
    │   └── map.ts              # MapData, MapConfig, LayerVisibility types
    │
    ├── i18n/
    │   ├── en.ts               # English translations
    │   ├── zh.ts               # Chinese translations
    │   └── index.ts            # getT() helper
    │
    ├── store/
    │   └── mapStore.ts         # useMapStore hook (all app state)
    │
    ├── hooks/
    │   └── useMapData.ts       # Map generation hook (WASM + JSON fallback)
    │
    └── components/
        ├── Header.tsx          # Stats bar + language/theme toggles
        ├── ConfigPanel.tsx     # Map config form (sidebar / drawer)
        ├── LayerToolbar.tsx    # Layer visibility switches (bottom bar)
        ├── MapViewer.tsx       # 2D SVG renderer
        └── MapViewer3D.tsx     # Three.js 3D viewer
```

---

## Technology Stack

| Concern | Library / Version |
|---|---|
| Framework | React 19 |
| Build tool | Vite 6 |
| Language | TypeScript 5 (strict mode) |
| UI components | Mantine v7 |
| CSS | Tailwind CSS v3 |
| 3D rendering | Three.js r170 (`WebGLRenderer` + `OrbitControls`) |
| WASM bridge | wasm-bindgen (via `fantasy-map-renderer-wasm`) |

---

## WASM integration details

`src/wasm-bridge.ts` implements the following strategy:

1. On first load, perform a **HEAD probe** to `/wasm/fantasy_map_renderer_wasm.js`.
2. If present, **dynamically import** the wasm-pack ES-module glue and call
   `init('/wasm/fantasy_map_renderer_wasm_bg.wasm')`.
3. Expose `generateMapWasm(config)` which calls `generate_map(…)` on the WASM
   module and returns a parsed `MapData`.
4. If any step fails, return `null` and let `useMapData` fall back to loading
   `public/map-data.json`.

The result is a seamless developer experience: the app always works (with demo
data), but gains live generation capability once the WASM build is run.

---

## Environment

No `.env` files are required.  All configuration is done through the UI config
panel and the `public/map-data.json` fallback file.

---

## Deployment

The GitHub Actions workflow `web.yml` automatically builds and deploys the app
to the `gh-pages` branch on every merge to `main`.

To deploy manually:

```bash
npm run build
# Upload contents of dist/ to your hosting provider
```
