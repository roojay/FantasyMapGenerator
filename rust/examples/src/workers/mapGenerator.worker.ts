/**
 * Web Worker for map generation
 * Offloads heavy generation and packet extraction from the main thread.
 */

import { packetTransferables, scenePacketFromWasm } from "@/lib/mapScenePacket";
import type { MapScenePacket } from "@/types/map";

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
}

interface GenerateResponse {
  type: "success" | "error";
  packet?: MapScenePacket;
  seed?: number;
  error?: string;
}

let wasmModule: WasmModule | null = null;
const workerScope = self as typeof self & {
  postMessage: (message: unknown, transfer?: Transferable[]) => void;
};

self.onmessage = async (event: MessageEvent<GenerateMessage>) => {
  const { type, seed, width, height, resolution, drawScale, cities, towns } = event.data;

  if (type !== "generate") return;

  try {
    if (!wasmModule) {
      const pkg = await import("../../pkg/fantasy_map_generator.js");
      await pkg.default();
      wasmModule = pkg;
    }

    const generator = new wasmModule.WasmMapGenerator(seed, width, height, resolution);
    generator.set_draw_scale(drawScale);

    const wasmPacket = generator.generate_render_packet(cities, towns);
    const actualSeed = generator.get_seed();

    const packet = scenePacketFromWasm({
      metadataJson: wasmPacket.metadata_json,
      legacyJson: wasmPacket.legacy_json,
      terrainPositions: wasmPacket.terrain_positions(),
      terrainNormals: wasmPacket.terrain_normals(),
      terrainUvs: wasmPacket.terrain_uvs(),
      terrainIndices: wasmPacket.terrain_indices(),
      heightTexture: wasmPacket.height_texture(),
      landMaskTexture: wasmPacket.land_mask_texture(),
      fluxTexture: wasmPacket.flux_texture(),
      albedoTexture: wasmPacket.albedo_texture(),
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
      landPolygonOffsets: wasmPacket.land_polygon_offsets()
    });

    wasmPacket.free();
    generator.free();

    const response: GenerateResponse = {
      type: "success",
      packet,
      seed: actualSeed
    };
    workerScope.postMessage(response, packetTransferables(packet));
  } catch (error) {
    const response: GenerateResponse = {
      type: "error",
      error: error instanceof Error ? error.message : String(error)
    };
    workerScope.postMessage(response);
  }
};
