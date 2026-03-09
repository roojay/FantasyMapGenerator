import { useState, useCallback } from 'react';
import type { MapConfig, LayerVisibility, ColorScheme, Language } from '../types/map';

export interface MapState {
  config: MapConfig;
  layers: LayerVisibility;
  colorScheme: ColorScheme;
  language: Language;
  isGenerating: boolean;
  generationTimeMs: number | null;
  error: string | null;
  polygonCount: number;
}

export const DEFAULT_CONFIG: MapConfig = {
  seed: 42,
  width: 1920,
  height: 1080,
  resolution: 0.08,
  cities: 5,
  towns: 15,
  erosionSteps: 3,
};

export const DEFAULT_LAYERS: LayerVisibility = {
  contour: true,
  rivers: true,
  slopes: true,
  territory: true,
  cities: true,
  towns: true,
  labels: true,
};

export function useMapStore() {
  const [config, setConfig] = useState<MapConfig>(DEFAULT_CONFIG);
  const [layers, setLayers] = useState<LayerVisibility>(DEFAULT_LAYERS);
  const [colorScheme, setColorScheme] = useState<ColorScheme>('light');
  const [language, setLanguage] = useState<Language>('en');
  const [isGenerating, setIsGenerating] = useState(false);
  const [generationTimeMs, setGenerationTimeMs] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [polygonCount, setPolygonCount] = useState(0);

  const toggleLayer = useCallback((layer: keyof LayerVisibility) => {
    setLayers(prev => ({ ...prev, [layer]: !prev[layer] }));
  }, []);

  const toggleColorScheme = useCallback(() => {
    setColorScheme(prev => (prev === 'light' ? 'dark' : 'light'));
  }, []);

  const toggleLanguage = useCallback(() => {
    setLanguage(prev => (prev === 'en' ? 'zh' : 'en'));
  }, []);

  return {
    config,
    setConfig,
    layers,
    setLayers,
    toggleLayer,
    colorScheme,
    setColorScheme,
    toggleColorScheme,
    language,
    setLanguage,
    toggleLanguage,
    isGenerating,
    setIsGenerating,
    generationTimeMs,
    setGenerationTimeMs,
    error,
    setError,
    polygonCount,
    setPolygonCount,
  };
}
