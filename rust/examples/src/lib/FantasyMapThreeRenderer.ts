import type {
  MapLayers,
  MapScenePacket,
  PathLayerPacket,
  RenderBackend,
  RendererPreference,
  RendererRuntimeBackend,
} from "@/types/map";

const DEFAULT_LAYERS: MapLayers = {
  slope: true,
  river: true,
  contour: true,
  border: true,
  city: true,
  town: true,
  label: true,
};

const PAPER_BACKGROUND = 0xf7f1e3;
const WEBGPU_BACKGROUND = 0x0a2740;
const WEBGPU_SURFACE = 0x11283a;
const SLOPE_COLOR = 0x24301f;
const RIVER_COLOR = 0x5fc0df;
const RIVER_GLOW = 0x9ae6ff;
const COAST_COLOR = 0x193043;
const COAST_GLOW_COLOR = 0x8fdfff;
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
}> | null = null;

async function loadThreeRuntime() {
  if (!threeRuntimePromise) {
    threeRuntimePromise = Promise.all([
      import("three"),
      import("three/webgpu"),
      import("three/examples/jsm/controls/MapControls.js"),
      import("three/examples/jsm/lines/webgpu/Line2.js"),
      import("three/examples/jsm/lines/LineGeometry.js"),
      import("three/examples/jsm/lines/webgpu/LineSegments2.js"),
      import("three/examples/jsm/lines/LineSegmentsGeometry.js"),
    ]).then(
      ([
        THREE,
        webgpu,
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
          MapControls: controls.MapControls,
          WebGPULine2: webgpuLine2.Line2,
          LineGeometry: lineGeometry.LineGeometry,
          WebGPULineSegments2: webgpuLineSegments2.LineSegments2,
          LineSegmentsGeometry: lineSegmentsGeometry.LineSegmentsGeometry,
        };
      },
    );
  }

  return threeRuntimePromise;
}

function packetToThreeTriplets(source: Float32Array) {
  const output = new Float32Array(source.length);
  for (let index = 0; index < source.length; index += 3) {
    output[index] = source[index];
    output[index + 1] = source[index + 2];
    output[index + 2] = source[index + 1];
  }
  return output;
}

function pathPositionSlice(layer: PathLayerPacket, pathIndex: number) {
  const start = layer.offsets[pathIndex] * 3;
  const end = layer.offsets[pathIndex + 1] * 3;
  return layer.positions.subarray(start, end);
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

  const hemi = new runtime.THREE.HemisphereLight(0xe9f2ff, 0x8a7b5d, 1.3);
  const keyLight = new runtime.THREE.DirectionalLight(0xfff4da, 1.75);
  keyLight.position.set(-maxSpan * 0.42, maxSpan * 0.55, maxSpan * 1.15);

  const fillLight = new runtime.THREE.DirectionalLight(0xa1c8e5, 0.38);
  fillLight.position.set(maxSpan * 0.34, -maxSpan * 0.24, maxSpan * 0.72);

  group.add(hemi);
  group.add(keyLight);
  group.add(fillLight);
  return group;
}

