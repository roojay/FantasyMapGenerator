export type RenderBackend = "webgpu" | "svg";
export type RendererPreference = RenderBackend | "auto";
export type RendererRuntimeBackend = "webgpu" | "webgl2" | "svg" | "unknown";
export type AppLanguage = "zh-CN" | "en";

export interface MapLabel {
  charextents: number[];
  extents: [number, number, number, number];
  fontface: string;
  fontsize: number;
  position: [number, number];
  score: number;
  text: string;
}

export interface MapLabelRenderItem {
  fontface: string;
  fontsize: number;
  text: string;
}

export interface MapRaster<T extends number = number> {
  width: number;
  height: number;
  data: T[];
}

export interface MapExportData {
  city: number[];
  contour: number[][];
  draw_scale: number;
  image_height: number;
  image_width: number;
  label: MapLabel[];
  river: number[][];
  slope: number[];
  territory: number[][];
  town: number[];
  heightmap?: MapRaster<number>;
  flux_map?: MapRaster<number>;
  land_mask?: MapRaster<number>;
  land_polygons?: number[][];
}

export interface MapSceneMetadata {
  imageWidth: number;
  imageHeight: number;
  drawScale: number;
  terrainWidth: number;
  terrainHeight: number;
  elevationScale: number;
  cityCount: number;
  townCount: number;
  riverCount: number;
  territoryCount: number;
  labelCount: number;
}

export interface PathLayerPacket {
  positions: Float32Array;
  offsets: Uint32Array;
}

export interface MapScenePacket {
  metadata: MapSceneMetadata;
  terrain: {
    positions: Float32Array;
    normals: Float32Array;
    uvs: Float32Array;
    indices: Uint32Array;
  };
  textures: {
    height: Uint8Array;
    landMask: Uint8Array;
    flux: Uint8Array;
    albedo?: Uint8Array;
    terrainAlbedo?: Uint8Array;
    roughness?: Uint8Array;
    ao?: Uint8Array;
    waterColor?: Uint8Array;
    waterAlpha?: Uint8Array;
    coastGlow?: Uint8Array;
  };
  layers: {
    slopeSegments: Float32Array;
    river: PathLayerPacket;
    contour: PathLayerPacket;
    border: PathLayerPacket;
  };
  markers: {
    city: Float32Array;
    town: Float32Array;
  };
  labels: {
    bytes: Uint8Array;
    offsets: Uint32Array;
    anchors: Float32Array;
    sizes: Float32Array;
    items: MapLabelRenderItem[];
  };
  landPolygonPositions: Float32Array;
  landPolygonOffsets: Uint32Array;
  mapJson?: string;
  svgMapJson: string;
  generatedFrom?: GeneratedMapSource;
}

export interface MapLayers {
  slope: boolean;
  river: boolean;
  contour: boolean;
  border: boolean;
  city: boolean;
  town: boolean;
  label: boolean;
}

export interface PresentationPluginCapabilityMetadata {
  supports_layer_config: boolean;
  supports_direct_svg_export: boolean;
  requires_raster_data: boolean;
  requires_heightmap: boolean;
  requires_land_mask: boolean;
  embeds_raster_images: boolean;
}

export interface PresentationLayerMetadata {
  id: keyof MapLayers;
  label: string;
  default_enabled: boolean;
}

export interface PresentationConfigFieldMetadata {
  key: string;
  label: string;
  description: string;
  field_type: "boolean" | "integer" | "float";
  default_value: boolean | number | string | null;
  min?: number | null;
  max?: number | null;
  step?: number | null;
}

export interface PresentationConfigSectionMetadata {
  id: string;
  label: string;
  description: string;
  fields: PresentationConfigFieldMetadata[];
}

export interface PresentationPluginMetadata {
  id: string;
  display_name: string;
  description: string;
  output_kind: "svg_scene" | "gpu_scene_packet";
  capabilities: PresentationPluginCapabilityMetadata;
  supported_layers: PresentationLayerMetadata[];
  config_sections: PresentationConfigSectionMetadata[];
}

export interface MapConfig {
  seed: number;
  width: number;
  height: number;
  resolution: number;
  cities: number;
  towns: number;
  drawScale: number;
}

export interface GeneratedMapSource {
  seed: number;
  width: number;
  height: number;
  resolution: number;
  cities: number;
  towns: number;
  drawScale: number;
}

export interface MapPresentationPreset {
  renderer: RendererPreference;
  layers: MapLayers;
}

export interface StatusMessage {
  tone: "neutral" | "success" | "error" | "info";
  text: string;
}
