import type {
  GeneratedMapSource,
  MapExportData,
  MapLabelRenderItem,
  MapSceneMetadata,
  MapScenePacket,
  PathLayerPacket,
} from "@/types/map";

const _encoder = new TextEncoder();
const TERRAIN_ELEVATION_SCALE = 64;
const WATER_DEPTH_SCALE = 10;
const OVERLAY_HEIGHT_OFFSET = 1.2;
const LABEL_HEIGHT_OFFSET = 2.4;
const textEncoder = new TextEncoder();
const textDecoder = new TextDecoder();

interface WasmPacketWire {
  metadataJson: string;
  svgJson: string;
  terrainPositions: Float32Array;
  terrainNormals: Float32Array;
  terrainUvs: Float32Array;
  terrainIndices: Uint32Array;
  heightTexture: Uint8Array;
  landMaskTexture: Uint8Array;
  fluxTexture: Uint8Array;
  terrainAlbedoTexture: Uint8Array;
  roughnessTexture: Uint8Array;
  aoTexture: Uint8Array;
  waterColorTexture: Uint8Array;
  waterAlphaTexture: Uint8Array;
  coastGlowTexture: Uint8Array;
  slopeSegments: Float32Array;
  riverPositions: Float32Array;
  riverOffsets: Uint32Array;
  contourPositions: Float32Array;
  contourOffsets: Uint32Array;
  borderPositions: Float32Array;
  borderOffsets: Uint32Array;
  cityPositions: Float32Array;
  townPositions: Float32Array;
  labelBytes: Uint8Array;
  labelOffsets: Uint32Array;
  labelAnchors: Float32Array;
  labelSizes: Float32Array;
  landPolygonPositions: Float32Array;
  landPolygonOffsets: Uint32Array;
}

function toTopDownFloats(data: number[], width: number, height: number) {
  const output = new Float32Array(width * height);
  for (let y = 0; y < height; y += 1) {
    const srcY = height - 1 - y;
    for (let x = 0; x < width; x += 1) {
      output[y * width + x] = data[srcY * width + x] ?? 0;
    }
  }
  return output;
}

function toTopDownMask(data: number[], width: number, height: number) {
  const output = new Uint8Array(width * height);
  for (let y = 0; y < height; y += 1) {
    const srcY = height - 1 - y;
    for (let x = 0; x < width; x += 1) {
      output[y * width + x] = (data[srcY * width + x] ?? 0) > 0 ? 1 : 0;
    }
  }
  return output;
}

function sampleGrid(data: Float32Array, width: number, height: number, x: number, y: number) {
  const clampedX = Math.max(0, Math.min(width - 1, x));
  const clampedY = Math.max(0, Math.min(height - 1, y));
  return data[clampedY * width + clampedX] ?? 0;
}

function sampleElevation(
  elevations: Float32Array,
  width: number,
  height: number,
  normalizedX: number,
  normalizedTopY: number,
) {
  const sampleX = Math.max(0, Math.min(width - 1, normalizedX * (width - 1)));
  const sampleY = Math.max(0, Math.min(height - 1, normalizedTopY * (height - 1)));
  const x0 = Math.floor(sampleX);
  const y0 = Math.floor(sampleY);
  const x1 = Math.min(width - 1, x0 + 1);
  const y1 = Math.min(height - 1, y0 + 1);
  const tx = sampleX - x0;
  const ty = sampleY - y0;
  const h00 = sampleGrid(elevations, width, height, x0, y0);
  const h10 = sampleGrid(elevations, width, height, x1, y0);
  const h01 = sampleGrid(elevations, width, height, x0, y1);
  const h11 = sampleGrid(elevations, width, height, x1, y1);
  const hx0 = h00 + (h10 - h00) * tx;
  const hx1 = h01 + (h11 - h01) * tx;
  return hx0 + (hx1 - hx0) * ty;
}

function terrainElevation(height: number, isLand: boolean) {
  if (isLand) {
    return (
      (Math.pow(Math.max(0, Math.min(1, height)), 1.12) * 0.9 + 0.04) * TERRAIN_ELEVATION_SCALE
    );
  }

  return -WATER_DEPTH_SCALE + Math.max(0, Math.min(1, height)) * (WATER_DEPTH_SCALE * 0.35);
}

