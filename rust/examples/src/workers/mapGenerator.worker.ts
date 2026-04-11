/**
 * Web Worker for map generation
 * Offloads heavy generation and packet extraction from the main thread.
 */

import { packetTransferables, scenePacketFromWasm } from "@/lib/mapScenePacket";
import type { GeneratedMapSource, MapScenePacket } from "@/types/map";

type WasmModule = typeof import("../../pkg/fantasy_map_generator.js");

interface InitMessage {
  type: "init";
}

interface GenerateMessage {
  type: "generate";
  requestId: number;
  seed: number;
  width: number;
  height: number;
  resolution: number;
  drawScale: number;
  cities: number;
  towns: number;
}

interface ExportJsonMessage extends GeneratedMapSource {
  type: "export-json";
  requestId: number;
}

interface ReadyResponse {
  type: "ready";
}

interface WasmMemoryLog {
  afterInit: number;
  beforeGenerate: number;
  afterGenerateRenderPacket: number;
  afterPacketExtraction: number;
  afterFree: number;
}

interface WorkerTimingLog {
  wasmGenerateMs: number;
  packetExtractMs: number;
  totalWorkerMs: number;
}

interface GenerateSuccessResponse {
  type: "generate-success";
  requestId: number;
  packet: MapScenePacket;
  seed: number;
  wasmMemoryLog?: WasmMemoryLog;
  workerTiming?: WorkerTimingLog;
}

interface ExportJsonSuccessResponse {
  type: "json-success";
  requestId: number;
  json: string;
}

interface ErrorResponse {
  type: "error";
  requestId: number;
  error?: string;
}

type WorkerMessage = InitMessage | GenerateMessage | ExportJsonMessage;
type WorkerResponse =
  | ReadyResponse
  | GenerateSuccessResponse
  | ExportJsonSuccessResponse
  | ErrorResponse;

let wasmModule: WasmModule | null = null;
let wasmMemory: WebAssembly.Memory | null = null;
let warmupComplete = false;
let memLog: WasmMemoryLog = { afterInit: 0, beforeGenerate: 0, afterGenerateRenderPacket: 0, afterPacketExtraction: 0, afterFree: 0 };
const workerScope = self as typeof self & {
  postMessage: (message: WorkerResponse, transfer?: Transferable[]) => void;
};

function getWasmMemoryMB(): string {
  if (!wasmMemory) return "N/A";
  return (wasmMemory.buffer.byteLength / (1024 * 1024)).toFixed(1) + " MB";
}

function getWasmMemoryBytes(): number {
  if (!wasmMemory) return 0;
  return wasmMemory.buffer.byteLength;
}

async function ensureWasmModule() {
  if (!wasmModule) {
    const pkg = await import("../../pkg/fantasy_map_generator.js");
    const exports = await pkg.default();
    // wasm-bindgen default() returns instance.exports which includes 'memory'
    wasmMemory = (exports as unknown as { memory: WebAssembly.Memory }).memory;
    wasmModule = pkg;
    memLog = { afterInit: 0, beforeGenerate: 0, afterGenerateRenderPacket: 0, afterPacketExtraction: 0, afterFree: 0 };
    memLog.afterInit = getWasmMemoryBytes();
    console.log(`[WASM-MEM] after init: ${getWasmMemoryMB()}`);
  }

  return wasmModule;
}

async function ensureWarmedUpModule() {
  const activeModule = await ensureWasmModule();

  if (!warmupComplete) {
    const generator = new activeModule.WasmMapGenerator(1, 256, 144, 0.16);
    generator.set_draw_scale(1);
    const packet = generator.generate_render_packet(0, 0);
    packet.free();
    generator.free();
    warmupComplete = true;
  }

  return activeModule;
}

