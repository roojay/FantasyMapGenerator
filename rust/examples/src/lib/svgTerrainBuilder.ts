import type { MapData, MapLayers } from "@/types/map";

type WasmModule = typeof import("../../pkg/fantasy_map_generator.js");

export interface SatelliteSvgOptions {
  maxEmbeddedImageSize?: number;
  jpegQuality?: number;
}

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

export async function generateSatelliteSVG(
  mapData: MapData,
  layers: MapLayers,
  options?: SatelliteSvgOptions
): Promise<string> {
  const wasm = await getWasmModule();
  if (options && Object.keys(options).length > 0) {
    return wasm.build_satellite_svg_with_options(
      JSON.stringify(mapData),
      JSON.stringify(layers),
      JSON.stringify({
        max_embedded_image_size: options.maxEmbeddedImageSize,
        jpeg_quality: options.jpegQuality
      })
    );
  }

  return wasm.build_satellite_svg(JSON.stringify(mapData), JSON.stringify(layers));
}