function lerpColor(
  a: [number, number, number],
  b: [number, number, number],
  t: number,
): [number, number, number] {
  const clamped = Math.max(0, Math.min(1, t));
  return [
    Math.round(a[0] + (b[0] - a[0]) * clamped),
    Math.round(a[1] + (b[1] - a[1]) * clamped),
    Math.round(a[2] + (b[2] - a[2]) * clamped),
  ];
}

function sampleMask(mask: Uint8Array, width: number, height: number, x: number, y: number) {
  const clampedX = Math.max(0, Math.min(width - 1, x));
  const clampedY = Math.max(0, Math.min(height - 1, y));
  return (mask[clampedY * width + clampedX] ?? 0) > 0 ? 1 : 0;
}

function coastalStrength(mask: Uint8Array, width: number, height: number, x: number, y: number) {
  const center = sampleMask(mask, width, height, x, y);
  let delta = 0;
  for (let oy = -1; oy <= 1; oy += 1) {
    for (let ox = -1; ox <= 1; ox += 1) {
      if (ox === 0 && oy === 0) continue;
      delta += Math.abs(center - sampleMask(mask, width, height, x + ox, y + oy));
    }
  }
  return Math.max(0, Math.min(1, delta / 8));
}

function colorizeLand(height: number, flux: number, coast: number): [number, number, number] {
  const base =
    height < 0.08
      ? lerpColor([207, 192, 145], [181, 167, 113], height / 0.08)
      : height < 0.22
        ? lerpColor([182, 176, 131], [119, 144, 84], (height - 0.08) / 0.14)
        : height < 0.48
          ? lerpColor([97, 127, 76], [86, 103, 70], (height - 0.22) / 0.26)
          : height < 0.72
            ? lerpColor([112, 110, 90], [142, 138, 118], (height - 0.48) / 0.24)
            : lerpColor([186, 186, 180], [244, 243, 238], (height - 0.72) / 0.28);

  const riverTint = [84, 158, 204] as [number, number, number];
  const coastTint = [224, 214, 176] as [number, number, number];
  return lerpColor(
    lerpColor(base, riverTint, Math.max(0, Math.min(0.8, flux * 1.6))),
    coastTint,
    Math.max(0, Math.min(0.35, coast * 0.35)),
  );
}

function colorizeWater(height: number, coast: number): [number, number, number] {
  const depth = Math.max(0, Math.min(1, 1 - height));
  const base = lerpColor([34, 82, 126], [10, 34, 74], depth * 0.9);
  return lerpColor(base, [76, 155, 187], Math.max(0, Math.min(0.75, coast * 0.75)));
}

function buildPathLayer(
  paths: number[][],
  metadata: MapSceneMetadata,
  elevations: Float32Array,
  yOffset: number,
): PathLayerPacket {
  const positions: number[] = [];
  const offsets = new Uint32Array(paths.length + 1);

  paths.forEach((path, pathIndex) => {
    for (let index = 0; index < path.length; index += 2) {
      const x = path[index] ?? 0;
      const y = path[index + 1] ?? 0;
      positions.push(
        (x - 0.5) * metadata.imageWidth,
        sampleElevation(elevations, metadata.terrainWidth, metadata.terrainHeight, x, 1 - y) +
          yOffset,
        (y - 0.5) * metadata.imageHeight,
      );
    }
    offsets[pathIndex + 1] = positions.length / 3;
  });

  return {
    positions: new Float32Array(positions),
    offsets,
  };
}

function buildPointPositions(
  points: number[],
  metadata: MapSceneMetadata,
  elevations: Float32Array,
  yOffset: number,
) {
  const positions = new Float32Array((points.length / 2) * 3);
  for (let index = 0; index < points.length; index += 2) {
    const x = points[index] ?? 0;
    const y = points[index + 1] ?? 0;
    const targetIndex = (index / 2) * 3;
    positions[targetIndex] = (x - 0.5) * metadata.imageWidth;
    positions[targetIndex + 1] =
      sampleElevation(elevations, metadata.terrainWidth, metadata.terrainHeight, x, 1 - y) +
      yOffset;
    positions[targetIndex + 2] = (y - 0.5) * metadata.imageHeight;
  }
  return positions;
}