self.onmessage = async (event: MessageEvent<WorkerMessage>) => {
  const { type } = event.data;

  if (type === "init") {
    await ensureWarmedUpModule();
    workerScope.postMessage({ type: "ready" });
    return;
  }

  const { requestId, seed, width, height, resolution, drawScale, cities, towns } = event.data;

  try {
    const activeModule = await ensureWarmedUpModule();

    const generator = new activeModule.WasmMapGenerator(seed, width, height, resolution);
    generator.set_draw_scale(drawScale);

    if (type === "export-json") {
      const json = generator.generate_with_options(cities, towns, true);
      generator.free();
      workerScope.postMessage({
        type: "json-success",
        requestId,
        json,
      });
      return;
    }

    console.log(`[WASM-MEM] before generate: ${getWasmMemoryMB()}`);
    memLog.beforeGenerate = getWasmMemoryBytes();
    const t0 = performance.now();
    const wasmPacket = generator.generate_render_packet(cities, towns);
    const t1 = performance.now();
    console.log(`[WASM-MEM] after generate_render_packet: ${getWasmMemoryMB()}`);
    memLog.afterGenerateRenderPacket = getWasmMemoryBytes();
    const actualSeed = generator.get_seed();

    const packet = scenePacketFromWasm({
      metadataJson: wasmPacket.metadata_json,
      svgJson: wasmPacket.svg_json,
      terrainPositions: wasmPacket.terrain_positions(),
      terrainNormals: wasmPacket.terrain_normals(),
      terrainUvs: wasmPacket.terrain_uvs(),
      terrainIndices: wasmPacket.terrain_indices(),
      heightTexture: wasmPacket.height_texture(),
      landMaskTexture: new Uint8Array(0),
      fluxTexture: new Uint8Array(0),
      terrainAlbedoTexture: wasmPacket.terrain_albedo_texture(),
      roughnessTexture: wasmPacket.roughness_texture(),
      aoTexture: wasmPacket.ao_texture(),
      waterColorTexture: wasmPacket.water_color_texture(),
      waterAlphaTexture: wasmPacket.water_alpha_texture(),
      coastGlowTexture: wasmPacket.coast_glow_texture(),
      slopeSegments: wasmPacket.slope_segments(),
      riverPositions: wasmPacket.river_positions(),
      riverOffsets: wasmPacket.river_offsets(),
      contourPositions: wasmPacket.contour_positions(),
      contourOffsets: wasmPacket.contour_offsets(),
      borderPositions: wasmPacket.border_positions(),
      borderOffsets: wasmPacket.border_offsets(),
      cityPositions: wasmPacket.city_positions(),
      townPositions: wasmPacket.town_positions(),
      labelBytes: wasmPacket.label_bytes(),
      labelOffsets: wasmPacket.label_offsets(),
      labelAnchors: wasmPacket.label_anchors(),
      labelSizes: wasmPacket.label_sizes(),
      landPolygonPositions: wasmPacket.land_polygon_positions(),
      landPolygonOffsets: wasmPacket.land_polygon_offsets(),
    }, {
      seed: actualSeed,
      width,
      height,
      resolution,
      drawScale,
      cities,
      towns,
    });

    console.log(`[WASM-MEM] after packet extraction: ${getWasmMemoryMB()}`);
    memLog.afterPacketExtraction = getWasmMemoryBytes();
    const t2 = performance.now();

    wasmPacket.free();
    generator.free();

    console.log(`[WASM-MEM] after free (packet+generator): ${getWasmMemoryMB()}`);
    memLog.afterFree = getWasmMemoryBytes();
    const t3 = performance.now();

    const workerTiming: WorkerTimingLog = {
      wasmGenerateMs: Math.round(t1 - t0),
      packetExtractMs: Math.round(t2 - t1),
      totalWorkerMs: Math.round(t3 - t0),
    };

    const response: GenerateSuccessResponse = {
      type: "generate-success",
      requestId,
      packet,
      seed: actualSeed,
      wasmMemoryLog: { ...memLog },
      workerTiming,
    };
    workerScope.postMessage(response, packetTransferables(packet));
  } catch (error) {
    const response: ErrorResponse = {
      type: "error",
      requestId,
      error: error instanceof Error ? error.message : String(error),
    };
    workerScope.postMessage(response);
  }
};
