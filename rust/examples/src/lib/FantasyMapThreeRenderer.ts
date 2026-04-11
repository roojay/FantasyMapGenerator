import type {
  MapLayers,
  MapScenePacket,
  PathLayerPacket,
  RenderBackend,
  RendererPreference,
  RendererRuntimeBackend,
} from "@/types/map";
import { getSvgMapJson } from "@/types/map";

const DEFAULT_LAYERS: MapLayers = {
  slope: true,
  river: true,
  contour: true,
  border: true,
  city: true,
  town: true,
  label: true,
};

function layersEqual(a: MapLayers, b: MapLayers) {
  return (
    a.slope === b.slope &&
    a.river === b.river &&
    a.contour === b.contour &&
    a.border === b.border &&
    a.city === b.city &&
    a.town === b.town &&
    a.label === b.label
  );
}

const PAPER_BACKGROUND = 0xf7f1e3;
const WEBGPU_BACKGROUND = 0x0a2740;
const WEBGPU_SURFACE = 0x11283a;
const SLOPE_COLOR = 0x24301f;
const RIVER_COLOR = 0x5fc0df;
const RIVER_GLOW = 0x9ae6ff;
const COAST_COLOR = 0x193043;
const COAST_GLOW_COLOR = 0x92f1ee;
const BORDER_COLOR = 0x1a1e18;
const BORDER_UNDER_COLOR = 0xf7f1e3;
const CITY_OUTER_COLOR = 0x1f251c;
const CITY_INNER_COLOR = 0xf9f7ef;
const TOWN_COLOR = 0xeff1e1;
const LABEL_COLOR = 0x11151b;
const LABEL_HALO_COLOR = 0xf7f6ef;
const CAMERA_DISTANCE = 900;
const FIT_PADDING = 0.92;
const LABEL_Z_OFFSET = 2.2;
const MARKER_BASE_Z_OFFSET = 0.82;
const MARKER_LAYER_Z_STEP = 0.18;
const PREVIEW_TEXTURE_MAX_DIMENSION = 1536;
const PREVIEW_TEXTURE_MAX_TEXELS = 1_500_000;
const MALDIVES_SHORE_COLOR: [number, number, number] = [197, 241, 229];
const MALDIVES_LAGOON_COLOR: [number, number, number] = [84, 216, 210];
const MALDIVES_TURQUOISE_COLOR: [number, number, number] = [30, 176, 194];
const MALDIVES_REEF_BLUE: [number, number, number] = [18, 108, 165];
const MALDIVES_OUTER_ATOLL: [number, number, number] = [9, 63, 129];
const MALDIVES_DEEP_OCEAN: [number, number, number] = [3, 30, 84];

type ThreeRuntime = Awaited<ReturnType<typeof loadThreeRuntime>>;
type RendererStateSnapshot = {
  renderer: any;
  controls: any;
  scene: any;
  camera: any;
  renderMode: RenderBackend | null;
  layerRoots: Partial<Record<keyof MapLayers, import("three").Object3D>>;
  managedRoots: import("three").Object3D[];
  svgMarkup: string | null;
  svgMarkupCacheKey: string | null;
  svgRenderDirty: boolean;
  nativeSvgViewport: SVGGElement | null;
  lineMaterials: Array<{ resolution?: import("three").Vector2; linewidth?: number }>;
};

type RenderViewState = {
  position: [number, number, number];
  target: [number, number, number];
  zoom: number;
};

type SvgBuildWorkerRequest = {
  type: "build-svg";
  requestId: number;
  mapJson: string;
  layers: MapLayers;
};

type SvgBuildWorkerResponse =
  | {
      type: "success";
      requestId: number;
      svgMarkup: string;
    }
  | {
      type: "error";
      requestId: number;
      error: string;
    };

let threeRuntimePromise: Promise<{
  THREE: typeof import("three");
  WebGPURenderer: typeof import("three/webgpu").WebGPURenderer;
  Line2NodeMaterial: typeof import("three/webgpu").Line2NodeMaterial;
  MapControls: typeof import("three/examples/jsm/controls/MapControls.js").MapControls;
  WebGPULine2: typeof import("three/examples/jsm/lines/webgpu/Line2.js").Line2;
  LineGeometry: typeof import("three/examples/jsm/lines/LineGeometry.js").LineGeometry;
  WebGPULineSegments2: typeof import("three/examples/jsm/lines/webgpu/LineSegments2.js").LineSegments2;
  LineSegmentsGeometry: typeof import("three/examples/jsm/lines/LineSegmentsGeometry.js").LineSegmentsGeometry;
  MeshStandardNodeMaterial: typeof import("three/webgpu").MeshStandardNodeMaterial;
  MeshBasicNodeMaterial: typeof import("three/webgpu").MeshBasicNodeMaterial;
  TSL: typeof import("three/tsl");
}> | null = null;

async function loadThreeRuntime() {
  if (!threeRuntimePromise) {
    threeRuntimePromise = Promise.all([
      import("three"),
      import("three/webgpu"),
      import("three/tsl"),
      import("three/examples/jsm/controls/MapControls.js"),
      import("three/examples/jsm/lines/webgpu/Line2.js"),
      import("three/examples/jsm/lines/LineGeometry.js"),
      import("three/examples/jsm/lines/webgpu/LineSegments2.js"),
      import("three/examples/jsm/lines/LineSegmentsGeometry.js"),
    ]).then(
      ([
        THREE,
        webgpu,
        tsl,
        controls,
        webgpuLine2,
        lineGeometry,
        webgpuLineSegments2,
        lineSegmentsGeometry,
      ]) => {
        return {
          THREE,
          WebGPURenderer: webgpu.WebGPURenderer,
          Line2NodeMaterial: webgpu.Line2NodeMaterial,
          MeshStandardNodeMaterial: webgpu.MeshStandardNodeMaterial,
          MeshBasicNodeMaterial: webgpu.MeshBasicNodeMaterial,
          MapControls: controls.MapControls,
          WebGPULine2: webgpuLine2.Line2,
          LineGeometry: lineGeometry.LineGeometry,
          WebGPULineSegments2: webgpuLineSegments2.LineSegments2,
          LineSegmentsGeometry: lineSegmentsGeometry.LineSegmentsGeometry,
          TSL: tsl,
        };
      },
    );
  }

  return threeRuntimePromise;
}

// Optimization D: Prefetch Three.js modules at import time so they load in
// parallel with WASM initialization, rather than waiting for initialize().
void loadThreeRuntime();

function packetToThreeTriplets(source: Float32Array) {
  const output = new Float32Array(source.length);
  for (let index = 0; index < source.length; index += 3) {
    output[index] = source[index];
    output[index + 1] = source[index + 2];
    output[index + 2] = source[index + 1];
  }
  return output;
}

function buildTerrainGeometry(runtime: ThreeRuntime, packet: MapScenePacket) {
  if (packet.terrain.positions.length === 0 || packet.terrain.indices.length === 0) {
    return null;
  }

  const geometry = new runtime.THREE.BufferGeometry();
  geometry.setAttribute(
    "position",
    new runtime.THREE.BufferAttribute(packetToThreeTriplets(packet.terrain.positions), 3),
  );
  geometry.setAttribute(
    "normal",
    new runtime.THREE.BufferAttribute(packetToThreeTriplets(packet.terrain.normals), 3),
  );
  geometry.setAttribute("uv", new runtime.THREE.BufferAttribute(packet.terrain.uvs, 2));
  geometry.setIndex(new runtime.THREE.BufferAttribute(packet.terrain.indices, 1));
  geometry.computeBoundingBox();
  geometry.computeBoundingSphere();
  return geometry;
}

function buildTerrainLights(runtime: ThreeRuntime, packet: MapScenePacket) {
  const group = new runtime.THREE.Group();
  const maxSpan = Math.max(packet.metadata.imageWidth, packet.metadata.imageHeight);

  const hemi = new runtime.THREE.HemisphereLight(0xe7f0ff, 0x877356, 1.18);
  const keyLight = new runtime.THREE.DirectionalLight(0xfff1d3, 1.85);
  keyLight.position.set(-maxSpan * 0.42, maxSpan * 0.55, maxSpan * 1.15);

  const fillLight = new runtime.THREE.DirectionalLight(0x9fc4e0, 0.46);
  fillLight.position.set(maxSpan * 0.34, -maxSpan * 0.24, maxSpan * 0.72);

  const rimLight = new runtime.THREE.DirectionalLight(0x5eb9f5, 0.24);
  rimLight.position.set(maxSpan * 0.18, maxSpan * 0.52, -maxSpan * 0.35);

  group.add(hemi);
  group.add(keyLight);
  group.add(fillLight);
  group.add(rimLight);
  return group;
}

function clamp01(value: number) {
  return Math.max(0, Math.min(1, value));
}

function lerp(a: number, b: number, t: number) {
  return a + (b - a) * clamp01(t);
}

function smoothstep(edge0: number, edge1: number, value: number) {
  if (edge0 === edge1) {
    return value < edge0 ? 0 : 1;
  }
  const t = clamp01((value - edge0) / (edge1 - edge0));
  return t * t * (3 - 2 * t);
}

function pixelOffset(width: number, x: number, y: number) {
  return (y * width + x) * 4;
}

function sampleTextureChannel(
  data: Uint8Array,
  width: number,
  height: number,
  x: number,
  y: number,
) {
  const clampedX = Math.max(0, Math.min(width - 1, x));
  const clampedY = Math.max(0, Math.min(height - 1, y));
  return (data[pixelOffset(width, clampedX, clampedY)] ?? 0) / 255;
}

function sampleMaskTexture(data: Uint8Array, width: number, height: number, x: number, y: number) {
  return sampleTextureChannel(data, width, height, x, y) > 0.5 ? 1 : 0;
}

function coastalStrengthFromTexture(
  mask: Uint8Array,
  width: number,
  height: number,
  x: number,
  y: number,
) {
  const center = sampleMaskTexture(mask, width, height, x, y);
  let delta = 0;
  for (let oy = -1; oy <= 1; oy += 1) {
    for (let ox = -1; ox <= 1; ox += 1) {
      if (ox === 0 && oy === 0) continue;
      delta += Math.abs(center - sampleMaskTexture(mask, width, height, x + ox, y + oy));
    }
  }
  return clamp01(delta / 8);
}

function mixColor(
  a: [number, number, number],
  b: [number, number, number],
  t: number,
): [number, number, number] {
  const amount = clamp01(t);
  return [
    Math.round(a[0] + (b[0] - a[0]) * amount),
    Math.round(a[1] + (b[1] - a[1]) * amount),
    Math.round(a[2] + (b[2] - a[2]) * amount),
  ];
}

function applyContrast(value: number, contrast: number) {
  return clamp01((value - 0.5) * contrast + 0.5);
}

function hashNoise(x: number, y: number, seed: number) {
  let hash = Math.imul(x, 374761393) ^ Math.imul(y, 668265263) ^ Math.imul(seed, 2246822519);
  hash = (hash ^ (hash >>> 13)) >>> 0;
  hash = Math.imul(hash, 1274126177) >>> 0;
  return hash / 0xffffffff;
}

function valueNoise2D(x: number, y: number, seed: number) {
  const x0 = Math.floor(x);
  const y0 = Math.floor(y);
  const tx = x - x0;
  const ty = y - y0;

  const v00 = hashNoise(x0, y0, seed);
  const v10 = hashNoise(x0 + 1, y0, seed);
  const v01 = hashNoise(x0, y0 + 1, seed);
  const v11 = hashNoise(x0 + 1, y0 + 1, seed);

  const sx = smoothstep(0, 1, tx);
  const sy = smoothstep(0, 1, ty);
  const nx0 = lerp(v00, v10, sx);
  const nx1 = lerp(v01, v11, sx);
  return lerp(nx0, nx1, sy);
}

function fbm2D(x: number, y: number, octaves: number, seed: number) {
  let amplitude = 0.5;
  let frequency = 1;
  let sum = 0;
  let normalization = 0;

  for (let octave = 0; octave < octaves; octave += 1) {
    sum += valueNoise2D(x * frequency, y * frequency, seed + octave * 31) * amplitude;
    normalization += amplitude;
    amplitude *= 0.52;
    frequency *= 2.03;
  }

  return normalization > 0 ? sum / normalization : 0;
}