function buildLabelBuffers(
  data: MapExportData,
  metadata: MapSceneMetadata,
  elevations: Float32Array,
) {
  const bytes: number[] = [];
  const offsets = new Uint32Array(data.label.length + 1);
  const anchors = new Float32Array(data.label.length * 3);
  const sizes = new Float32Array(data.label.length);
  const items: MapLabelRenderItem[] = new Array(data.label.length);

  data.label.forEach((label, index) => {
    const encoded = textEncoder.encode(label.text);
    bytes.push(...encoded);
    offsets[index + 1] = bytes.length;
    anchors[index * 3] = (label.position[0] - 0.5) * metadata.imageWidth;
    anchors[index * 3 + 1] =
      sampleElevation(
        elevations,
        metadata.terrainWidth,
        metadata.terrainHeight,
        label.position[0],
        1 - label.position[1],
      ) + LABEL_HEIGHT_OFFSET;
    anchors[index * 3 + 2] = (label.position[1] - 0.5) * metadata.imageHeight;
    sizes[index] = label.fontsize;
    items[index] = {
      fontface: label.fontface,
      fontsize: label.fontsize,
      text: label.text,
    };
  });

  return {
    bytes: new Uint8Array(bytes),
    offsets,
    anchors,
    sizes,
    items,
  };
}

function decodeLabelItems(
  bytes: Uint8Array,
  offsets: Uint32Array,
  sizes: Float32Array,
  mapData: Pick<MapExportData, "label">,
) {
  if (Array.isArray(mapData.label)) {
    return mapData.label.map<MapLabelRenderItem>((label) => ({
      fontface: label.fontface,
      fontsize: label.fontsize,
      text: label.text,
    }));
  }

  const items: MapLabelRenderItem[] = [];
  for (let index = 0; index < offsets.length - 1; index += 1) {
    items.push({
      fontface: "Times New Roman",
      fontsize: sizes[index] ?? 12,
      text: textDecoder.decode(bytes.subarray(offsets[index], offsets[index + 1])),
    });
  }
  return items;
}

function buildLandPolygonBuffers(polygons?: number[][] | null) {
  const validPolygons = (polygons ?? []).filter(
    (polygon): polygon is number[] =>
      Array.isArray(polygon) && polygon.length >= 6 && polygon.length % 2 === 0,
  );
  const totalValues = validPolygons.reduce((sum, polygon) => sum + polygon.length, 0);
  const positions = new Float32Array(totalValues);
  const offsets = new Uint32Array(validPolygons.length + 1);

  let cursor = 0;
  validPolygons.forEach((polygon, index) => {
    positions.set(polygon, cursor);
    cursor += polygon.length;
    offsets[index + 1] = cursor;
  });

  return { positions, offsets };
}

function createSvgMapExportData(data: MapExportData): MapExportData {
  const {
    heightmap: _heightmap,
    flux_map: _fluxMap,
    land_mask: _landMask,
    land_polygons: _landPolygons,
    ...svgData
  } = data;

  return svgData;
}

