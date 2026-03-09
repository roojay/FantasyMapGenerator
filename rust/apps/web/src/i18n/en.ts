export interface TranslationKeys {
  title: string;
  header: {
    generationTime: string;
    polygons: string;
    version: string;
    seed: string;
    ms: string;
    wasmActive: string;
    wasmFallback: string;
  };
  config: {
    title: string;
    seed: string;
    width: string;
    height: string;
    resolution: string;
    cities: string;
    towns: string;
    erosionSteps: string;
    generate: string;
    generating: string;
    close: string;
    openConfig: string;
  };
  layers: {
    title: string;
    contour: string;
    rivers: string;
    slopes: string;
    territory: string;
    cities: string;
    towns: string;
    labels: string;
  };
  errors: {
    loadFailed: string;
    generateFailed: string;
    webgpuNotSupported: string;
  };
  theme: {
    light: string;
    dark: string;
  };
  lang: {
    en: string;
    zh: string;
  };
}

export const en: TranslationKeys = {
  title: 'Fantasy Map Generator',
  header: {
    generationTime: 'Generation Time',
    polygons: 'Polygons',
    version: 'Version',
    seed: 'Seed',
    ms: 'ms',
    wasmActive: 'Live generation via WebAssembly',
    wasmFallback: 'Using static demo data (run npm run wasm:build to enable live generation)',
  },
  config: {
    title: 'Map Configuration',
    seed: 'Seed',
    width: 'Width',
    height: 'Height',
    resolution: 'Resolution',
    cities: 'Cities',
    towns: 'Towns',
    erosionSteps: 'Erosion Steps',
    generate: 'Generate Map',
    generating: 'Generating...',
    close: 'Close',
    openConfig: 'Config',
  },
  layers: {
    title: 'Layers',
    contour: 'Contour',
    rivers: 'Rivers',
    slopes: 'Slopes',
    territory: 'Territory',
    cities: 'Cities',
    towns: 'Towns',
    labels: 'Labels',
  },
  errors: {
    loadFailed: 'Failed to load map data',
    generateFailed: 'Map generation failed',
    webgpuNotSupported: 'WebGPU not supported, using fallback renderer',
  },
  theme: {
    light: 'Light',
    dark: 'Dark',
  },
  lang: {
    en: 'English',
    zh: '中文',
  },
};