function buildDistanceField(
  mask: Uint8Array,
  width: number,
  height: number,
  targetLand: boolean,
) {
  const distance = new Float32Array(width * height);
  const diagonal = Math.SQRT2;
  const infinity = width + height;

  for (let y = 0; y < height; y += 1) {
    for (let x = 0; x < width; x += 1) {
      const index = y * width + x;
      const isLand = (mask[pixelOffset(width, x, y)] ?? 0) > 127;
      distance[index] = isLand === targetLand ? 0 : infinity;
    }
  }

  for (let y = 0; y < height; y += 1) {
    for (let x = 0; x < width; x += 1) {
      const index = y * width + x;
      let value = distance[index];
      if (x > 0) value = Math.min(value, distance[index - 1] + 1);
      if (y > 0) value = Math.min(value, distance[index - width] + 1);
      if (x > 0 && y > 0) value = Math.min(value, distance[index - width - 1] + diagonal);
      if (x < width - 1 && y > 0) {
        value = Math.min(value, distance[index - width + 1] + diagonal);
      }
      distance[index] = value;
    }
  }

  for (let y = height - 1; y >= 0; y -= 1) {
    for (let x = width - 1; x >= 0; x -= 1) {
      const index = y * width + x;
      let value = distance[index];
      if (x < width - 1) value = Math.min(value, distance[index + 1] + 1);
      if (y < height - 1) value = Math.min(value, distance[index + width] + 1);
      if (x < width - 1 && y < height - 1) {
        value = Math.min(value, distance[index + width + 1] + diagonal);
      }
      if (x > 0 && y < height - 1) {
        value = Math.min(value, distance[index + width - 1] + diagonal);
      }
      distance[index] = value;
    }
  }

  return distance;
}

function hashString(value: string) {
  let hash = 2166136261;
  for (let index = 0; index < value.length; index += 1) {
    hash ^= value.charCodeAt(index);
    hash = Math.imul(hash, 16777619);
  }
  return (hash >>> 0).toString(36);
}

function getTextureMaxAnisotropy(renderer: any) {
  try {
    return renderer?.capabilities?.getMaxAnisotropy?.() ?? 1;
  } catch {
    return 1;
  }
}

function configureTextureForRenderer(texture: import("three").Texture, renderer: any) {
  texture.anisotropy = Math.max(1, getTextureMaxAnisotropy(renderer));
}

function isPowerOfTwo(value: number) {
  return value > 0 && (value & (value - 1)) === 0;
}

function clampPreviewTextureSize(width: number, height: number) {
  const safeWidth = Math.max(1, Math.round(width));
  const safeHeight = Math.max(1, Math.round(height));
  const texelCount = safeWidth * safeHeight;
  if (
    safeWidth <= PREVIEW_TEXTURE_MAX_DIMENSION &&
    safeHeight <= PREVIEW_TEXTURE_MAX_DIMENSION &&
    texelCount <= PREVIEW_TEXTURE_MAX_TEXELS
  ) {
    return { width: safeWidth, height: safeHeight };
  }

  const dimensionScale = Math.min(
    PREVIEW_TEXTURE_MAX_DIMENSION / safeWidth,
    PREVIEW_TEXTURE_MAX_DIMENSION / safeHeight,
  );
  const texelScale = Math.sqrt(PREVIEW_TEXTURE_MAX_TEXELS / texelCount);
  const scale = Math.min(dimensionScale, texelScale, 1);

  return {
    width: Math.max(1, Math.round(safeWidth * scale)),
    height: Math.max(1, Math.round(safeHeight * scale)),
  };
}

function dataTexture(
  runtime: ThreeRuntime,
  data: Uint8Array,
  width: number,
  height: number,
  srgb = false,
) {
  const texture = new runtime.THREE.DataTexture(data, width, height, runtime.THREE.RGBAFormat);
  const canMipMap = isPowerOfTwo(width) && isPowerOfTwo(height);
  texture.flipY = true;
  texture.wrapS = runtime.THREE.ClampToEdgeWrapping;
  texture.wrapT = runtime.THREE.ClampToEdgeWrapping;
  texture.minFilter = canMipMap
    ? runtime.THREE.LinearMipmapLinearFilter
    : runtime.THREE.LinearFilter;
  texture.magFilter = runtime.THREE.LinearFilter;
  texture.generateMipmaps = canMipMap;
  if (srgb) {
    texture.colorSpace = runtime.THREE.SRGBColorSpace;
  }
  texture.needsUpdate = true;
  return texture;
}

/**
 * PERF: Single-channel (R8) DataTexture — 4× smaller than RGBA for scalar
 * data like height, roughness, AO, waterAlpha, and coastGlow.  The shader
 * reads `.r`; Three.js fills `.g = .b = 0`, `.a = 1` automatically.
 */
function dataTextureR8(
  runtime: ThreeRuntime,
  data: Uint8Array,
  width: number,
  height: number,
) {
  const texture = new runtime.THREE.DataTexture(data, width, height, runtime.THREE.RedFormat);
  const canMipMap = isPowerOfTwo(width) && isPowerOfTwo(height);
  texture.flipY = true;
  texture.wrapS = runtime.THREE.ClampToEdgeWrapping;
  texture.wrapT = runtime.THREE.ClampToEdgeWrapping;
  texture.minFilter = canMipMap
    ? runtime.THREE.LinearMipmapLinearFilter
    : runtime.THREE.LinearFilter;
  texture.magFilter = runtime.THREE.LinearFilter;
  texture.generateMipmaps = canMipMap;
  texture.needsUpdate = true;
  return texture;
}

function disposeMaterial(
  material: import("three").Material & {
    map?: import("three").Texture | null;
    alphaMap?: import("three").Texture | null;
    aoMap?: import("three").Texture | null;
    bumpMap?: import("three").Texture | null;
    emissiveMap?: import("three").Texture | null;
    roughnessMap?: import("three").Texture | null;
  },
) {
  const textures = new Set<import("three").Texture>();
  if (material.map) textures.add(material.map);
  if (material.alphaMap) textures.add(material.alphaMap);
  if (material.aoMap) textures.add(material.aoMap);
  if (material.bumpMap) textures.add(material.bumpMap);
  if (material.emissiveMap) textures.add(material.emissiveMap);
  if (material.roughnessMap) textures.add(material.roughnessMap);
  for (const texture of textures) {
    texture.dispose();
  }
  material.dispose();
}

function buildSurfaceTextures(runtime: ThreeRuntime, packet: MapScenePacket) {
  const width = packet.metadata.textureWidth;
  const height = packet.metadata.textureHeight;
  const precomputedTerrainAlbedo = packet.textures.terrainAlbedo;
  const precomputedRoughness = packet.textures.roughness;
  const precomputedAo = packet.textures.ao;
  const precomputedWaterColor = packet.textures.waterColor;
  const precomputedWaterAlpha = packet.textures.waterAlpha;
  const precomputedCoastGlow = packet.textures.coastGlow;

  if (
    precomputedTerrainAlbedo &&
    precomputedRoughness &&
    precomputedAo &&
    precomputedWaterColor &&
    precomputedWaterAlpha &&
    precomputedCoastGlow
  ) {
    return {
      terrainAlbedo: dataTexture(runtime, precomputedTerrainAlbedo, width, height, true),
      height: dataTextureR8(runtime, packet.textures.height, width, height),
      roughness: dataTextureR8(runtime, precomputedRoughness, width, height),
      ao: dataTextureR8(runtime, precomputedAo, width, height),
      waterColor: dataTexture(runtime, precomputedWaterColor, width, height, true),
      waterAlpha: dataTextureR8(runtime, precomputedWaterAlpha, width, height),
      coastGlow: dataTextureR8(runtime, precomputedCoastGlow, width, height),
    };
  }

  const sourceAlbedo = packet.textures.albedo;
  if (!sourceAlbedo) {
    throw new Error("Fallback surface textures require albedo data");
  }
  const sourceHeight = packet.textures.height;
  const sourceFlux = packet.textures.flux;
  const sourceMask = packet.textures.landMask;
  const distanceToLand = buildDistanceField(sourceMask, width, height, true);
  const distanceToWater = buildDistanceField(sourceMask, width, height, false);
  const terrainAlbedo = new Uint8Array(sourceAlbedo.length);
  const roughness = new Uint8Array(sourceAlbedo.length);
  const ao = new Uint8Array(sourceAlbedo.length);
  const waterColor = new Uint8Array(sourceAlbedo.length);
  const waterAlpha = new Uint8Array(sourceAlbedo.length);
  const coastGlow = new Uint8Array(sourceAlbedo.length);

  for (let y = 0; y < height; y += 1) {
    for (let x = 0; x < width; x += 1) {
      const pixelIndex = y * width + x;
      const offset = pixelOffset(width, x, y);
      const heightValue = (sourceHeight[offset] ?? 0) / 255;
      const fluxValue = (sourceFlux[offset] ?? 0) / 255;
      const isLand = (sourceMask[offset] ?? 0) > 127;
      const coast = coastalStrengthFromTexture(sourceMask, width, height, x, y);
      const landDistance = distanceToWater[pixelIndex] ?? 0;
      const waterDistance = distanceToLand[pixelIndex] ?? 0;
      const left = sampleTextureChannel(sourceHeight, width, height, x - 1, y);
      const right = sampleTextureChannel(sourceHeight, width, height, x + 1, y);
      const up = sampleTextureChannel(sourceHeight, width, height, x, y - 1);
      const down = sampleTextureChannel(sourceHeight, width, height, x, y + 1);
      const relief = clamp01((Math.abs(right - left) + Math.abs(down - up)) * 3.4);
      const macroNoise = fbm2D(x / 42 + heightValue * 2.7, y / 42 + fluxValue * 2.1, 4, 17);
      const detailNoise = fbm2D(x / 14 + 9.3, y / 14 + 6.1, 3, 37);
      const ridgeNoise = 1 - Math.abs(fbm2D(x / 22 + relief * 7.5, y / 22 + heightValue * 5.4, 4, 61) * 2 - 1);
      const fertileMix = clamp01(fluxValue * 0.84 + (1 - heightValue) * 0.12 + (macroNoise - 0.5) * 0.18);
      const bathymetry = clamp01((1 - heightValue) * 0.26 + smoothstep(0, 54, waterDistance) * 0.82);
      const beachMix = (1 - smoothstep(1.4, 10, landDistance)) * (1 - smoothstep(0.16, 0.34, heightValue));
      const lowlandMix = smoothstep(0.06, 0.22, heightValue) * (1 - smoothstep(0.4, 0.6, heightValue));
      const meadowMix =
        smoothstep(0.12, 0.28, heightValue) *
        (1 - smoothstep(0.5, 0.72, heightValue)) *
        clamp01(fertileMix * 0.84 + macroNoise * 0.2);
      const forestMix =
        smoothstep(0.18, 0.42, heightValue) *
        (1 - smoothstep(0.58, 0.8, heightValue)) *
        clamp01(fertileMix * 1.05 + detailNoise * 0.16);
      const rockMix = clamp01(
        smoothstep(0.52, 0.84, heightValue) * 0.72 +
          smoothstep(0.22, 0.58, relief) * 0.56 +
          smoothstep(0.46, 0.74, ridgeNoise) * 0.18,
      );
      const cliffMix =
        smoothstep(0.28, 0.64, relief) * smoothstep(0.22, 0.62, heightValue);
      const snowMix = clamp01(
        smoothstep(0.78, 0.92, heightValue) +
          smoothstep(0.56, 0.82, heightValue) * smoothstep(0.52, 0.8, relief) * 0.2,
      );
      const shoreWaterMix = 1 - smoothstep(0.8, 11, waterDistance);
      const shelfWaterMix =
        smoothstep(4, 20, waterDistance) * (1 - smoothstep(20, 56, waterDistance));
      const deepWaterMix = smoothstep(28, 74, waterDistance);

      let color: [number, number, number] = [
        sourceAlbedo[offset] ?? 0,
        sourceAlbedo[offset + 1] ?? 0,
        sourceAlbedo[offset + 2] ?? 0,
      ];

      if (isLand) {
        color = [
          Math.round(applyContrast(color[0] / 255, 1.08) * 255),
          Math.round(applyContrast(color[1] / 255, 1.1) * 255),
          Math.round(applyContrast(color[2] / 255, 1.06) * 255),
        ];
        color = mixColor(
          color,
          [201, 187, 146],
          beachMix * 0.9,
        );
        color = mixColor(
          color,
          [132, 165, 96],
          meadowMix * (0.55 + macroNoise * 0.22),
        );
        color = mixColor(
          color,
          [78, 103, 58],
          forestMix * (0.68 + detailNoise * 0.16),
        );
        color = mixColor(
          color,
          [138, 126, 104],
          rockMix * 0.52,
        );
        color = mixColor(
          color,
          [105, 98, 87],
          cliffMix * 0.42,
        );
        color = mixColor(
          color,
          [239, 242, 245],
          snowMix,
        );
        color = mixColor(
          color,
          [118, 146, 92],
          clamp01(lowlandMix * 0.1 + Math.max(0, macroNoise - 0.56) * 0.12),
        );
        color = mixColor(
          color,
          [95, 87, 74],
          clamp01(cliffMix * 0.08 + Math.max(0, 0.5 - macroNoise) * 0.08),
        );
      } else {
        color = [
          Math.round(lerp(190, 93, smoothstep(0.06, 0.24, bathymetry))),
          Math.round(lerp(176, 141, smoothstep(0.06, 0.24, bathymetry))),
          Math.round(lerp(138, 121, smoothstep(0.06, 0.24, bathymetry))),
        ];
        color = mixColor(color, [82, 132, 126], shelfWaterMix * 0.24 + detailNoise * 0.08);
        color = mixColor(color, [57, 87, 105], smoothstep(0.26, 0.58, bathymetry) * 0.6);
        color = mixColor(color, [29, 46, 71], smoothstep(0.56, 0.92, bathymetry) * 0.88);
        color = mixColor(color, [212, 198, 160], shoreWaterMix * 0.48);
      }

      const roughnessValue = isLand
        ? clamp01(
            0.92 -
              fertileMix * 0.14 +
              beachMix * 0.05 -
              snowMix * 0.22 +
              rockMix * 0.08 +
              (0.5 - detailNoise) * 0.06,
          )
        : clamp01(0.88 + bathymetry * 0.08 - shoreWaterMix * 0.08 + shelfWaterMix * 0.04);
      const aoValue = isLand
        ? clamp01(
            0.94 -
              relief * 0.34 -
              cliffMix * 0.1 +
              fertileMix * 0.06 +
              Math.max(0, 0.5 - ridgeNoise) * 0.06,
          )
        : clamp01(0.9 - shelfWaterMix * 0.08 - shoreWaterMix * 0.06 + deepWaterMix * 0.04);

      terrainAlbedo[offset] = color[0];
      terrainAlbedo[offset + 1] = color[1];
      terrainAlbedo[offset + 2] = color[2];
      terrainAlbedo[offset + 3] = 255;

      const roughnessEncoded = Math.round(roughnessValue * 255);
      roughness[offset] = roughnessEncoded;
      roughness[offset + 1] = roughnessEncoded;
      roughness[offset + 2] = roughnessEncoded;
      roughness[offset + 3] = 255;

      const aoEncoded = Math.round(aoValue * 255);
      ao[offset] = aoEncoded;
      ao[offset + 1] = aoEncoded;
      ao[offset + 2] = aoEncoded;
      ao[offset + 3] = 255;

      const waterTintNoise = fbm2D(x / 18 + 4.2, y / 18 + 11.6, 3, 89);
      let waterTint = mixColor(
        MALDIVES_SHORE_COLOR,
        MALDIVES_LAGOON_COLOR,
        smoothstep(0.04, 0.22, bathymetry),
      );
      waterTint = mixColor(
        waterTint,
        MALDIVES_TURQUOISE_COLOR,
        smoothstep(0.18, 0.4, bathymetry),
      );
      waterTint = mixColor(waterTint, MALDIVES_REEF_BLUE, smoothstep(0.34, 0.64, bathymetry));
      waterTint = mixColor(waterTint, MALDIVES_OUTER_ATOLL, smoothstep(0.58, 0.84, bathymetry));
      waterTint = mixColor(waterTint, MALDIVES_DEEP_OCEAN, smoothstep(0.82, 1, bathymetry));
      waterTint = mixColor(
        waterTint,
        [224, 246, 228],
        shoreWaterMix * 0.14 + (1 - smoothstep(0.24, 0.58, bathymetry)) * 0.12,
      );
      waterTint = mixColor(
        waterTint,
        [144, 244, 235],
        shoreWaterMix * clamp01(0.12 + waterTintNoise * 0.18),
      );
      waterTint = mixColor(
        waterTint,
        [6, 24, 68],
        deepWaterMix * clamp01(0.16 + macroNoise * 0.16),
      );

      const waterOpacity = isLand
        ? 0
        : clamp01(0.52 + bathymetry * 0.34 + deepWaterMix * 0.08 - shoreWaterMix * 0.18);
      const glowOpacity = isLand
        ? 0
        : clamp01(shoreWaterMix * 0.86 + shelfWaterMix * 0.12 + coast * 0.26);

      waterColor[offset] = waterTint[0];
      waterColor[offset + 1] = waterTint[1];
      waterColor[offset + 2] = waterTint[2];
      waterColor[offset + 3] = 255;

      const waterAlphaEncoded = Math.round(waterOpacity * 255);
      waterAlpha[offset] = waterAlphaEncoded;
      waterAlpha[offset + 1] = waterAlphaEncoded;
      waterAlpha[offset + 2] = waterAlphaEncoded;
      waterAlpha[offset + 3] = 255;

      const glowEncoded = Math.round(glowOpacity * 255);
      coastGlow[offset] = glowEncoded;
      coastGlow[offset + 1] = glowEncoded;
      coastGlow[offset + 2] = glowEncoded;
      coastGlow[offset + 3] = 255;
    }
  }

  return {
    terrainAlbedo: dataTexture(runtime, terrainAlbedo, width, height, true),
    height: dataTexture(runtime, sourceHeight, width, height),
    roughness: dataTexture(runtime, roughness, width, height),
    ao: dataTexture(runtime, ao, width, height),
    waterColor: dataTexture(runtime, waterColor, width, height, true),
    waterAlpha: dataTexture(runtime, waterAlpha, width, height),
    coastGlow: dataTexture(runtime, coastGlow, width, height),
  };
}