function buildExportTerrain(data: MapExportData) {
  const rasterWidth = data.heightmap?.width ?? 2;
  const rasterHeight = data.heightmap?.height ?? 2;
  const heightTop = data.heightmap
    ? toTopDownFloats(data.heightmap.data, rasterWidth, rasterHeight)
    : new Float32Array(rasterWidth * rasterHeight);
  const landTop = data.land_mask
    ? toTopDownMask(data.land_mask.data, rasterWidth, rasterHeight)
    : new Uint8Array(rasterWidth * rasterHeight).fill(1);
  const fluxTop = data.flux_map
    ? toTopDownFloats(data.flux_map.data, rasterWidth, rasterHeight)
    : new Float32Array(rasterWidth * rasterHeight);

  const elevations = new Float32Array(rasterWidth * rasterHeight);
  for (let index = 0; index < elevations.length; index += 1) {
    elevations[index] = terrainElevation(heightTop[index] ?? 0, (landTop[index] ?? 0) > 0);
  }

  const positions = new Float32Array(rasterWidth * rasterHeight * 3);
  const normals = new Float32Array(rasterWidth * rasterHeight * 3);
  const uvs = new Float32Array(rasterWidth * rasterHeight * 2);
  const indices = new Uint32Array(Math.max(0, (rasterWidth - 1) * (rasterHeight - 1) * 6));

  let vertexCursor = 0;
  let uvCursor = 0;
  for (let y = 0; y < rasterHeight; y += 1) {
    const vTop = rasterHeight > 1 ? y / (rasterHeight - 1) : 0;
    const worldZ = (0.5 - vTop) * data.image_height;
    for (let x = 0; x < rasterWidth; x += 1) {
      const u = rasterWidth > 1 ? x / (rasterWidth - 1) : 0;
      const worldX = (u - 0.5) * data.image_width;
      const elevation = elevations[y * rasterWidth + x] ?? 0;
      positions[vertexCursor] = worldX;
      positions[vertexCursor + 1] = elevation;
      positions[vertexCursor + 2] = worldZ;

      const left = sampleGrid(elevations, rasterWidth, rasterHeight, x - 1, y);
      const right = sampleGrid(elevations, rasterWidth, rasterHeight, x + 1, y);
      const up = sampleGrid(elevations, rasterWidth, rasterHeight, x, y - 1);
      const down = sampleGrid(elevations, rasterWidth, rasterHeight, x, y + 1);
      const nx = left - right;
      const nz = down - up;
      const length = Math.hypot(nx, 2, nz) || 1;
      normals[vertexCursor] = nx / length;
      normals[vertexCursor + 1] = 2 / length;
      normals[vertexCursor + 2] = nz / length;
      vertexCursor += 3;

      uvs[uvCursor] = u;
      uvs[uvCursor + 1] = 1 - vTop;
      uvCursor += 2;
    }
  }

  let indexCursor = 0;
  for (let y = 0; y < rasterHeight - 1; y += 1) {
    for (let x = 0; x < rasterWidth - 1; x += 1) {
      const i0 = y * rasterWidth + x;
      const i1 = i0 + 1;
      const i2 = i0 + rasterWidth;
      const i3 = i2 + 1;
      indices[indexCursor] = i0;
      indices[indexCursor + 1] = i2;
      indices[indexCursor + 2] = i1;
      indices[indexCursor + 3] = i1;
      indices[indexCursor + 4] = i2;
      indices[indexCursor + 5] = i3;
      indexCursor += 6;
    }
  }

  const heightTexture = new Uint8Array(rasterWidth * rasterHeight * 4);
  const landMaskTexture = new Uint8Array(rasterWidth * rasterHeight * 4);
  const fluxTexture = new Uint8Array(rasterWidth * rasterHeight * 4);
  const albedoTexture = new Uint8Array(rasterWidth * rasterHeight * 4);

  for (let y = 0; y < rasterHeight; y += 1) {
    for (let x = 0; x < rasterWidth; x += 1) {
      const pixelIndex = y * rasterWidth + x;
      const targetIndex = pixelIndex * 4;
      const heightValue = Math.max(0, Math.min(1, heightTop[pixelIndex] ?? 0));
      const fluxValue = Math.max(0, Math.min(1, fluxTop[pixelIndex] ?? 0));
      const isLand = (landTop[pixelIndex] ?? 0) > 0;
      const coast = coastalStrength(landTop, rasterWidth, rasterHeight, x, y);
      const encodedHeight = Math.round(heightValue * 255);
      const encodedFlux = Math.round(fluxValue * 255);
      const [r, g, b] = isLand
        ? colorizeLand(heightValue, fluxValue, coast)
        : colorizeWater(heightValue, coast);

      heightTexture[targetIndex] = encodedHeight;
      heightTexture[targetIndex + 1] = encodedHeight;
      heightTexture[targetIndex + 2] = encodedHeight;
      heightTexture[targetIndex + 3] = 255;

      const landEncoded = isLand ? 255 : 0;
      landMaskTexture[targetIndex] = landEncoded;
      landMaskTexture[targetIndex + 1] = landEncoded;
      landMaskTexture[targetIndex + 2] = landEncoded;
      landMaskTexture[targetIndex + 3] = 255;

      fluxTexture[targetIndex] = encodedFlux;
      fluxTexture[targetIndex + 1] = encodedFlux;
      fluxTexture[targetIndex + 2] = encodedFlux;
      fluxTexture[targetIndex + 3] = 255;

      albedoTexture[targetIndex] = r;
      albedoTexture[targetIndex + 1] = g;
      albedoTexture[targetIndex + 2] = b;
      albedoTexture[targetIndex + 3] = 255;
    }
  }

  return {
    terrainWidth: rasterWidth,
    terrainHeight: rasterHeight,
    elevations,
    terrain: {
      positions,
      normals,
      uvs,
      indices,
    },
    textures: {
      height: heightTexture,
      landMask: landMaskTexture,
      flux: fluxTexture,
      albedo: albedoTexture,
    },
  };
}

