import { useState, useEffect, useCallback } from 'react';
import type { MapData, MapConfig } from '../types/map';

export function useMapData(_config: MapConfig) {
  const [mapData, setMapData] = useState<MapData | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [generationTimeMs, setGenerationTimeMs] = useState<number | null>(null);

  const loadStaticData = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    const start = performance.now();
    try {
      const res = await fetch('/map-data.json');
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data: MapData = await res.json();
      setMapData(data);
      setGenerationTimeMs(Math.round(performance.now() - start));
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Unknown error');
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    loadStaticData();
  }, [loadStaticData]);

  const regenerate = useCallback(async () => {
    await loadStaticData();
  }, [loadStaticData]);

  return { mapData, isLoading, error, generationTimeMs, regenerate };
}