function buildOverlayPlane(
  runtime: ThreeRuntime,
  width: number,
  height: number,
  z: number,
  material: import("three").Material,
) {
  const mesh = new runtime.THREE.Mesh(new runtime.THREE.PlaneGeometry(width, height), material);
  mesh.position.set(0, 0, z);
  return mesh;
}

function buildCoastGlowMaterial(runtime: ThreeRuntime, alphaMap: import("three").Texture) {
  const { MeshBasicNodeMaterial, TSL } = runtime;
  const { texture, uv, float, vec4, color, sin, time, clamp } = TSL;

  const uv0 = uv();
  const alpha = texture(alphaMap, uv0).r;
  const glowIntensity = float(0.4).add(sin(time.mul(float(0.46))).mul(float(0.07)));
  const banding = sin(uv0.x.mul(float(82)).add(time.mul(float(0.8))))
    .mul(sin(uv0.y.mul(float(63)).sub(time.mul(float(0.62)))))
    .mul(float(0.08))
    .add(float(0.96));
  const glowAlpha = clamp(alpha.mul(glowIntensity).mul(banding), float(0), float(0.82));

  const material = new MeshBasicNodeMaterial();
  material.colorNode = vec4(color(COAST_GLOW_COLOR).rgb, glowAlpha);
  material.transparent = true;
  material.depthTest = false;
  material.depthWrite = false;
  material.blending = runtime.THREE.AdditiveBlending;
  material.toneMapped = false;

  return material;
}

function buildWaterMaterial(
  runtime: ThreeRuntime,
  colorMap: import("three").Texture,
  alphaMap: import("three").Texture,
  depthMap: import("three").Texture,
  coastMap: import("three").Texture,
) {
  const { MeshStandardNodeMaterial, TSL } = runtime;
  const { texture, uv, float, vec3, vec4, sin, cos, time, mix, clamp, smoothstep } = TSL;

  const uv0 = uv();
  const timeValue = time;
  const animatedUV = uv0.add(
    vec3(
      sin(timeValue.mul(float(0.3)).add(uv0.y.mul(float(10)))).mul(float(0.002)),
      cos(timeValue.mul(float(0.25)).add(uv0.x.mul(float(8)))).mul(float(0.0015)),
      float(0),
    ).xy,
  );
  const waterColor = texture(colorMap, animatedUV).rgb;
  const waterAlpha = texture(alphaMap, uv0).r;
  const waterHeight = texture(depthMap, uv0).r;
  const coastMask = texture(coastMap, uv0).r;

  const depthFactor = clamp(float(1).sub(waterHeight), float(0), float(1));
  const nearshoreMask = smoothstep(float(0.14), float(0.9), coastMask);
  const deepOceanMask = smoothstep(float(0.42), float(0.92), depthFactor);
  const sandbarColor = vec3(0.77, 0.95, 0.9);
  const lagoonColor = vec3(0.33, 0.84, 0.82);
  const shelfColor = vec3(0.12, 0.53, 0.69);
  const reefBlue = vec3(0.07, 0.32, 0.57);
  const abyssColor = vec3(0.02, 0.09, 0.26);

  let proceduralColor = mix(
    sandbarColor,
    lagoonColor,
    smoothstep(float(0.02), float(0.16), depthFactor),
  );
  proceduralColor = mix(
    proceduralColor,
    shelfColor,
    smoothstep(float(0.14), float(0.38), depthFactor),
  );
  proceduralColor = mix(
    proceduralColor,
    reefBlue,
    smoothstep(float(0.34), float(0.72), depthFactor),
  );
  proceduralColor = mix(proceduralColor, abyssColor, deepOceanMask);
  proceduralColor = mix(
    proceduralColor,
    vec3(0.7, 0.96, 0.93),
    nearshoreMask.mul(float(0.24)),
  );

  const swell = sin(uv0.x.mul(float(44)).add(timeValue.mul(float(0.86))))
    .mul(cos(uv0.y.mul(float(34)).sub(timeValue.mul(float(0.58)))));
  const ripples = sin(uv0.x.mul(float(98)).sub(timeValue.mul(float(1.7))))
    .mul(sin(uv0.y.mul(float(72)).add(timeValue.mul(float(1.18)))));
  const waveHighlight = swell.mul(float(0.04)).add(float(1));
  const caustics = nearshoreMask
    .mul(ripples.mul(float(0.5)).add(float(0.5)))
    .mul(float(0.18));

  const baseWater = waterColor.mul(proceduralColor).mul(waveHighlight);
  const waterWithHighlights = mix(
    baseWater,
    baseWater.add(vec3(0.24, 0.28, 0.24)),
    caustics,
  );
  const surfaceAlpha = clamp(
    waterAlpha.add(deepOceanMask.mul(float(0.04))).sub(nearshoreMask.mul(float(0.08))),
    float(0.42),
    float(0.95),
  );

  const material = new MeshStandardNodeMaterial();
  material.colorNode = vec4(waterWithHighlights, surfaceAlpha);
  material.roughnessNode = float(0.1)
    .add(deepOceanMask.mul(float(0.08))) as unknown as typeof material.roughnessNode;
  material.metalnessNode = float(0.03) as unknown as typeof material.metalnessNode;
  material.emissiveNode = waterWithHighlights
    .mul(nearshoreMask.mul(float(0.04)).add(float(0.02))) as unknown as typeof material.emissiveNode;
  material.transparent = true;
  material.depthWrite = false;
  material.lights = true;

  return material;
}

function configureOverlayMaterial(material: import("three").Material) {
  material.depthTest = false;
  material.depthWrite = false;
}

function setRenderOrder(root: import("three").Object3D, renderOrder: number) {
  root.traverse((child) => {
    child.renderOrder = renderOrder;
  });
}

function setLayeredRenderOrder(root: import("three").Object3D, baseRenderOrder: number) {
  root.renderOrder = baseRenderOrder;
  root.children.forEach((child, layerIndex) => {
    child.traverse((descendant) => {
      descendant.renderOrder = baseRenderOrder + layerIndex;
    });
  });
}

