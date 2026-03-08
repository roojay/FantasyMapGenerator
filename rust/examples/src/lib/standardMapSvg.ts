import type { MapLayers } from "@/types/map";

type WasmModule = typeof import("../../pkg/fantasy_map_generator.js");

let wasmModulePromise: Promise<WasmModule> | null = null;

async function getWasmModule(): Promise<WasmModule> {
  if (!wasmModulePromise) {
    wasmModulePromise = import("../../pkg/fantasy_map_generator.js").then(async (pkg) => {
      await pkg.default();
      return pkg;
    });
  }

  return wasmModulePromise;
}

export async function buildStandardMapSvg(mapJson: string, layers: MapLayers): Promise<string> {
  const wasm = await getWasmModule();
  return wasm.build_map_svg(mapJson, JSON.stringify(layers));
}
