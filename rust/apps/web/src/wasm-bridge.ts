/**
 * wasm-bridge.ts
 *
 * Runtime bridge between the React web app and the compiled WebAssembly module
 * produced by `wasm-pack` from `crates/renderer-wasm`.
 *
 * ## How it works
 *
 * 1. `scripts/build-wasm.sh` compiles `crates/renderer-wasm` with
 *    `--target web` and writes the output to `public/wasm/`.
 * 2. Vite serves the `public/` directory as static assets, so the files are
 *    reachable at `/wasm/…` in both dev and production.
 * 3. This module does a HEAD probe to check whether the WASM glue script
 *    exists, then dynamically imports it and initialises the WASM binary.
 * 4. If the WASM artefacts are absent (e.g. first clone, CI environment
 *    without Rust) the functions return `null` and the app falls back to
 *    loading the static `/map-data.json` placeholder.
 *
 * ## Building the WASM module
 *
 * ```bash
 * # From repo root
 * ./scripts/build-wasm.sh            # optimised release build
 * ./scripts/build-wasm.sh --dev      # fast debug build
 *
 * # Or via npm scripts inside apps/web
 * npm run wasm:build
 * npm run wasm:build:dev
 * ```
 *
 * ## Generated WASM API
 *
 * The `generate_map` export accepts plain numbers and returns a `WasmMapHandle`
 * whose `to_json()` method serialises the full `MapData` to a JSON string.
 * Call `handle.free()` when done to release WASM heap memory.
 */

import type { MapConfig, MapData } from './types/map';

// ── Type declarations for the wasm-pack generated module ──────────────────
// These mirror the `#[wasm_bindgen]` exports in crates/renderer-wasm/src/lib.rs.
// They are only used for static type-checking; the actual module is loaded
// dynamically at runtime.

interface WasmMapHandle {
  /** Serialise the full MapData as a compact JSON string. */
  to_json(): string;
  /** Number of f32 values in the vertex buffer. */
  vertex_count(): number;
  /** Number of u32 values in the index buffer. */
  index_count(): number;
  /** Raw pointer to the vertex buffer (for zero-copy Float32Array view). */
  vertex_ptr(): number;
  /** Raw pointer to the index buffer (for zero-copy Uint32Array view). */
  index_ptr(): number;
  /** Raw pointer to the colour buffer (for zero-copy Float32Array view). */
  color_ptr(): number;
  /** Image width stored in the map data. */
  image_width(): number;
  /** Image height stored in the map data. */
  image_height(): number;
  /** Release the WASM heap allocation. Always call when done. */
  free(): void;
}

interface WasmModule {
  /** Default export: init function – call once before using other exports. */
  default(wasmUrl?: string): Promise<void>;
  generate_map(
    seed: number,
    imgWidth: number,
    imgHeight: number,
    resolution: number,
    numCities: number,
    numTowns: number,
  ): WasmMapHandle;
  /** WebAssembly linear memory (for zero-copy typed-array views). */
  memory?: WebAssembly.Memory;
}

// ── Module-level cache ─────────────────────────────────────────────────────

let wasmModule: WasmModule | null = null;
/** Set to true after the first load attempt so we don't retry on every call. */
let loadAttempted = false;

// ── Public API ─────────────────────────────────────────────────────────────

/**
 * Try to initialise the WASM module.
 *
 * Returns the loaded module on success, or `null` if the WASM artefacts are
 * not present (the app will fall back to static JSON data in that case).
 */
export async function tryLoadWasm(): Promise<WasmModule | null> {
  if (loadAttempted) return wasmModule;
  loadAttempted = true;

  try {
    // Fast HEAD probe – avoids downloading the JS glue when WASM is absent
    const probeRes = await fetch('/wasm/fantasy_map_renderer_wasm.js', { method: 'HEAD' });
    if (!probeRes.ok) {
      console.info('[wasm-bridge] WASM module not found at /wasm/ – using static fallback');
      return null;
    }

    // Dynamic import of the wasm-bindgen ES-module glue.
    // `/* @vite-ignore */` suppresses Vite's static-analysis warning for the
    // absolute-URL import, which is intentional here.
    // `/* @ts-ignore */` suppresses the TS2307 error for the runtime-only path.
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore – WASM artefact is a runtime dependency, not a compile-time one
    const mod = await import(/* @vite-ignore */ '/wasm/fantasy_map_renderer_wasm.js') as WasmModule;

    // Initialise the WASM binary (explicit path avoids relative-URL issues)
    await mod.default('/wasm/fantasy_map_renderer_wasm_bg.wasm');

    wasmModule = mod;
    console.info('[wasm-bridge] WASM module loaded successfully');
    return wasmModule;
  } catch (err) {
    console.warn('[wasm-bridge] Failed to load WASM module, falling back to static data:', err);
    wasmModule = null;
    return null;
  }
}

/**
 * Generate a map using the WASM module.
 *
 * @returns Parsed `MapData` on success, `null` if WASM is unavailable.
 */
export async function generateMapWasm(config: MapConfig): Promise<MapData | null> {
  const mod = await tryLoadWasm();
  if (!mod) return null;

  const handle = mod.generate_map(
    config.seed,
    config.width,
    config.height,
    config.resolution,
    config.cities,
    config.towns,
  );

  try {
    const json = handle.to_json();
    return JSON.parse(json) as MapData;
  } finally {
    handle.free(); // always release WASM heap memory
  }
}

/**
 * Check whether the WASM module has been successfully loaded.
 *
 * Useful for showing a UI indicator that tells the user whether live
 * generation or static demo data is being used.
 */
export function isWasmAvailable(): boolean {
  return wasmModule !== null;
}

/**
 * Get a zero-copy `Float32Array` view of the vertex buffer inside the WASM
 * linear memory.  Only valid while the `handle` is alive; do NOT keep a
 * reference after calling `handle.free()`.
 *
 * Returns `null` when the WASM module is not loaded.
 */
export function getVertexBufferView(
  handle: WasmMapHandle,
  mod: WasmModule,
): Float32Array | null {
  if (!mod.memory) return null;
  const ptr = handle.vertex_ptr();
  const len = handle.vertex_count();
  return new Float32Array(mod.memory.buffer, ptr, len);
}

/**
 * Get a zero-copy `Uint32Array` view of the index buffer.
 */
export function getIndexBufferView(
  handle: WasmMapHandle,
  mod: WasmModule,
): Uint32Array | null {
  if (!mod.memory) return null;
  const ptr = handle.index_ptr();
  const len = handle.index_count();
  return new Uint32Array(mod.memory.buffer, ptr, len);
}