async function svgMarkupToTexture(
  runtime: ThreeRuntime,
  svgMarkup: string,
  width: number,
  height: number,
) {
  const blob = new Blob([svgMarkup], { type: "image/svg+xml;charset=utf-8" });
  const url = URL.createObjectURL(blob);

  try {
    const image = await new Promise<HTMLImageElement>((resolve, reject) => {
      const nextImage = new Image();
      nextImage.onload = () => resolve(nextImage);
      nextImage.onerror = () => reject(new Error("Failed to load SVG texture"));
      nextImage.src = url;
    });

    const canvas = document.createElement("canvas");
    canvas.width = width;
    canvas.height = height;
    const context = canvas.getContext("2d");
    if (!context) {
      throw new Error("Failed to create texture canvas context");
    }

    context.clearRect(0, 0, width, height);
    context.drawImage(image, 0, 0, width, height);

    const texture = new runtime.THREE.CanvasTexture(canvas);
    texture.flipY = true;
    texture.wrapS = runtime.THREE.ClampToEdgeWrapping;
    texture.wrapT = runtime.THREE.ClampToEdgeWrapping;
    texture.minFilter = runtime.THREE.LinearFilter;
    texture.magFilter = runtime.THREE.LinearFilter;
    texture.colorSpace = runtime.THREE.SRGBColorSpace;
    texture.generateMipmaps = false;
    texture.needsUpdate = true;
    return texture;
  } finally {
    URL.revokeObjectURL(url);
  }
}

function svgMarkupToRoot(svgMarkup: string) {
  const parser = new DOMParser();
  const documentNode = parser.parseFromString(svgMarkup, "image/svg+xml");
  const svgElement = documentNode.documentElement;
  if (!(svgElement instanceof SVGSVGElement)) {
    throw new Error("Failed to parse SVG markup into an SVG root");
  }

  return document.importNode(svgElement, true) as SVGSVGElement;
}

function createNativeSvgRenderer() {
  const domElement = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  domElement.setAttribute("xmlns", "http://www.w3.org/2000/svg");
  domElement.setAttribute("overflow", "hidden");
  domElement.setAttribute("preserveAspectRatio", "xMidYMid meet");

  return {
    domElement,
    setClearColor() {},
    setPixelRatio() {},
    setSize(width: number, height: number) {
      domElement.setAttribute("width", `${Math.max(1, width)}`);
      domElement.setAttribute("height", `${Math.max(1, height)}`);
      domElement.style.width = "100%";
      domElement.style.height = "100%";
    },
    render() {},
    dispose() {
      domElement.replaceChildren();
    },
  };
}

function disposeObject(root: import("three").Object3D) {
  root.traverse((child) => {
    const mesh = child as import("three").Mesh;
    if (mesh.geometry && "dispose" in mesh.geometry) {
      mesh.geometry.dispose();
    }

    if (Array.isArray(mesh.material)) {
      for (const material of mesh.material) {
        disposeMaterial(
          material as import("three").Material & { map?: import("three").Texture | null },
        );
      }
    } else if (mesh.material && "dispose" in mesh.material) {
      disposeMaterial(
        mesh.material as import("three").Material & { map?: import("three").Texture | null },
      );
    }
  });
}

function resolveLabelFontFamily(fontface: string) {
  if (fontface === "Times New Roman") {
    return "Times New Roman, Georgia, serif";
  }
  return `${fontface}, serif`;
}

export class FantasyMapThreeRenderer {
  private stage: HTMLDivElement;
  private runtime: ThreeRuntime | null = null;
  private modeSnapshots: Partial<Record<RenderBackend, RendererStateSnapshot>> = {};
  private modeDirty: Partial<Record<RenderBackend, boolean>> = { svg: true, webgpu: true };
  private renderer: any = null;
  private controls: any = null;
  private scene: any = null;
  private camera: any = null;
  private renderMode: RenderBackend | null = null;
  private availableModes: RenderBackend[] = [];
  private packet: MapScenePacket | null = null;
  private layers: MapLayers = { ...DEFAULT_LAYERS };
  private layerRoots: Partial<Record<keyof MapLayers, import("three").Object3D>> = {};
  private managedRoots: import("three").Object3D[] = [];
  private svgMarkup: string | null = null;
  private svgMarkupCacheKey: string | null = null;
  private svgRenderDirty = true;
  private nativeSvgViewport: SVGGElement | null = null;
  private svgMarkupPromise: Promise<string> | null = null;
  private svgMarkupPromiseKey: string | null = null;
  private svgWorker: Worker | null = null;
  private svgWorkerRequestId = 0;
  private readonly svgWorkerPending = new Map<
    number,
    {
      resolve: (svgMarkup: string) => void;
      reject: (error: Error) => void;
    }
  >();
  private lineMaterials: Array<{ resolution?: import("three").Vector2; linewidth?: number }> = [];
  private svgSourceHash: string | null = null;
  private lineMaterialCache = new Map<string, any>();
  constructor(stage: HTMLDivElement) {
    this.stage = stage;
  }

  get currentMode() {
    return this.renderMode;
  }

  get actualBackend(): RendererRuntimeBackend {
    if (!this.renderMode) {
      return "unknown";
    }

    if (this.renderMode === "svg") {
      return "svg";
    }

    const backend = this.renderer?.backend as
      | { isWebGPUBackend?: boolean; isWebGLBackend?: boolean }
      | undefined;

    if (backend?.isWebGPUBackend) {
      return "webgpu";
    }

    if (backend?.isWebGLBackend) {
      return "webgl2";
    }

    return "unknown";
  }

  getMapSize() {
    if (!this.packet) return null;

    return {
      width: this.packet.metadata.imageWidth,
      height: this.packet.metadata.imageHeight,
    };
  }

  private ensureSvgWorker() {
    if (!this.svgWorker) {
      this.svgWorker = new Worker(new URL("../workers/svgRender.worker.ts", import.meta.url), {
        type: "module",
      });
      this.svgWorker.addEventListener("message", this.handleSvgWorkerMessage);
      this.svgWorker.addEventListener("error", this.handleSvgWorkerError);
    }

    return this.svgWorker;
  }

  private readonly handleSvgWorkerMessage = (event: MessageEvent<SvgBuildWorkerResponse>) => {
    const pending = this.svgWorkerPending.get(event.data.requestId);
    if (!pending) {
      return;
    }

    this.svgWorkerPending.delete(event.data.requestId);

    if (event.data.type === "success") {
      pending.resolve(event.data.svgMarkup);
      return;
    }

    pending.reject(new Error(event.data.error));
  };

  private readonly handleSvgWorkerError = (event: ErrorEvent) => {
    const message = event.message || "SVG worker failed";
    for (const pending of this.svgWorkerPending.values()) {
      pending.reject(new Error(message));
    }
    this.svgWorkerPending.clear();
  };

  private terminateSvgWorker() {
    if (!this.svgWorker) {
      return;
    }

    this.svgWorker.removeEventListener("message", this.handleSvgWorkerMessage);
    this.svgWorker.removeEventListener("error", this.handleSvgWorkerError);
    this.svgWorker.terminate();
    this.svgWorker = null;

    for (const pending of this.svgWorkerPending.values()) {
      pending.reject(new Error("SVG worker terminated"));
    }
    this.svgWorkerPending.clear();
  }

  private makeSvgCacheKey() {
    const svgJson = this.packet ? getSvgMapJson(this.packet) : undefined;
    if (!svgJson) {
      return null;
    }

    if (this.svgSourceHash === null) {
      this.svgSourceHash = `${svgJson.length}:${hashString(svgJson)}`;
    }

    return JSON.stringify({
      mapJsonHash: this.svgSourceHash,
      layers: this.layers,
    });
  }

  private invalidateSvgCache() {
    this.svgMarkup = null;
    this.svgMarkupCacheKey = null;
    this.svgMarkupPromise = null;
    this.svgMarkupPromiseKey = null;
    this.svgRenderDirty = true;
    this.modeDirty.svg = true;
  }

  private invalidateRenderCaches() {
    this.invalidateSvgCache();
    this.modeDirty.webgpu = true;
    for (const mat of this.lineMaterialCache.values()) {
      if (typeof mat.dispose === "function") mat.dispose();
    }
    this.lineMaterialCache.clear();
  }

  private captureViewState(): RenderViewState | null {
    if (!this.camera || !this.controls) {
      return null;
    }

    return {
      position: [this.camera.position.x, this.camera.position.y, this.camera.position.z],
      target: [this.controls.target.x, this.controls.target.y, this.controls.target.z],
      zoom: this.camera.zoom,
    };
  }

  private applyViewState(viewState: RenderViewState | null) {
    if (!viewState || !this.camera || !this.controls) {
      return;
    }

    this.camera.position.set(...viewState.position);
    this.camera.zoom = viewState.zoom;
    this.camera.updateProjectionMatrix();
    this.controls.target.set(...viewState.target);
    this.controls.update();
  }

  private setSnapshotVisibility(snapshot: RendererStateSnapshot | null, visible: boolean) {
    const domElement = snapshot?.renderer?.domElement as HTMLElement | SVGElement | undefined;
    if (!domElement) {
      return;
    }

    domElement.style.opacity = visible ? "1" : "0";
    domElement.style.pointerEvents = visible ? "" : "none";
    domElement.style.visibility = visible ? "visible" : "hidden";
  }

  private stashSnapshot(snapshot: RendererStateSnapshot | null) {
    if (!snapshot?.renderMode) {
      return;
    }

    this.setSnapshotVisibility(snapshot, false);
    this.modeSnapshots[snapshot.renderMode] = snapshot;
  }

  private disposeCachedSnapshots() {
    for (const snapshot of Object.values(this.modeSnapshots)) {
      if (snapshot) {
        this.disposeSnapshot(snapshot);
      }
    }
    this.modeSnapshots = {};
  }

  async initialize(preferredMode: RendererPreference = "auto") {
    this.runtime = await loadThreeRuntime();
    await this.detectAvailableModes();
    const activeSnapshot = this.snapshotState();
    const activeMode = this.renderMode;
    const viewState = this.captureViewState();

    const candidates =
      preferredMode === "auto"
        ? this.availableModes
        : this.availableModes.includes(preferredMode as RenderBackend)
          ? [preferredMode as RenderBackend]
          : this.availableModes;

    for (const mode of candidates) {
      try {
        if (mode === activeMode && activeSnapshot) {
          if (this.modeDirty[mode] && this.packet) {
            await this.rebuildScene();
            this.modeDirty[mode] = false;
            this.resize();
          }

          return {
            mode,
            availableModes: [...this.availableModes],
          };
        }

        const cachedSnapshot = this.modeSnapshots[mode];

        if (cachedSnapshot) {
          this.clearStateReferences();
          this.restoreSnapshot(cachedSnapshot);
          delete this.modeSnapshots[mode];

          if (this.modeDirty[mode] && this.packet) {
            await this.rebuildScene();
            this.modeDirty[mode] = false;
          }

          this.applyLayerVisibility();
          this.applyViewState(viewState);
          this.resize();
          this.setSnapshotVisibility(cachedSnapshot, true);
          this.stashSnapshot(activeSnapshot);

          return {
            mode,
            availableModes: [...this.availableModes],
          };
        }

        this.clearStateReferences();
        await this.initializeMode(mode, Boolean(activeSnapshot));
        this.applyViewState(viewState);
        this.modeDirty[mode] = false;
        this.resize();
        this.stashSnapshot(activeSnapshot);

        return {
          mode,
          availableModes: [...this.availableModes],
        };
      } catch (error) {
        const failedSnapshot = this.snapshotState();
        this.clearStateReferences();
        if (failedSnapshot) {
          this.disposeSnapshot(failedSnapshot);
        }

        if (activeSnapshot) {
          this.restoreSnapshot(activeSnapshot);
          this.setSnapshotVisibility(activeSnapshot, true);
        }

        if (mode === candidates[candidates.length - 1]) {
          throw error;
        }
      }
    }

    throw new Error("No rendering backend available");
  }

  destroy() {
    this.cleanup();
  }

  cleanup() {
    const snapshot = this.snapshotState();
    this.clearStateReferences();
    if (snapshot) {
      this.disposeSnapshot(snapshot);
    }
    this.disposeCachedSnapshots();
    this.terminateSvgWorker();
  }

  loadMapData(packet: MapScenePacket) {
    if (this.packet === packet) {
      return;
    }

    this.packet = packet;
    this.svgSourceHash = null;
    this.invalidateRenderCaches();
    this.disposeCachedSnapshots();
  }

  setLayers(layers: MapLayers) {
    if (layersEqual(this.layers, layers)) {
      return;
    }

    this.layers = { ...layers };
    this.invalidateSvgCache();

    if (this.modeSnapshots.webgpu) {
      this.applyLayerVisibility(this.modeSnapshots.webgpu.layerRoots);
    }

    if (this.renderMode === "webgpu") {
      this.modeDirty.webgpu = false;
      this.applyLayerVisibility();
      this.draw();
    }
  }