function clamp01(value: number) {
  return Math.max(0, Math.min(1, value));
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

function dataTexture(
  runtime: ThreeRuntime,
  data: Uint8Array,
  width: number,
  height: number,
  srgb = false,
) {
  const texture = new runtime.THREE.DataTexture(data, width, height, runtime.THREE.RGBAFormat);
  texture.flipY = true;
  texture.wrapS = runtime.THREE.ClampToEdgeWrapping;
  texture.wrapT = runtime.THREE.ClampToEdgeWrapping;
  texture.minFilter = runtime.THREE.LinearFilter;
  texture.magFilter = runtime.THREE.LinearFilter;
  texture.generateMipmaps = false;
  if (srgb) {
    texture.colorSpace = runtime.THREE.SRGBColorSpace;
  }
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
  const width = packet.metadata.terrainWidth;
  const height = packet.metadata.terrainHeight;
  const sourceAlbedo = packet.textures.albedo;
  const sourceHeight = packet.textures.height;
  const sourceFlux = packet.textures.flux;
  const sourceMask = packet.textures.landMask;
  const terrainAlbedo = new Uint8Array(sourceAlbedo.length);
  const roughness = new Uint8Array(sourceAlbedo.length);
  const ao = new Uint8Array(sourceAlbedo.length);
  const waterColor = new Uint8Array(sourceAlbedo.length);
  const waterAlpha = new Uint8Array(sourceAlbedo.length);
  const coastGlow = new Uint8Array(sourceAlbedo.length);

  for (let y = 0; y < height; y += 1) {
    for (let x = 0; x < width; x += 1) {
      const offset = pixelOffset(width, x, y);
      const heightValue = (sourceHeight[offset] ?? 0) / 255;
      const fluxValue = (sourceFlux[offset] ?? 0) / 255;
      const isLand = (sourceMask[offset] ?? 0) > 127;
      const coast = coastalStrengthFromTexture(sourceMask, width, height, x, y);
      const left = sampleTextureChannel(sourceHeight, width, height, x - 1, y);
      const right = sampleTextureChannel(sourceHeight, width, height, x + 1, y);
      const up = sampleTextureChannel(sourceHeight, width, height, x, y - 1);
      const down = sampleTextureChannel(sourceHeight, width, height, x, y + 1);
      const relief = clamp01((Math.abs(right - left) + Math.abs(down - up)) * 3.4);

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
          [86, 110, 53],
          clamp01((0.58 - Math.abs(heightValue - 0.34)) * 0.58 + fluxValue * 0.14),
        );
        color = mixColor(
          color,
          [172, 152, 101],
          clamp01((0.18 - heightValue) * 3.1) * (1 - fluxValue * 0.7),
        );
        color = mixColor(
          color,
          [98, 96, 101],
          clamp01(relief * 0.68 + Math.max(0, heightValue - 0.58) * 0.6),
        );
        color = mixColor(
          color,
          [242, 244, 247],
          clamp01((heightValue - 0.78) * 3.5 + relief * 0.2),
        );
        color = mixColor(
          color,
          [201, 190, 145],
          clamp01(coast * 0.18 + Math.max(0, 0.1 - heightValue) * 2.3),
        );
      } else {
        color = mixColor(color, [18, 44, 68], 0.34);
      }

      const roughnessValue = isLand
        ? clamp01(0.96 - fluxValue * 0.22 - Math.max(0, heightValue - 0.76) * 0.16 + relief * 0.08)
        : 1;
      const aoValue = isLand
        ? clamp01(0.92 - relief * 0.42 + fluxValue * 0.06 + Math.max(0, heightValue - 0.72) * 0.04)
        : clamp01(0.96 - coast * 0.08);

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

      const waterDepth = clamp01(1 - heightValue);
      const shallowMix = clamp01(1 - waterDepth * 1.28);
      const waterBase = mixColor([9, 38, 69], [66, 158, 199], shallowMix * 0.8 + coast * 0.18);
      const waterTint = mixColor(waterBase, [138, 218, 242], coast * 0.26);
      const waterOpacity = isLand ? 0 : clamp01(0.84 - coast * 0.22 + shallowMix * 0.08);
      const glowOpacity = isLand ? 0 : clamp01(coast * 0.78 + shallowMix * 0.12);

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
  return new runtime.THREE.MeshBasicMaterial({
    alphaMap,
    color: COAST_GLOW_COLOR,
    depthTest: false,
    depthWrite: false,
    opacity: 0.5,
    transparent: true,
    blending: runtime.THREE.AdditiveBlending,
    toneMapped: false,
  });
}

function buildWaterMaterial(
  runtime: ThreeRuntime,
  colorMap: import("three").Texture,
  alphaMap: import("three").Texture,
) {
  return new runtime.THREE.MeshStandardMaterial({
    map: colorMap,
    alphaMap,
    color: 0xffffff,
    emissive: new runtime.THREE.Color(0x13344f),
    emissiveMap: colorMap,
    emissiveIntensity: 0.22,
    transparent: true,
    opacity: 0.94,
    depthWrite: false,
    roughness: 0.22,
    metalness: 0.02,
  });
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
    if (!this.packet?.mapJson) {
      return null;
    }

    return JSON.stringify({
      mapJsonHash: `${this.packet.mapJson.length}:${hashString(this.packet.mapJson)}`,
      layers: this.layers,
    });
  }

  private invalidateRenderCaches() {
    this.svgMarkup = null;
    this.svgMarkupCacheKey = null;
    this.svgMarkupPromise = null;
    this.svgMarkupPromiseKey = null;
    this.svgRenderDirty = true;
    this.modeDirty.svg = true;
    this.modeDirty.webgpu = true;
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
    this.packet = packet;
    this.invalidateRenderCaches();
    this.disposeCachedSnapshots();
  }

  setLayers(layers: MapLayers) {
    this.layers = { ...layers };
    this.invalidateRenderCaches();
  }

  async render() {
    if (!this.packet || !this.runtime || !this.renderMode) {
      return;
    }

    if (!this.renderer || !this.scene || !this.camera) {
      return;
    }

    await this.rebuildScene();
    this.modeDirty[this.renderMode] = false;
    this.resize();
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
    if (!this.packet?.mapJson) {
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

    for (const root of this.managedRoots) {
      this.scene.remove(root);
      disposeObject(root);
    }
    this.managedRoots = [];
    this.layerRoots = {};
    this.lineMaterials = [];
    this.svgRenderDirty = true;
    this.nativeSvgViewport = null;

    const { scene, layerRoots, roots, lineMaterials } = await this.createSceneGraph(
      this.renderMode,
    );
    this.scene = scene;
    this.layerRoots = layerRoots;
    this.managedRoots = roots;
    this.lineMaterials = lineMaterials;

    this.applyLayerVisibility();
    this.draw();
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

  private async createSceneGraph(mode: RenderBackend) {
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

    if (useTextureFallback && this.packet.mapJson) {
      const texture = await svgMarkupToTexture(
        this.runtime,
        await this.buildMapSvgMarkup(),
        this.packet.metadata.imageWidth,
        this.packet.metadata.imageHeight,
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

    const backgroundPlane = new this.runtime.THREE.Mesh(
      new this.runtime.THREE.PlaneGeometry(
        this.packet.metadata.imageWidth,
        this.packet.metadata.imageHeight,
      ),
      new this.runtime.THREE.MeshBasicMaterial({ color: WEBGPU_SURFACE }),
    );
    backgroundPlane.position.set(0, 0, -(this.packet.metadata.elevationScale + 32));
    scene.add(backgroundPlane);
    roots.push(backgroundPlane);

    const lightRig = buildTerrainLights(this.runtime, this.packet);
    scene.add(lightRig);
    roots.push(lightRig);

    const terrainGeometry = buildTerrainGeometry(this.runtime, this.packet);
    if (!terrainGeometry) {
      throw new Error("No terrain geometry available for WebGPU rendering");
    }

    const surfaceTextures =
      this.packet.textures.terrainAlbedo &&
      this.packet.textures.roughness &&
      this.packet.textures.ao &&
      this.packet.textures.waterColor &&
      this.packet.textures.waterAlpha &&
      this.packet.textures.coastGlow
        ? {
            terrainAlbedo: dataTexture(
              this.runtime,
              this.packet.textures.terrainAlbedo,
              this.packet.metadata.terrainWidth,
              this.packet.metadata.terrainHeight,
              true,
            ),
            height: dataTexture(
              this.runtime,
              this.packet.textures.height,
              this.packet.metadata.terrainWidth,
              this.packet.metadata.terrainHeight,
            ),
            roughness: dataTexture(
              this.runtime,
              this.packet.textures.roughness,
              this.packet.metadata.terrainWidth,
              this.packet.metadata.terrainHeight,
            ),
            ao: dataTexture(
              this.runtime,
              this.packet.textures.ao,
              this.packet.metadata.terrainWidth,
              this.packet.metadata.terrainHeight,
            ),
            waterColor: dataTexture(
              this.runtime,
              this.packet.textures.waterColor,
              this.packet.metadata.terrainWidth,
              this.packet.metadata.terrainHeight,
              true,
            ),
            waterAlpha: dataTexture(
              this.runtime,
              this.packet.textures.waterAlpha,
              this.packet.metadata.terrainWidth,
              this.packet.metadata.terrainHeight,
            ),
            coastGlow: dataTexture(
              this.runtime,
              this.packet.textures.coastGlow,
              this.packet.metadata.terrainWidth,
              this.packet.metadata.terrainHeight,
            ),
          }
        : buildSurfaceTextures(this.runtime, this.packet);
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
    scene.add(terrainMesh);
    roots.push(terrainMesh);

    const waterMesh = buildOverlayPlane(
      this.runtime,
      this.packet.metadata.imageWidth,
      this.packet.metadata.imageHeight,
      0.45,
      buildWaterMaterial(this.runtime, surfaceTextures.waterColor, surfaceTextures.waterAlpha),
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
    setRenderOrder(cityRoot, 50);
    layerRoots.city = cityRoot;
    scene.add(cityRoot);
    roots.push(cityRoot);

    const townRoot = this.createInstancedTownGroup();
    setRenderOrder(townRoot, 50);
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
    const { THREE } = this.runtime!;
    const uvAttribute = terrainGeometry.getAttribute("uv");
    if (uvAttribute && !terrainGeometry.getAttribute("uv2")) {
      terrainGeometry.setAttribute(
        "uv2",
        new THREE.BufferAttribute(new Float32Array(uvAttribute.array as ArrayLike<number>), 2),
      );
    }

    return new THREE.MeshStandardMaterial({
      color: 0xffffff,
      map: albedoTexture,
      bumpMap: heightTexture,
      bumpScale: this.packet ? this.packet.metadata.elevationScale * 0.085 : 5,
      roughness: 0.96,
      roughnessMap: roughnessTexture,
      aoMap: aoTexture,
      aoMapIntensity: 0.45,
      metalness: 0,
      side: THREE.DoubleSide,
    });
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
    geometry.setPositions(positions);

    const material = new this.runtime.Line2NodeMaterial({
      color: style.color,
      linewidth: style.linewidth,
      opacity: style.opacity,
      dashed: style.dashed,
      dashSize: 4 * this.packet!.metadata.drawScale,
      gapSize: 5 * this.packet!.metadata.drawScale,
    });
    configureOverlayMaterial(material);
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

    for (const style of styles) {
      const styleGroup = new this.runtime!.THREE.Group();

      for (let pathIndex = 0; pathIndex < layer.offsets.length - 1; pathIndex += 1) {
        const positions = pathPositionSlice(layer, pathIndex);
        const pathLine = this.createPolyline(positions, style, lineMaterials);
        if (!pathLine) continue;
        styleGroup.add(pathLine);
        hasLines = true;
      }

      if (styleGroup.children.length > 0) {
        group.add(styleGroup);
      }
    }

    return hasLines ? group : null;
  }

  private createPolyline(
    packetPositions: Float32Array,
    style: { color: number; opacity: number; linewidth: number; dashed: boolean },
    lineMaterials: Array<{ resolution?: import("three").Vector2; linewidth?: number }>,
  ) {
    if (!this.runtime || packetPositions.length < 6) return null;

    const positions = packetToThreeTriplets(packetPositions);
    const geometry = new this.runtime.LineGeometry();
    geometry.setPositions(positions);

    const material = new this.runtime.Line2NodeMaterial({
      color: style.color,
      linewidth: style.linewidth,
      worldUnits: true,
      opacity: style.opacity,
      dashed: style.dashed,
      dashSize: 4 * this.packet!.metadata.drawScale,
      gapSize: 5 * this.packet!.metadata.drawScale,
    });
    configureOverlayMaterial(material);
    const line = new this.runtime.WebGPULine2(geometry, material);
    if ("computeLineDistances" in line && typeof line.computeLineDistances === "function") {
      line.computeLineDistances();
    }
    lineMaterials.push(material);
    return line;
  }

  private createInstancedCityGroup() {
    const group = new this.runtime!.THREE.Group();
    const outerRadius = 10 * this.packet!.metadata.drawScale;
    const innerRadius = 5 * this.packet!.metadata.drawScale;
    const positions = packetToThreeTriplets(this.packet!.markers.city);

    const outer = new this.runtime!.THREE.InstancedMesh(
      new this.runtime!.THREE.CircleGeometry(outerRadius, 32),
      new this.runtime!.THREE.MeshBasicMaterial({ color: CITY_OUTER_COLOR }),
      positions.length / 3,
    );
    const inner = new this.runtime!.THREE.InstancedMesh(
      new this.runtime!.THREE.CircleGeometry(innerRadius, 32),
      new this.runtime!.THREE.MeshBasicMaterial({ color: CITY_INNER_COLOR }),
      positions.length / 3,
    );
    configureOverlayMaterial(outer.material);
    configureOverlayMaterial(inner.material);

    const dummy = new this.runtime!.THREE.Object3D();
    for (let index = 0; index < positions.length; index += 3) {
      dummy.position.set(positions[index], positions[index + 1], positions[index + 2] + 0.8);
      dummy.updateMatrix();
      outer.setMatrixAt(index / 3, dummy.matrix);
      dummy.position.set(positions[index], positions[index + 1], positions[index + 2] + 1.2);
      dummy.updateMatrix();
      inner.setMatrixAt(index / 3, dummy.matrix);
    }
    outer.instanceMatrix.needsUpdate = true;
    inner.instanceMatrix.needsUpdate = true;

    group.add(outer);
    group.add(inner);
    return group;
  }

  private createInstancedTownGroup() {
    const group = new this.runtime!.THREE.Group();
    const radius = 5 * this.packet!.metadata.drawScale;
    const positions = packetToThreeTriplets(this.packet!.markers.town);
    const mesh = new this.runtime!.THREE.InstancedMesh(
      new this.runtime!.THREE.CircleGeometry(radius, 28),
      new this.runtime!.THREE.MeshBasicMaterial({ color: TOWN_COLOR }),
      positions.length / 3,
    );
    configureOverlayMaterial(mesh.material);

    const dummy = new this.runtime!.THREE.Object3D();
    for (let index = 0; index < positions.length; index += 3) {
      dummy.position.set(positions[index], positions[index + 1], positions[index + 2] + 0.8);
      dummy.updateMatrix();
      mesh.setMatrixAt(index / 3, dummy.matrix);
    }
    mesh.instanceMatrix.needsUpdate = true;
    group.add(mesh);
    return group;
  }

  private createLabelGroup() {
    const group = new this.runtime!.THREE.Group();
    const anchors = packetToThreeTriplets(this.packet!.labels.anchors);
    const items = this.packet!.labels.items;

    for (let index = 0; index < items.length; index += 1) {
      const item = items[index];
      if (!item?.text) continue;
      const anchorIndex = index * 3;
      const { mesh, offsetX, offsetY } = this.createLabelMesh(
        item.text,
        item.fontface,
        item.fontsize,
      );
      mesh.position.set(
        anchors[anchorIndex] + offsetX,
        anchors[anchorIndex + 1] + offsetY,
        anchors[anchorIndex + 2] + LABEL_Z_OFFSET,
      );
      group.add(mesh);
    }

    return group;
  }

  private createLabelMesh(text: string, fontface: string, fontsize: number) {
    const canvas = document.createElement("canvas");
    const context = canvas.getContext("2d");
    if (!context) {
      throw new Error("Failed to create label canvas context");
    }

    const pixelRatio = Math.max(1, Math.min(window.devicePixelRatio || 1, 2));
    const fontSize = Math.max(1, fontsize);
    const haloWidth = Math.max(3, Math.round(fontSize * 0.12));
    const padding = Math.max(4, Math.round(fontSize * 0.24));
    const font = `${fontSize}px ${resolveLabelFontFamily(fontface)}`;

    context.font = font;
    context.textAlign = "left";
    context.textBaseline = "alphabetic";

    const metrics = context.measureText(text);
    const left = metrics.actualBoundingBoxLeft || 0;
    const right = metrics.actualBoundingBoxRight || metrics.width || fontSize;
    const ascent = metrics.actualBoundingBoxAscent || fontSize * 0.78;
    const descent = metrics.actualBoundingBoxDescent || fontSize * 0.22;
    const logicalWidth = Math.max(1, Math.ceil(left + right + padding * 2 + haloWidth * 2));
    const logicalHeight = Math.max(1, Math.ceil(ascent + descent + padding * 2 + haloWidth * 2));
    const drawX = padding + haloWidth + left;
    const drawY = padding + haloWidth + ascent;

    canvas.width = Math.ceil(logicalWidth * pixelRatio);
    canvas.height = Math.ceil(logicalHeight * pixelRatio);

    context.setTransform(pixelRatio, 0, 0, pixelRatio, 0, 0);
    context.clearRect(0, 0, logicalWidth, logicalHeight);
    context.font = font;
    context.textAlign = "left";
    context.textBaseline = "alphabetic";
    context.lineJoin = "round";
    context.miterLimit = 2;
    context.strokeStyle = `#${LABEL_HALO_COLOR.toString(16).padStart(6, "0")}`;
    context.fillStyle = `#${LABEL_COLOR.toString(16).padStart(6, "0")}`;
    context.lineWidth = haloWidth;
    context.strokeText(text, drawX, drawY);
    context.fillText(text, drawX, drawY);

    const texture = new this.runtime!.THREE.CanvasTexture(canvas);
    texture.colorSpace = this.runtime!.THREE.SRGBColorSpace;
    texture.minFilter = this.runtime!.THREE.LinearFilter;
    texture.magFilter = this.runtime!.THREE.LinearFilter;
    texture.generateMipmaps = false;
    texture.needsUpdate = true;

    const geometry = new this.runtime!.THREE.PlaneGeometry(logicalWidth, logicalHeight);
    const material = new this.runtime!.THREE.MeshBasicMaterial({
      alphaTest: 0.05,
      depthTest: false,
      depthWrite: false,
      map: texture,
      toneMapped: false,
      transparent: true,
    });
    configureOverlayMaterial(material);
    const mesh = new this.runtime!.THREE.Mesh(geometry, material);
    mesh.renderOrder = 40;

    return {
      mesh,
      offsetX: logicalWidth * 0.5 - drawX,
      offsetY: drawY - logicalHeight * 0.5,
    };
  }

  private applyLayerVisibility() {
    for (const [key, root] of Object.entries(this.layerRoots) as Array<
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
    if (!this.packet?.mapJson) {
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
        mapJson: this.packet!.mapJson,
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
