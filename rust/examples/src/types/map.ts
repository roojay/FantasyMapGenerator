export type RenderBackend = "webgpu" | "webgl" | "canvas" | "svg";
export type RendererPreference = RenderBackend | "auto";
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

export interface MapRaster<T extends number = number> {
  width: number;
  height: number;
  data: T[];
}

export interface MapData {
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
  heightmap?: MapRaster;
  flux_map?: MapRaster;
  land_mask?: MapRaster;
  land_polygons?: number[][];
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

export interface MapConfig {
  seed: number;
  width: number;
  height: number;
  resolution: number;
  cities: number;
  towns: number;
  drawScale: number;
  renderer: RendererPreference;
}

export interface StatusMessage {
  tone: "neutral" | "success" | "error" | "info";
  text: string;
}
