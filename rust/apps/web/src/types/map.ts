export interface LabelData {
  text: string;
  fontface: string;
  fontsize: number;
  position: [number, number];
  extents: [number, number, number, number];
  char_extents: number[];
  score: number;
}

export interface MapData {
  image_width: number;
  image_height: number;
  draw_scale: number;
  contour: number[][];
  river: number[][];
  slope: number[];
  city: number[];
  town: number[];
  territory: number[][];
  label: LabelData[];
}

export interface MapConfig {
  seed: number;
  width: number;
  height: number;
  resolution: number;
  cities: number;
  towns: number;
  erosionSteps: number;
}

export interface LayerVisibility {
  contour: boolean;
  rivers: boolean;
  slopes: boolean;
  territory: boolean;
  cities: boolean;
  towns: boolean;
  labels: boolean;
}

export type ColorScheme = 'light' | 'dark';
export type Language = 'en' | 'zh';
