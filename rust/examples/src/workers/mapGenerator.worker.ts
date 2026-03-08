/**
 * Web Worker for map generation
 * Offloads heavy computation from main thread
 */

type WasmModule = typeof import("../../pkg/fantasy_map_generator.js");

interface GenerateMessage {
  type: "generate";
  seed: number;
  width: number;
  height: number;
  resolution: number;
  drawScale: number;
  cities: number;
  towns: number;
  includeRasterData: boolean;
}

interface GenerateResponse {
  type: "success" | "error";
  json?: string;
  seed?: number;
  error?: string;
}

let wasmModule: WasmModule | null = null;

self.onmessage = async (e: MessageEvent<GenerateMessage>) => {
  const { type, seed, width, height, resolution, drawScale, cities, towns, includeRasterData } = e.data;

  if (type !== "generate") return;

  try {
    // 初始化 WASM（如果尚未初始化）
    if (!wasmModule) {
      const pkg = await import("../../pkg/fantasy_map_generator.js");
      await pkg.default();
      wasmModule = pkg;
    }

    // 生成地图
    const generator = new wasmModule.WasmMapGenerator(seed, width, height, resolution);
    generator.set_draw_scale(drawScale);

    // keep raster export opt-in so ordinary generation avoids copying
    // large satellite-only arrays back to the main thread.
    const json = generator.generate_with_options(cities, towns, includeRasterData);
    const actualSeed = generator.get_seed();
    generator.free();

    // 返回结果
    const response: GenerateResponse = {
      type: "success",
      json,
      seed: actualSeed
    };
    self.postMessage(response);
  } catch (error) {
    const response: GenerateResponse = {
      type: "error",
      error: error instanceof Error ? error.message : String(error)
    };
    self.postMessage(response);
  }
};