  async render() {
    if (!this.packet || !this.runtime || !this.renderMode) {
      return;
    }

    if (!this.renderer || !this.scene || !this.camera) {
      return;
    }

    if (this.modeDirty[this.renderMode]) {
      await this.rebuildScene();
      this.modeDirty[this.renderMode] = false;
      this.resize();
      return;
    }

    this.applyLayerVisibility();
    this.draw();
  }

  resize() {
    if (!this.renderer || !this.camera) return;

    const width = Math.max(1, this.stage.clientWidth || this.packet?.metadata.imageWidth || 1);
    const height = Math.max(1, this.stage.clientHeight || this.packet?.metadata.imageHeight || 1);

    this.renderer.setSize(width, height);
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));

    this.camera.left = -width / 2;
    this.camera.right = width / 2;
    this.camera.top = height / 2;
    this.camera.bottom = -height / 2;
    this.camera.updateProjectionMatrix();

    for (const material of this.lineMaterials) {
      material.resolution?.set(width, height);
    }

    this.draw();
  }

  fitToScreen() {
    if (!this.camera || !this.packet || !this.controls) return;

    const width = Math.max(1, this.stage.clientWidth || this.packet.metadata.imageWidth);
    const height = Math.max(1, this.stage.clientHeight || this.packet.metadata.imageHeight);
    const zoom =
      Math.min(width / this.packet.metadata.imageWidth, height / this.packet.metadata.imageHeight) *
      FIT_PADDING;

    this.camera.position.set(0, 0, CAMERA_DISTANCE);
    this.camera.zoom = zoom;
    this.controls.target.set(0, 0, 0);
    this.camera.updateProjectionMatrix();
    this.controls.update();
    this.draw();
  }

  resetView() {
    if (!this.camera || !this.controls) return;

    this.camera.position.set(0, 0, CAMERA_DISTANCE);
    this.camera.zoom = 1;
    this.controls.target.set(0, 0, 0);
    this.camera.updateProjectionMatrix();
    this.controls.update();
    this.draw();
  }

  async exportToPNG() {
    if (!this.packet) throw new Error("Renderer is not ready");

    if (this.renderMode === "svg") {
      const svg = await this.buildSVGString();
      return rasterizeSvgToPng(
        svg,
        this.packet.metadata.imageWidth,
        this.packet.metadata.imageHeight,
      );
    }

    if (!this.renderer) {
      throw new Error("Renderer is not ready");
    }

    this.draw();
    return (this.renderer.domElement as HTMLCanvasElement).toDataURL("image/png");
  }

  async buildSVGString() {
    if (!this.packet) {
      throw new Error("No map data to export");
    }

    const cacheKey = this.makeSvgCacheKey();
    if (this.svgMarkup && this.svgMarkupCacheKey && this.svgMarkupCacheKey === cacheKey) {
      return this.svgMarkup;
    }

    return this.buildMapSvgMarkup();
  }

  primeSvgMarkup() {
    if (!this.packet || !getSvgMapJson(this.packet)) {
      return;
    }

    void this.buildMapSvgMarkup().catch(() => undefined);
  }

  private async detectAvailableModes() {
    const modes: RenderBackend[] = [];

    if (typeof navigator !== "undefined" && "gpu" in navigator) {
      try {
        const adapter = await navigator.gpu.requestAdapter();
        if (adapter) {
          modes.push("webgpu");
        }
      } catch {
        // Fall back to SVG if adapter acquisition fails or is blocked.
      }
    }

    modes.push("svg");
    this.availableModes = modes;
  }

  private async initializeMode(mode: RenderBackend, keepPreviousFrameVisible = false) {
    if (!this.runtime) {
      throw new Error("Three runtime has not been initialized");
    }

    const width = Math.max(1, this.stage.clientWidth || this.packet?.metadata.imageWidth || 1);
    const height = Math.max(1, this.stage.clientHeight || this.packet?.metadata.imageHeight || 1);

    this.scene = new this.runtime.THREE.Scene();
    this.scene.background = new this.runtime.THREE.Color(
      mode === "svg" ? PAPER_BACKGROUND : WEBGPU_BACKGROUND,
    );
    this.camera = new this.runtime.THREE.OrthographicCamera(
      -width / 2,
      width / 2,
      height / 2,
      -height / 2,
      1,
      4000,
    );
    this.camera.position.set(0, 0, CAMERA_DISTANCE);
    this.camera.lookAt(0, 0, 0);

    if (mode === "svg") {
      this.renderer = createNativeSvgRenderer();
      this.renderer.setClearColor(PAPER_BACKGROUND);
      this.renderer.domElement.classList.add("three-map-stage", "pointer-events-auto");
      if (keepPreviousFrameVisible) {
        this.renderer.domElement.style.opacity = "0";
        this.renderer.domElement.style.pointerEvents = "none";
      }
      this.stage.appendChild(this.renderer.domElement);
    } else {
      this.renderer = new this.runtime.WebGPURenderer({
        antialias: true,
        alpha: false,
      });

      if ("init" in this.renderer && typeof this.renderer.init === "function") {
        await this.renderer.init();
      }

      this.renderer.setClearColor(WEBGPU_BACKGROUND);
      this.renderer.outputColorSpace = this.runtime.THREE.SRGBColorSpace;
      this.renderer.toneMapping = this.runtime.THREE.ACESFilmicToneMapping;
      this.renderer.toneMappingExposure = 1.08;
      this.renderer.domElement.classList.add("three-map-stage", "pointer-events-auto");
      if (keepPreviousFrameVisible) {
        this.renderer.domElement.style.opacity = "0";
        this.renderer.domElement.style.pointerEvents = "none";
      }
      this.stage.appendChild(this.renderer.domElement);
    }

    this.controls = new this.runtime.MapControls(this.camera, this.renderer.domElement);
    this.controls.enableRotate = false;
    this.controls.screenSpacePanning = true;
    this.controls.enableDamping = false;
    this.controls.zoomSpeed = 1.05;
    this.controls.addEventListener("change", () => {
      this.draw();
    });

    this.renderMode = mode;
    this.resize();
    if (this.packet) {
      await this.rebuildScene();
      this.resize();
    }

    if (
      keepPreviousFrameVisible &&
      this.renderer?.domElement &&
      (this.renderer.domElement instanceof HTMLElement ||
        this.renderer.domElement instanceof SVGElement)
    ) {
      this.renderer.domElement.style.opacity = "1";
      this.renderer.domElement.style.pointerEvents = "";
    }
  }

  private snapshotState(): RendererStateSnapshot | null {
    if (
      !this.renderer &&
      !this.controls &&
      !this.scene &&
      !this.camera &&
      this.managedRoots.length === 0
    ) {
      return null;
    }

    return {
      renderer: this.renderer,
      controls: this.controls,
      scene: this.scene,
      camera: this.camera,
      renderMode: this.renderMode,
      layerRoots: this.layerRoots,
      managedRoots: this.managedRoots,
      svgMarkup: this.svgMarkup,
      svgMarkupCacheKey: this.svgMarkupCacheKey,
      svgRenderDirty: this.svgRenderDirty,
      nativeSvgViewport: this.nativeSvgViewport,
      lineMaterials: this.lineMaterials,
    };
  }

  private clearStateReferences() {
    this.controls = null;
    this.renderer = null;
    this.scene = null;
    this.camera = null;
    this.renderMode = null;
    this.layerRoots = {};
    this.managedRoots = [];
    this.svgMarkup = null;
    this.svgMarkupCacheKey = null;
    this.svgRenderDirty = true;
    this.nativeSvgViewport = null;
    this.lineMaterials = [];
  }

  private restoreSnapshot(snapshot: RendererStateSnapshot) {
    this.renderer = snapshot.renderer;
    this.controls = snapshot.controls;
    this.scene = snapshot.scene;
    this.camera = snapshot.camera;
    this.renderMode = snapshot.renderMode;
    this.layerRoots = snapshot.layerRoots;
    this.managedRoots = snapshot.managedRoots;
    this.svgMarkup = snapshot.svgMarkup;
    this.svgMarkupCacheKey = snapshot.svgMarkupCacheKey;
    this.svgRenderDirty = snapshot.svgRenderDirty;
    this.nativeSvgViewport =
      snapshot.nativeSvgViewport ??
      ((snapshot.renderer?.domElement as Element | undefined)?.querySelector?.(
        '[data-svg-viewport="true"]',
      ) as SVGGElement | null);
    this.lineMaterials = snapshot.lineMaterials;
  }

  private disposeSnapshot(snapshot: RendererStateSnapshot) {
    snapshot.controls?.dispose();

    for (const root of snapshot.managedRoots) {
      disposeObject(root);
    }

    if (snapshot.renderer?.dispose) {
      snapshot.renderer.dispose();
    }

    const domElement = snapshot.renderer?.domElement as Element | undefined;
    if (domElement?.parentElement === this.stage) {
      domElement.remove();
    }
  }

  private async rebuildScene() {
    if (!this.scene || !this.packet || !this.runtime || !this.renderMode) return;
    const sceneStart = performance.now();

    for (const root of this.managedRoots) {
      this.scene.remove(root);
      disposeObject(root);
    }
    this.managedRoots = [];
    this.layerRoots = {};
    this.lineMaterials = [];
    this.svgRenderDirty = true;
    this.nativeSvgViewport = null;
    const disposeEnd = performance.now();

    const { scene, layerRoots, roots, lineMaterials } = this.renderMode === "svg"
      ? await this.createSceneGraphAsync(this.renderMode)
      : this.createSceneGraphSync();
    const sceneEnd = performance.now();
    this.scene = scene;
    this.layerRoots = layerRoots;
    this.managedRoots = roots;
    this.lineMaterials = lineMaterials;

    // Release heavy binary data from the packet now that the Three.js scene
    // owns all geometry and textures.  Keeps metadata + SVG JSON for export.
    if (this.packet) {
      const empty32 = new Float32Array(0);
      const empty8 = new Uint8Array(0);
      const emptyU32 = new Uint32Array(0);
      this.packet.terrain = { positions: empty32, normals: empty32, uvs: empty32, indices: emptyU32 };
      this.packet.textures = { height: empty8, landMask: empty8, flux: empty8 };
      this.packet.layers = {
        slopeSegments: empty32,
        river: { positions: empty32, offsets: emptyU32 },
        contour: { positions: empty32, offsets: emptyU32 },
        border: { positions: empty32, offsets: emptyU32 },
      };
      this.packet.markers = { city: empty32, town: empty32 };
      this.packet.labels = { bytes: empty8, offsets: emptyU32, anchors: empty32, sizes: empty32, items: [] };
      this.packet.landPolygonPositions = empty32;
      this.packet.landPolygonOffsets = emptyU32;
    }

    this.applyLayerVisibility();

    // --- Optimization A: compileAsync pre-compilation ---
    // Pre-compile all shader pipelines before first draw to avoid blocking render.
    if (
      this.renderMode !== "svg" &&
      this.renderer &&
      this.camera &&
      typeof this.renderer.compileAsync === "function"
    ) {
      // --- Optimization C+E: Progressive rendering ---
      // Phase 1: Hide overlay layers, compile & draw terrain only first.
      const overlayKeys: (keyof typeof layerRoots)[] = [
        "slope", "river", "contour", "border", "city", "town", "label",
      ];
      for (const key of overlayKeys) {
        const root = layerRoots[key];
        if (root) root.visible = false;
      }

      const compileStart = performance.now();
      await this.renderer.compileAsync(this.scene, this.camera);
      const phase1Ms = Math.round(performance.now() - compileStart);

      // Draw terrain immediately so user sees the map surface
      this.draw();

      // Phase 2: Show overlays, compile their shaders, then redraw
      for (const key of overlayKeys) {
        const root = layerRoots[key];
        if (root) root.visible = true;
      }
      this.applyLayerVisibility();

      // Yield to main thread so Phase 1 frame paints on screen
      await new Promise<void>((r) => requestAnimationFrame(() => r()));

      const phase2Start = performance.now();
      await this.renderer.compileAsync(this.scene, this.camera);
      const phase2Ms = Math.round(performance.now() - phase2Start);

      console.log(`[PERF] ShaderCompile Phase1=${phase1Ms}ms Phase2=${phase2Ms}ms`);
      (self as unknown as Record<string, unknown>).__shaderCompileMs = phase1Ms + phase2Ms;
    }

    this.draw();
    const drawEnd = performance.now();
    const disposeMs = Math.round(disposeEnd - sceneStart);
    const sceneBuildMs = Math.round(sceneEnd - disposeEnd);
    const drawMs = Math.round(drawEnd - sceneEnd);
    console.log(`[PERF] Dispose: ${disposeMs}ms | SceneBuild: ${sceneBuildMs}ms | Draw: ${drawMs}ms`);
    (self as unknown as Record<string, unknown>).__sceneTiming = { disposeMs, sceneBuildMs, drawMs };
  }

  private mountNativeSvgMarkup(svgMarkup: string) {
    const svgRoot = this.renderer?.domElement;
    if (!(svgRoot instanceof SVGSVGElement) || !this.packet) {
      throw new Error("Native SVG renderer is not ready");
    }

    const sourceRoot = svgMarkupToRoot(svgMarkup);
    const width = this.packet.metadata.imageWidth;
    const height = this.packet.metadata.imageHeight;
    const viewport = document.createElementNS("http://www.w3.org/2000/svg", "g");
    viewport.setAttribute("data-svg-viewport", "true");

    svgRoot.replaceChildren();
    svgRoot.setAttribute("viewBox", sourceRoot.getAttribute("viewBox") ?? `0 0 ${width} ${height}`);
    svgRoot.setAttribute(
      "preserveAspectRatio",
      sourceRoot.getAttribute("preserveAspectRatio") ?? "xMidYMid meet",
    );
    svgRoot.setAttribute("overflow", sourceRoot.getAttribute("overflow") ?? "hidden");
    svgRoot.setAttribute("width", `${width}`);
    svgRoot.setAttribute("height", `${height}`);

    for (const attributeName of ["role", "aria-label"]) {
      const value = sourceRoot.getAttribute(attributeName);
      if (value) {
        svgRoot.setAttribute(attributeName, value);
      } else {
        svgRoot.removeAttribute(attributeName);
      }
    }

    for (const child of Array.from(sourceRoot.childNodes)) {
      const imported = document.importNode(child, true);
      if (
        imported instanceof SVGElement &&
        ["defs", "style", "title", "desc", "metadata"].includes(imported.tagName)
      ) {
        svgRoot.appendChild(imported);
        continue;
      }
      viewport.appendChild(imported);
    }

    svgRoot.appendChild(viewport);
    this.nativeSvgViewport = viewport;
    this.svgRenderDirty = false;
    this.applySvgViewportTransform();
  }

  private async createSceneGraphAsync(mode: RenderBackend): Promise<{
    scene: import("three").Scene;
    layerRoots: Partial<Record<keyof MapLayers, import("three").Object3D>>;
    roots: import("three").Object3D[];
    lineMaterials: Array<{ resolution?: import("three").Vector2; linewidth?: number }>;
  }> {
    if (!this.runtime || !this.packet || !this.camera) {
      throw new Error("Renderer is not ready");
    }

    const scene = new this.runtime.THREE.Scene();
    scene.background = new this.runtime.THREE.Color(
      mode === "svg" ? PAPER_BACKGROUND : WEBGPU_BACKGROUND,
    );
    const roots: import("three").Object3D[] = [];
    const layerRoots: Partial<Record<keyof MapLayers, import("three").Object3D>> = {};
    const lineMaterials: Array<{ resolution?: import("three").Vector2; linewidth?: number }> = [];

    if (mode === "svg") {
      this.mountNativeSvgMarkup(await this.buildMapSvgMarkup());
      return { scene, layerRoots, roots, lineMaterials };
    }

    const useTextureFallback = this.packet.landPolygonOffsets.length <= 1;

    if (useTextureFallback && getSvgMapJson(this.packet)) {
      const fallbackTextureSize = this.getPreviewTextureDimensions();
      const texture = await svgMarkupToTexture(
        this.runtime,
        await this.buildMapSvgMarkup(),
        fallbackTextureSize.width,
        fallbackTextureSize.height,
      );

      const plane = new this.runtime.THREE.Mesh(
        new this.runtime.THREE.PlaneGeometry(
          this.packet.metadata.imageWidth,
          this.packet.metadata.imageHeight,
        ),
        new this.runtime.THREE.MeshBasicMaterial({
          map: texture,
          toneMapped: false,
        }),
      );
      plane.renderOrder = 1;
      scene.add(plane);
      roots.push(plane);

      return { scene, layerRoots, roots, lineMaterials };
    }

    // Non-fallback WebGPU path: delegate to synchronous builder
    return this.createSceneGraphSync();
  }

  /** Synchronous scene graph builder for the WebGPU direct-render path. */
  private createSceneGraphSync() {
    if (!this.runtime || !this.packet || !this.camera) {
      throw new Error("Renderer is not ready");
    }

    const scene = new this.runtime.THREE.Scene();
    scene.background = new this.runtime.THREE.Color(WEBGPU_BACKGROUND);
    const roots: import("three").Object3D[] = [];
    const layerRoots: Partial<Record<keyof MapLayers, import("three").Object3D>> = {};
    const lineMaterials: Array<{ resolution?: import("three").Vector2; linewidth?: number }> = [];

    const backgroundPlane = new this.runtime.THREE.Mesh(
      new this.runtime.THREE.PlaneGeometry(
        this.packet.metadata.imageWidth,
        this.packet.metadata.imageHeight,
      ),
      new this.runtime.THREE.MeshBasicMaterial({ color: WEBGPU_SURFACE }),
    );
    backgroundPlane.position.set(0, 0, -(this.packet.metadata.elevationScale + 32));
    // backgroundPlane is added to scene via terrainBundle below

    const lightRig = buildTerrainLights(this.runtime, this.packet);
    scene.add(lightRig);
    roots.push(lightRig);

    const terrainGeometry = buildTerrainGeometry(this.runtime, this.packet);
    if (!terrainGeometry) {
      // Packet binary data may have been released after a previous build.
      // Return a minimal scene so callers don't crash.
      return { scene, layerRoots, roots, lineMaterials };
    }
    const surfaceTextures = buildSurfaceTextures(this.runtime, this.packet);

    configureTextureForRenderer(surfaceTextures.terrainAlbedo, this.renderer);
    configureTextureForRenderer(surfaceTextures.height, this.renderer);
    configureTextureForRenderer(surfaceTextures.roughness, this.renderer);
    configureTextureForRenderer(surfaceTextures.ao, this.renderer);
    configureTextureForRenderer(surfaceTextures.waterColor, this.renderer);
    configureTextureForRenderer(surfaceTextures.waterAlpha, this.renderer);
    configureTextureForRenderer(surfaceTextures.coastGlow, this.renderer);

    const terrainMesh = new this.runtime.THREE.Mesh(
      terrainGeometry,
      this.createTerrainMaterial(
        terrainGeometry,
        surfaceTextures.terrainAlbedo,
        surfaceTextures.height,
        surfaceTextures.roughness,
        surfaceTextures.ao,
      ),
    );
    terrainMesh.renderOrder = 1;

    scene.add(backgroundPlane);
    scene.add(terrainMesh);
    roots.push(backgroundPlane);
    roots.push(terrainMesh);

    const waterMesh = buildOverlayPlane(
      this.runtime,
      this.packet.metadata.imageWidth,
      this.packet.metadata.imageHeight,
      0.45,
      buildWaterMaterial(
        this.runtime,
        surfaceTextures.waterColor,
        surfaceTextures.waterAlpha,
        surfaceTextures.height,
        surfaceTextures.coastGlow,
      ),
    );
    waterMesh.renderOrder = 2;
    scene.add(waterMesh);
    roots.push(waterMesh);

    const coastGlowMesh = buildOverlayPlane(
      this.runtime,
      this.packet.metadata.imageWidth,
      this.packet.metadata.imageHeight,
      0.65,
      buildCoastGlowMaterial(this.runtime, surfaceTextures.coastGlow),
    );
    coastGlowMesh.renderOrder = 3;
    scene.add(coastGlowMesh);
    roots.push(coastGlowMesh);

    const slopeRoot = this.createLineGroup(
      this.packet.layers.slopeSegments,
      {
        color: SLOPE_COLOR,
        opacity: 0.18,
        linewidth: 0.72 * this.packet.metadata.drawScale,
        dashed: false,
      },
      lineMaterials,
    );
    if (slopeRoot) {
      setRenderOrder(slopeRoot, 10);
      layerRoots.slope = slopeRoot;
      scene.add(slopeRoot);
      roots.push(slopeRoot);
    }

    const riverRoot = this.createPathGroup(
      this.packet.layers.river,
      [
        {
          color: RIVER_GLOW,
          opacity: 0.28,
          linewidth: 4.4 * this.packet.metadata.drawScale,
          dashed: false,
        },
        {
          color: RIVER_COLOR,
          opacity: 0.95,
          linewidth: 2.5 * this.packet.metadata.drawScale,
          dashed: false,
        },
      ],
      lineMaterials,
    );
    if (riverRoot) {
      setRenderOrder(riverRoot, 20);
      layerRoots.river = riverRoot;
      scene.add(riverRoot);
      roots.push(riverRoot);
    }

    const contourRoot = this.createPathGroup(
      this.packet.layers.contour,
      [
        {
          color: COAST_GLOW_COLOR,
          opacity: 0.18,
          linewidth: 3.8 * this.packet.metadata.drawScale,
          dashed: false,
        },
        {
          color: COAST_COLOR,
          opacity: 0.7,
          linewidth: 1.1 * this.packet.metadata.drawScale,
          dashed: false,
        },
      ],
      lineMaterials,
    );
    if (contourRoot) {
      setRenderOrder(contourRoot, 30);
      layerRoots.contour = contourRoot;
      scene.add(contourRoot);
      roots.push(contourRoot);
    }

    const borderRoot = this.createPathGroup(
      this.packet.layers.border,
      [
        {
          color: BORDER_UNDER_COLOR,
          opacity: 1,
          linewidth: 6 * this.packet.metadata.drawScale,
          dashed: false,
        },
        {
          color: BORDER_COLOR,
          opacity: 0.92,
          linewidth: 3.1 * this.packet.metadata.drawScale,
          dashed: true,
        },
      ],
      lineMaterials,
    );
    if (borderRoot) {
      setRenderOrder(borderRoot, 40);
      layerRoots.border = borderRoot;
      scene.add(borderRoot);
      roots.push(borderRoot);
    }

    const cityRoot = this.createInstancedCityGroup();
    setLayeredRenderOrder(cityRoot, 50);
    layerRoots.city = cityRoot;
    scene.add(cityRoot);
    roots.push(cityRoot);

    const townRoot = this.createInstancedTownGroup();
    setLayeredRenderOrder(townRoot, 50);
    layerRoots.town = townRoot;
    scene.add(townRoot);
    roots.push(townRoot);

    const labelRoot = this.createLabelGroup();
    setRenderOrder(labelRoot, 60);
    layerRoots.label = labelRoot;
    scene.add(labelRoot);
    roots.push(labelRoot);

    return { scene, layerRoots, roots, lineMaterials };
  }

  private createTerrainMaterial(
    terrainGeometry: import("three").BufferGeometry,
    albedoTexture: import("three").Texture,
    heightTexture: import("three").Texture,
    roughnessTexture: import("three").Texture,
    aoTexture: import("three").Texture,
  ) {
    const { THREE, MeshStandardNodeMaterial, TSL } = this.runtime!;
    const uvAttribute = terrainGeometry.getAttribute("uv");
    if (uvAttribute && !terrainGeometry.getAttribute("uv2")) {
      terrainGeometry.setAttribute(
        "uv2",
        new THREE.BufferAttribute(new Float32Array(uvAttribute.array as ArrayLike<number>), 2),
      );
    }

    const {
      texture,
      uv,
      normalLocal,
      float,
      vec3,
      vec4,
      smoothstep,
      mix,
      normalize,
      abs,
      dot,
      clamp,
      sin,
      cos,
    } = TSL;

    const uv0 = uv();
    const albedo = texture(albedoTexture, uv0);
    const height = texture(heightTexture, uv0);
    const roughnessMap = texture(roughnessTexture, uv0);
    const ao = texture(aoTexture, uv0);
    const surfaceNormal = normalize(normalLocal);
    const upVector = vec3(0, 0, 1);
    const slopeValue = float(1).sub(abs(dot(surfaceNormal, upVector)));
    const beachColor = vec3(0.77, 0.72, 0.59);
    const grassColor = vec3(0.42, 0.55, 0.26);
    const forestColor = vec3(0.24, 0.36, 0.18);
    const rockColor = vec3(0.48, 0.46, 0.41);
    const snowColor = vec3(0.95, 0.96, 0.94);
    const beachThreshold = float(0.12);
    const grassThreshold = float(0.28);
    const forestThreshold = float(0.48);
    const rockThreshold = float(0.72);
    const snowThreshold = float(0.85);
    const grassWeight = smoothstep(beachThreshold, grassThreshold, height.r);
    const forestWeight = smoothstep(grassThreshold, forestThreshold, height.r);
    const rockWeight = smoothstep(forestThreshold, rockThreshold, height.r);
    const snowWeight = smoothstep(snowThreshold, float(1), height.r);
    const terrainColor1 = mix(beachColor, grassColor, grassWeight);
    const terrainColor2 = mix(terrainColor1, forestColor, forestWeight.mul(float(0.8)));
    const terrainColor3 = mix(terrainColor2, rockColor, rockWeight.mul(float(0.6)));
    const terrainColor = mix(terrainColor3, snowColor, snowWeight);
    const slopeRockMixWeight = smoothstep(float(0.35), float(0.65), slopeValue);
    const slopeColor = mix(terrainColor, rockColor, slopeRockMixWeight.mul(float(0.7)));
    // Per-pixel sin/cos detail computed on the GPU for full precision
    const macroDetail = sin(uv0.x.mul(float(30)).add(height.r.mul(float(6))))
      .mul(cos(uv0.y.mul(float(26)).sub(height.r.mul(float(5)))))
      .mul(float(0.5))
      .add(float(0.5));
    const microDetail = sin(uv0.x.mul(float(94)).add(uv0.y.mul(float(37))))
      .mul(cos(uv0.y.mul(float(102)).sub(uv0.x.mul(float(44)))))
      .mul(float(0.5))
      .add(float(0.5));
    const valleyTint = vec3(0.34, 0.45, 0.27);
    const ridgeTint = vec3(0.76, 0.72, 0.67);

    let detailedAlbedo = mix(
      albedo.rgb,
      albedo.rgb.mul(vec3(0.94, 1.04, 0.97)),
      macroDetail.mul(float(0.18)),
    );
    detailedAlbedo = mix(
      detailedAlbedo,
      valleyTint,
      grassWeight.mul(float(0.12)).mul(float(1).sub(microDetail)),
    );
    detailedAlbedo = mix(
      detailedAlbedo,
      ridgeTint,
      rockWeight.add(slopeRockMixWeight).mul(microDetail).mul(float(0.08)),
    );
    const finalColor = detailedAlbedo.mul(slopeColor);

    const dynamicRoughness = clamp(
      roughnessMap.r
        .add(slopeValue.mul(float(0.12)))
        .sub(height.r.mul(float(0.08)))
        .add(microDetail.mul(float(0.05))),
      float(0.3),
      float(0.95),
    );
    const dynamicAO = clamp(
      ao.r
        .mul(float(1).sub(slopeValue.mul(float(0.18))))
        .sub(float(1).sub(macroDetail).mul(float(0.08))),
      float(0.32),
      float(1),
    );

    const material = new MeshStandardNodeMaterial();
    material.colorNode = vec4(finalColor, float(1)) as unknown as typeof material.colorNode;
    material.roughnessNode = dynamicRoughness as unknown as typeof material.roughnessNode;
    material.aoNode = dynamicAO as unknown as typeof material.aoNode;
    material.metalnessNode = float(0) as unknown as typeof material.metalnessNode;
    material.lights = true;
    material.side = THREE.DoubleSide;

    return material;
  }

  private getOrCreateLineMaterial(
    style: { color: number; opacity: number; linewidth: number; dashed: boolean },
    worldUnits: boolean,
  ) {
    const key = `${style.color}-${style.linewidth}-${style.opacity}-${style.dashed}-${worldUnits}`;
    let material = this.lineMaterialCache.get(key);
    if (!material) {
      material = new this.runtime!.Line2NodeMaterial({
        color: style.color,
        linewidth: style.linewidth,
        worldUnits,
        opacity: style.opacity,
        dashed: style.dashed,
        dashSize: 4 * this.packet!.metadata.drawScale,
        gapSize: 5 * this.packet!.metadata.drawScale,
      });
      configureOverlayMaterial(material);
      this.lineMaterialCache.set(key, material);
    }
    return material;
  }

  private createLineGroup(
    packetPositions: Float32Array,
    style: { color: number; opacity: number; linewidth: number; dashed: boolean },
    lineMaterials: Array<{ resolution?: import("three").Vector2; linewidth?: number }>,
  ) {
    if (!this.runtime || packetPositions.length === 0) return null;

    const group = new this.runtime.THREE.Group();
    const positions = packetToThreeTriplets(packetPositions);

    const geometry = new this.runtime.LineSegmentsGeometry();
    geometry.setPositions(positions as unknown as number[]);

    const material = this.getOrCreateLineMaterial(style, false);
    const line = new this.runtime.WebGPULineSegments2(geometry, material);
    if ("computeLineDistances" in line && typeof line.computeLineDistances === "function") {
      line.computeLineDistances();
    }
    group.add(line);
    lineMaterials.push(material);
    return group;
  }

  private createPathGroup(
    layer: PathLayerPacket,
    styles: Array<{ color: number; opacity: number; linewidth: number; dashed: boolean }>,
    lineMaterials: Array<{ resolution?: import("three").Vector2; linewidth?: number }>,
  ) {
    const group = new this.runtime!.THREE.Group();
    let hasLines = false;

    // First pass: count total segment floats to allocate one buffer
    let totalSegmentFloats = 0;
    for (let pathIndex = 0; pathIndex < layer.offsets.length - 1; pathIndex += 1) {
      const start = layer.offsets[pathIndex] * 3;
      const end = layer.offsets[pathIndex + 1] * 3;
      const len = end - start;
      if (len < 6) continue;
      const pointCount = len / 3;
      totalSegmentFloats += (pointCount - 1) * 6;
    }

    if (totalSegmentFloats === 0) return null;

    // Second pass: fill merged buffer with Y/Z swap inlined (no intermediate arrays)
    const mergedPositions = new Float32Array(totalSegmentFloats);
    let offset = 0;
    const src = layer.positions;
    for (let pathIndex = 0; pathIndex < layer.offsets.length - 1; pathIndex += 1) {
      const start = layer.offsets[pathIndex] * 3;
      const end = layer.offsets[pathIndex + 1] * 3;
      const len = end - start;
      if (len < 6) continue;
      const pointCount = len / 3;
      for (let i = 0; i < pointCount - 1; i++) {
        const si = start + i * 3;
        const ni = start + (i + 1) * 3;
        // Y/Z swap: packet [x, y, z] → Three.js [x, z, y]
        mergedPositions[offset]     = src[si];
        mergedPositions[offset + 1] = src[si + 2];
        mergedPositions[offset + 2] = src[si + 1];
        mergedPositions[offset + 3] = src[ni];
        mergedPositions[offset + 4] = src[ni + 2];
        mergedPositions[offset + 5] = src[ni + 1];
        offset += 6;
      }
    }

    // Create one batched draw call per style (reuse same position data)
    for (const style of styles) {
      const geometry = new this.runtime!.LineSegmentsGeometry();
      geometry.setPositions(mergedPositions as unknown as number[]);

      const material = this.getOrCreateLineMaterial(style, true);
      const line = new this.runtime!.WebGPULineSegments2(geometry, material);
      if ("computeLineDistances" in line && typeof line.computeLineDistances === "function") {
        line.computeLineDistances();
      }
      group.add(line);
      lineMaterials.push(material);
      hasLines = true;
    }

    return hasLines ? group : null;
  }

  private createMarkerMaterial(fillColor: number) {
    const { MeshBasicNodeMaterial, TSL } = this.runtime!;
    const { color, float, vec4 } = TSL;

    const material = new MeshBasicNodeMaterial();
    material.colorNode = vec4(color(fillColor).rgb, float(1));
    material.toneMapped = false;
    configureOverlayMaterial(material);
    return material;
  }

  private createInstancedMarkerLayer(
    positions: Float32Array,
    geometry: import("three").BufferGeometry,
    material: import("three").Material,
    zOffset: number,
  ) {
    const { THREE } = this.runtime!;
    const count = positions.length / 3;
    const mesh = new THREE.InstancedMesh(geometry, material, count);
    mesh.instanceMatrix.setUsage(THREE.StaticDrawUsage);

    const dummy = new THREE.Object3D();
    for (let index = 0; index < positions.length; index += 3) {
      dummy.position.set(positions[index], positions[index + 1], positions[index + 2] + zOffset);
      dummy.updateMatrix();
      mesh.setMatrixAt(index / 3, dummy.matrix);
    }

    mesh.instanceMatrix.needsUpdate = true;
    mesh.computeBoundingBox();
    mesh.computeBoundingSphere();
    return mesh;
  }

  private createInstancedCityGroup() {
    const group = new this.runtime!.THREE.Group();
    const { THREE } = this.runtime!;
    const outerRadius = 10 * this.packet!.metadata.drawScale;
    const innerRadius = 5 * this.packet!.metadata.drawScale;
    const outlineWidth = Math.max(1 * this.packet!.metadata.drawScale, outerRadius * 0.08);
    const fillRadius = Math.max(innerRadius + outlineWidth * 0.35, outerRadius - outlineWidth);
    const positions = packetToThreeTriplets(this.packet!.markers.city);

    const outline = this.createInstancedMarkerLayer(
      positions,
      new THREE.RingGeometry(fillRadius, outerRadius, 40),
      this.createMarkerMaterial(CITY_OUTER_COLOR),
      MARKER_BASE_Z_OFFSET,
    );
    const fill = this.createInstancedMarkerLayer(
      positions,
      new THREE.CircleGeometry(fillRadius, 40),
      this.createMarkerMaterial(CITY_INNER_COLOR),
      MARKER_BASE_Z_OFFSET + MARKER_LAYER_Z_STEP,
    );
    const core = this.createInstancedMarkerLayer(
      positions,
      new THREE.CircleGeometry(innerRadius, 32),
      this.createMarkerMaterial(CITY_OUTER_COLOR),
      MARKER_BASE_Z_OFFSET + MARKER_LAYER_Z_STEP * 2,
    );

    group.add(outline);
    group.add(fill);
    group.add(core);
    return group;
  }

  private createInstancedTownGroup() {
    const group = new this.runtime!.THREE.Group();
    const { THREE } = this.runtime!;
    const radius = 5 * this.packet!.metadata.drawScale;
    const outlineWidth = Math.max(1 * this.packet!.metadata.drawScale, radius * 0.12);
    const fillRadius = Math.max(radius - outlineWidth, radius * 0.58);
    const positions = packetToThreeTriplets(this.packet!.markers.town);

    const outline = this.createInstancedMarkerLayer(
      positions,
      new THREE.RingGeometry(fillRadius, radius, 36),
      this.createMarkerMaterial(CITY_OUTER_COLOR),
      MARKER_BASE_Z_OFFSET,
    );
    const fill = this.createInstancedMarkerLayer(
      positions,
      new THREE.CircleGeometry(fillRadius, 36),
      this.createMarkerMaterial(TOWN_COLOR),
      MARKER_BASE_Z_OFFSET + MARKER_LAYER_Z_STEP,
    );

    group.add(outline);
    group.add(fill);
    return group;
  }

  private createLabelGroup() {
    const group = new this.runtime!.THREE.Group();
    const anchors = packetToThreeTriplets(this.packet!.labels.anchors);
    const items = this.packet!.labels.items;
    const { THREE } = this.runtime!;

    const pixelRatio = Math.max(1, Math.min(window.devicePixelRatio || 1, 2));

    // Phase 1: Measure all labels to determine atlas layout
    interface LabelEntry {
      index: number;
      text: string;
      fontface: string;
      fontsize: number;
      logicalWidth: number;
      logicalHeight: number;
      drawX: number;
      drawY: number;
      font: string;
      haloWidth: number;
      offsetX: number;
      offsetY: number;
    }
    const entries: LabelEntry[] = [];

    const measureCanvas = document.createElement("canvas");
    measureCanvas.width = 1;
    measureCanvas.height = 1;
    const measureCtx = measureCanvas.getContext("2d")!;

    for (let index = 0; index < items.length; index += 1) {
      const item = items[index];
      if (!item?.text) continue;

      const fontSize = Math.max(1, item.fontsize);
      const haloWidth = Math.max(3, Math.round(fontSize * 0.12));
      const padding = Math.max(4, Math.round(fontSize * 0.24));
      const font = `${fontSize}px ${resolveLabelFontFamily(item.fontface)}`;

      measureCtx.font = font;
      measureCtx.textAlign = "left";
      measureCtx.textBaseline = "alphabetic";

      const metrics = measureCtx.measureText(item.text);
      const left = metrics.actualBoundingBoxLeft || 0;
      const right = metrics.actualBoundingBoxRight || metrics.width || fontSize;
      const ascent = metrics.actualBoundingBoxAscent || fontSize * 0.78;
      const descent = metrics.actualBoundingBoxDescent || fontSize * 0.22;
      const logicalWidth = Math.max(1, Math.ceil(left + right + padding * 2 + haloWidth * 2));
      const logicalHeight = Math.max(1, Math.ceil(ascent + descent + padding * 2 + haloWidth * 2));
      const drawX = padding + haloWidth + left;
      const drawY = padding + haloWidth + ascent;

      entries.push({
        index,
        text: item.text,
        fontface: item.fontface,
        fontsize: item.fontsize,
        logicalWidth,
        logicalHeight,
        drawX,
        drawY,
        font,
        haloWidth,
        offsetX: logicalWidth * 0.5 - drawX,
        offsetY: drawY - logicalHeight * 0.5,
      });
    }

    if (entries.length === 0) return group;

    // Phase 2: Pack labels into atlas rows (simple shelf packing)
    const maxAtlasSize = 4096;
    let atlasWidth = 0;
    let atlasHeight = 0;
    const placements: Array<{ x: number; y: number }> = [];
    let rowX = 0;
    let rowY = 0;
    let rowHeight = 0;

    for (const entry of entries) {
      const pw = Math.ceil(entry.logicalWidth * pixelRatio);
      const ph = Math.ceil(entry.logicalHeight * pixelRatio);

      if (rowX + pw > maxAtlasSize) {
        // Move to next row
        rowY += rowHeight;
        rowX = 0;
        rowHeight = 0;
      }

      placements.push({ x: rowX, y: rowY });
      rowX += pw;
      rowHeight = Math.max(rowHeight, ph);
      atlasWidth = Math.max(atlasWidth, rowX);
    }
    atlasHeight = rowY + rowHeight;

    // Round up to power-of-two friendly sizes (not required but GPU-friendly)
    atlasWidth = Math.min(maxAtlasSize, atlasWidth);
    atlasHeight = Math.min(maxAtlasSize, atlasHeight);

    // Phase 3: Render all labels onto atlas canvas
    const atlasCanvas = document.createElement("canvas");
    atlasCanvas.width = atlasWidth;
    atlasCanvas.height = atlasHeight;
    const atlasCtx = atlasCanvas.getContext("2d")!;
    atlasCtx.clearRect(0, 0, atlasWidth, atlasHeight);

    for (let i = 0; i < entries.length; i++) {
      const entry = entries[i];
      const placement = placements[i];

      atlasCtx.save();
      atlasCtx.translate(placement.x, placement.y);
      atlasCtx.scale(pixelRatio, pixelRatio);

      atlasCtx.font = entry.font;
      atlasCtx.textAlign = "left";
      atlasCtx.textBaseline = "alphabetic";
      atlasCtx.lineJoin = "round";
      atlasCtx.miterLimit = 2;
      atlasCtx.strokeStyle = `#${LABEL_HALO_COLOR.toString(16).padStart(6, "0")}`;
      atlasCtx.fillStyle = `#${LABEL_COLOR.toString(16).padStart(6, "0")}`;
      atlasCtx.lineWidth = entry.haloWidth;
      atlasCtx.strokeText(entry.text, entry.drawX, entry.drawY);
      atlasCtx.fillText(entry.text, entry.drawX, entry.drawY);

      atlasCtx.restore();
    }

    // Phase 4: Create shared atlas texture
    const atlasTexture = new THREE.CanvasTexture(atlasCanvas);
    atlasTexture.colorSpace = THREE.SRGBColorSpace;
    atlasTexture.minFilter = THREE.LinearFilter;
    atlasTexture.magFilter = THREE.LinearFilter;
    atlasTexture.generateMipmaps = false;
    atlasTexture.needsUpdate = true;

    // Phase 5: Build merged geometry with per-label UVs
    const vertCount = entries.length * 4; // 4 vertices per quad
    const triCount = entries.length * 2;  // 2 triangles per quad
    const positions = new Float32Array(vertCount * 3);
    const uvs = new Float32Array(vertCount * 2);
    const indices = new Uint32Array(triCount * 3);

    for (let i = 0; i < entries.length; i++) {
      const entry = entries[i];
      const placement = placements[i];
      const anchorIndex = entry.index * 3;
      const cx = anchors[anchorIndex] + entry.offsetX;
      const cy = anchors[anchorIndex + 1] + entry.offsetY;
      const cz = anchors[anchorIndex + 2] + LABEL_Z_OFFSET;
      const hw = entry.logicalWidth * 0.5;
      const hh = entry.logicalHeight * 0.5;

      // Quad vertices (centered at label position)
      const vi = i * 4;
      // bottom-left
      positions[(vi) * 3] = cx - hw;
      positions[(vi) * 3 + 1] = cy - hh;
      positions[(vi) * 3 + 2] = cz;
      // bottom-right
      positions[(vi + 1) * 3] = cx + hw;
      positions[(vi + 1) * 3 + 1] = cy - hh;
      positions[(vi + 1) * 3 + 2] = cz;
      // top-right
      positions[(vi + 2) * 3] = cx + hw;
      positions[(vi + 2) * 3 + 1] = cy + hh;
      positions[(vi + 2) * 3 + 2] = cz;
      // top-left
      positions[(vi + 3) * 3] = cx - hw;
      positions[(vi + 3) * 3 + 1] = cy + hh;
      positions[(vi + 3) * 3 + 2] = cz;

      // UVs mapped to atlas region
      const pw = Math.ceil(entry.logicalWidth * pixelRatio);
      const ph = Math.ceil(entry.logicalHeight * pixelRatio);
      const u0 = placement.x / atlasWidth;
      const u1 = (placement.x + pw) / atlasWidth;
      const v0 = 1.0 - (placement.y + ph) / atlasHeight; // flip Y for texture
      const v1 = 1.0 - placement.y / atlasHeight;

      uvs[vi * 2] = u0;       uvs[vi * 2 + 1] = v0;
      uvs[(vi + 1) * 2] = u1; uvs[(vi + 1) * 2 + 1] = v0;
      uvs[(vi + 2) * 2] = u1; uvs[(vi + 2) * 2 + 1] = v1;
      uvs[(vi + 3) * 2] = u0; uvs[(vi + 3) * 2 + 1] = v1;

      // Indices
      const ti = i * 6;
      indices[ti] = vi;
      indices[ti + 1] = vi + 1;
      indices[ti + 2] = vi + 2;
      indices[ti + 3] = vi;
      indices[ti + 4] = vi + 2;
      indices[ti + 5] = vi + 3;
    }

    const geometry = new THREE.BufferGeometry();
    geometry.setAttribute("position", new THREE.BufferAttribute(positions, 3));
    geometry.setAttribute("uv", new THREE.BufferAttribute(uvs, 2));
    geometry.setIndex(new THREE.BufferAttribute(indices, 1));

    // Single material for all labels
    const { MeshBasicNodeMaterial, TSL } = this.runtime!;
    const { texture: tslTexture, uv: tslUv } = TSL;

    const material = new MeshBasicNodeMaterial();
    material.colorNode = tslTexture(atlasTexture, tslUv());
    material.transparent = true;
    material.depthTest = false;
    material.depthWrite = false;
    material.toneMapped = false;

    const mesh = new THREE.Mesh(geometry, material);
    mesh.renderOrder = 40;
    group.add(mesh);

    return group;
  }

  private applyLayerVisibility(
    roots: Partial<Record<keyof MapLayers, import("three").Object3D>> = this.layerRoots,
  ) {
    for (const [key, root] of Object.entries(roots) as Array<
      [keyof MapLayers, import("three").Object3D | undefined]
    >) {
      if (!root) continue;
      root.visible = this.layers[key];
    }
  }

  private draw() {
    if (!this.renderer || !this.scene || !this.camera) return;

    if (this.renderMode === "svg") {
      if (this.svgRenderDirty && this.svgMarkup) {
        this.mountNativeSvgMarkup(this.svgMarkup);
      }
      this.applySvgViewportTransform();
      return;
    }

    this.renderer.render(this.scene, this.camera);
  }

  private getPreviewTextureDimensions() {
    const pixelRatio = Math.max(1, Math.min(window.devicePixelRatio || 1, 2));
    const width = this.stage.clientWidth || this.packet?.metadata.imageWidth || 1;
    const height = this.stage.clientHeight || this.packet?.metadata.imageHeight || 1;
    return clampPreviewTextureSize(width * pixelRatio, height * pixelRatio);
  }

  private applySvgViewportTransform() {
    if (!this.camera || !this.packet) {
      return;
    }

    const viewport =
      this.nativeSvgViewport ??
      ((this.renderer?.domElement as Element | undefined)?.querySelector?.(
        '[data-svg-viewport="true"]',
      ) as SVGGElement | null);
    if (!(viewport instanceof SVGGElement)) {
      return;
    }

    const scale = this.camera.zoom;
    const tx =
      this.camera.right -
      (this.packet.metadata.imageWidth * 0.5 + this.camera.position.x) * scale;
    const ty =
      this.camera.top -
      (this.packet.metadata.imageHeight * 0.5 - this.camera.position.y) * scale;
    viewport.setAttribute("transform", `matrix(${scale} 0 0 ${scale} ${tx} ${ty})`);
  }

  private async buildMapSvgMarkup() {
    const svgJson = this.packet ? getSvgMapJson(this.packet) : undefined;
    if (!svgJson) {
      throw new Error("SVG mode requires exported map JSON data");
    }

    const cacheKey = this.makeSvgCacheKey();
    if (!cacheKey) {
      throw new Error("SVG mode requires exported map JSON data");
    }

    if (this.svgMarkup && this.svgMarkupCacheKey === cacheKey) {
      return this.svgMarkup;
    }

    if (this.svgMarkupPromise && this.svgMarkupPromiseKey === cacheKey) {
      return this.svgMarkupPromise;
    }

    const worker = this.ensureSvgWorker();
    const requestId = ++this.svgWorkerRequestId;

    this.svgMarkupPromiseKey = cacheKey;
    this.svgMarkupPromise = new Promise<string>((resolve, reject) => {
      this.svgWorkerPending.set(requestId, {
        resolve: (svgMarkup) => {
          if (this.svgMarkupPromiseKey === cacheKey) {
            this.svgMarkup = svgMarkup;
            this.svgMarkupCacheKey = cacheKey;
          }
          this.svgMarkupPromise = null;
          this.svgMarkupPromiseKey = null;
          resolve(svgMarkup);
        },
        reject: (error) => {
          this.svgMarkupPromise = null;
          this.svgMarkupPromiseKey = null;
          reject(error);
        },
      });

      const request: SvgBuildWorkerRequest = {
        type: "build-svg",
        requestId,
        mapJson: getSvgMapJson(this.packet!)!,
        layers: { ...this.layers },
      };
      worker.postMessage(request);
    });

    return this.svgMarkupPromise;
  }
}

async function rasterizeSvgToPng(svgString: string, width: number, height: number) {
  const blob = new Blob([svgString], { type: "image/svg+xml;charset=utf-8" });
  const url = URL.createObjectURL(blob);

  return new Promise<string>((resolve, reject) => {
    const image = new Image();
    image.onload = () => {
      const canvas = document.createElement("canvas");
      canvas.width = width;
      canvas.height = height;
      const context = canvas.getContext("2d");
      if (!context) {
        URL.revokeObjectURL(url);
        reject(new Error("Failed to create PNG export canvas"));
        return;
      }

      context.drawImage(image, 0, 0, width, height);
      URL.revokeObjectURL(url);
      resolve(canvas.toDataURL("image/png"));
    };
    image.onerror = () => {
      URL.revokeObjectURL(url);
      reject(new Error("Failed to rasterize SVG export"));
    };
    image.src = url;
  });
}