export function packetTransferables(packet: MapScenePacket): Transferable[] {
  const buffers = [
    packet.terrain.positions.buffer,
    packet.terrain.normals.buffer,
    packet.terrain.uvs.buffer,
    packet.terrain.indices.buffer,
    packet.textures.height.buffer,
    packet.textures.landMask.buffer,
    packet.textures.flux.buffer,
    packet.layers.slopeSegments.buffer,
    packet.layers.river.positions.buffer,
    packet.layers.river.offsets.buffer,
    packet.layers.contour.positions.buffer,
    packet.layers.contour.offsets.buffer,
    packet.layers.border.positions.buffer,
    packet.layers.border.offsets.buffer,
    packet.markers.city.buffer,
    packet.markers.town.buffer,
    packet.labels.bytes.buffer,
    packet.labels.offsets.buffer,
    packet.labels.anchors.buffer,
    packet.labels.sizes.buffer,
    packet.landPolygonPositions.buffer,
    packet.landPolygonOffsets.buffer,
  ];
  const transferables: Transferable[] = buffers.filter(
    (buffer): buffer is ArrayBuffer => buffer instanceof ArrayBuffer && buffer.byteLength > 0,
  );

  if (packet.textures.albedo && packet.textures.albedo.byteLength > 0) {
    transferables.push(packet.textures.albedo.buffer);
  }
  if (packet.textures.terrainAlbedo && packet.textures.terrainAlbedo.byteLength > 0) {
    transferables.push(packet.textures.terrainAlbedo.buffer);
  }
  if (packet.textures.roughness && packet.textures.roughness.byteLength > 0) {
    transferables.push(packet.textures.roughness.buffer);
  }
  if (packet.textures.ao && packet.textures.ao.byteLength > 0) {
    transferables.push(packet.textures.ao.buffer);
  }
  if (packet.textures.waterColor && packet.textures.waterColor.byteLength > 0) {
    transferables.push(packet.textures.waterColor.buffer);
  }
  if (packet.textures.waterAlpha && packet.textures.waterAlpha.byteLength > 0) {
    transferables.push(packet.textures.waterAlpha.buffer);
  }
  if (packet.textures.coastGlow && packet.textures.coastGlow.byteLength > 0) {
    transferables.push(packet.textures.coastGlow.buffer);
  }

  if (packet.svgMapJsonBytes && packet.svgMapJsonBytes.byteLength > 0) {
    transferables.push(packet.svgMapJsonBytes.buffer);
  }

  return transferables;
}

export function scenePacketFromWasm(
  wire: WasmPacketWire,
  generatedFrom?: GeneratedMapSource,
): MapScenePacket {
  const svgData = JSON.parse(wire.svgJson) as MapExportData;
  const rawMetadata = JSON.parse(wire.metadataJson) as {
    image_width: number;
    image_height: number;
    draw_scale: number;
    terrain_width: number;
    terrain_height: number;
    texture_width: number;
    texture_height: number;
    elevation_scale: number;
    city_count: number;
    town_count: number;
    river_count: number;
    territory_count: number;
    label_count: number;
  };

  const metadata: MapSceneMetadata = {
    imageWidth: rawMetadata.image_width,
    imageHeight: rawMetadata.image_height,
    drawScale: rawMetadata.draw_scale,
    terrainWidth: rawMetadata.terrain_width,
    terrainHeight: rawMetadata.terrain_height,
    textureWidth: rawMetadata.texture_width ?? rawMetadata.terrain_width,
    textureHeight: rawMetadata.texture_height ?? rawMetadata.terrain_height,
    elevationScale: rawMetadata.elevation_scale,
    cityCount: rawMetadata.city_count,
    townCount: rawMetadata.town_count,
    riverCount: rawMetadata.river_count,
    territoryCount: rawMetadata.territory_count,
    labelCount: rawMetadata.label_count,
  };

  return {
    metadata,
    terrain: {
      positions: wire.terrainPositions,
      normals: wire.terrainNormals,
      uvs: wire.terrainUvs,
      indices: wire.terrainIndices,
    },
    textures: {
      height: wire.heightTexture,
      landMask: wire.landMaskTexture,
      flux: wire.fluxTexture,
      terrainAlbedo: wire.terrainAlbedoTexture,
      roughness: wire.roughnessTexture,
      ao: wire.aoTexture,
      waterColor: wire.waterColorTexture,
      waterAlpha: wire.waterAlphaTexture,
      coastGlow: wire.coastGlowTexture,
    },
    layers: {
      slopeSegments: wire.slopeSegments,
      river: {
        positions: wire.riverPositions,
        offsets: wire.riverOffsets,
      },
      contour: {
        positions: wire.contourPositions,
        offsets: wire.contourOffsets,
      },
      border: {
        positions: wire.borderPositions,
        offsets: wire.borderOffsets,
      },
    },
    markers: {
      city: wire.cityPositions,
      town: wire.townPositions,
    },
    labels: {
      bytes: wire.labelBytes,
      offsets: wire.labelOffsets,
      anchors: wire.labelAnchors,
      sizes: wire.labelSizes,
      items: decodeLabelItems(wire.labelBytes, wire.labelOffsets, wire.labelSizes, svgData),
    },
    landPolygonPositions: wire.landPolygonPositions,
    landPolygonOffsets: wire.landPolygonOffsets,
    svgMapJsonBytes: _encoder.encode(wire.svgJson),
    generatedFrom,
  };
}

export function compileMapExportData(data: MapExportData, mapJson?: string): MapScenePacket {
  const terrainBundle = buildExportTerrain(data);
  const landPolygonBuffers = buildLandPolygonBuffers(data.land_polygons);
  const metadata: MapSceneMetadata = {
    imageWidth: data.image_width,
    imageHeight: data.image_height,
    drawScale: data.draw_scale,
    terrainWidth: terrainBundle.terrainWidth,
    terrainHeight: terrainBundle.terrainHeight,
    textureWidth: terrainBundle.terrainWidth,
    textureHeight: terrainBundle.terrainHeight,
    elevationScale: TERRAIN_ELEVATION_SCALE,
    cityCount: data.city.length / 2,
    townCount: data.town.length / 2,
    riverCount: data.river.length,
    territoryCount: data.territory.length,
    labelCount: data.label.length,
  };

  const slopeSegments = new Float32Array((data.slope.length / 4) * 6);
  for (let index = 0; index < data.slope.length; index += 4) {
    const targetIndex = (index / 4) * 6;
    const x1 = data.slope[index] ?? 0;
    const y1 = data.slope[index + 1] ?? 0;
    const x2 = data.slope[index + 2] ?? 0;
    const y2 = data.slope[index + 3] ?? 0;
    slopeSegments[targetIndex] = (x1 - 0.5) * metadata.imageWidth;
    slopeSegments[targetIndex + 1] =
      sampleElevation(
        terrainBundle.elevations,
        metadata.terrainWidth,
        metadata.terrainHeight,
        x1,
        1 - y1,
      ) + OVERLAY_HEIGHT_OFFSET;
    slopeSegments[targetIndex + 2] = (y1 - 0.5) * metadata.imageHeight;
    slopeSegments[targetIndex + 3] = (x2 - 0.5) * metadata.imageWidth;
    slopeSegments[targetIndex + 4] =
      sampleElevation(
        terrainBundle.elevations,
        metadata.terrainWidth,
        metadata.terrainHeight,
        x2,
        1 - y2,
      ) + OVERLAY_HEIGHT_OFFSET;
    slopeSegments[targetIndex + 5] = (y2 - 0.5) * metadata.imageHeight;
  }

  const labels = buildLabelBuffers(data, metadata, terrainBundle.elevations);

  return {
    metadata,
    terrain: terrainBundle.terrain,
    textures: terrainBundle.textures,
    layers: {
      slopeSegments,
      river: buildPathLayer(
        data.river,
        metadata,
        terrainBundle.elevations,
        OVERLAY_HEIGHT_OFFSET + 0.5,
      ),
      contour: buildPathLayer(
        data.contour,
        metadata,
        terrainBundle.elevations,
        OVERLAY_HEIGHT_OFFSET,
      ),
      border: buildPathLayer(
        data.territory,
        metadata,
        terrainBundle.elevations,
        OVERLAY_HEIGHT_OFFSET + 0.25,
      ),
    },
    markers: {
      city: buildPointPositions(data.city, metadata, terrainBundle.elevations, LABEL_HEIGHT_OFFSET),
      town: buildPointPositions(
        data.town,
        metadata,
        terrainBundle.elevations,
        LABEL_HEIGHT_OFFSET - 0.7,
      ),
    },
    labels,
    landPolygonPositions: landPolygonBuffers.positions,
    landPolygonOffsets: landPolygonBuffers.offsets,
    mapJson: mapJson ?? JSON.stringify(data),
    svgMapJsonBytes: _encoder.encode(JSON.stringify(createSvgMapExportData(data))),
  };
}

export function parseMapExportJson(json: string) {
  return compileMapExportData(JSON.parse(json) as MapExportData, json);
}
