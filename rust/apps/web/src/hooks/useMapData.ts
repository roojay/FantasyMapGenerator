import { useState, useEffect, useCallback, useRef } from 'react';
import type { MapData, MapConfig } from '../types/map';
import { generateMapWasm, isWasmAvailable, tryLoadWasm } from '../wasm-bridge';

export function useMapData(config: MapConfig) {
  const [mapData, setMapData] = useState<MapData | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [generationTimeMs, setGenerationTimeMs] = useState<number | null>(null);
  const [usingWasm, setUsingWasm] = useState(false);
  // Keep a ref to the current config so the generate callback always sees fresh values
  const configRef = useRef(config);
  configRef.current = config;

  const generate = useCallback(async (cfg: MapConfig) => {
    setIsLoading(true);
    setError(null);
    const start = performance.now();

    try {
      // ── Attempt 1: WASM live generation ──────────────────────────────
      const wasmData = await generateMapWasm(cfg);
      if (wasmData) {
        setMapData(wasmData);
        setUsingWasm(true);
        setGenerationTimeMs(Math.round(performance.now() - start));
        return;
      }

      // ── Attempt 2: Static JSON fallback ───────────────────────────────
      const res = await fetch('/map-data.json');
      if (!res.ok) throw new Error(`HTTP ${res.status}: failed to load map-data.json`);
      const data: MapData = await res.json();
      setMapData(data);
      setUsingWasm(false);
      setGenerationTimeMs(Math.round(performance.now() - start));
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Unknown error');
    } finally {
      setIsLoading(false);
    }
  }, []);

  // Load on mount – also try to initialise WASM eagerly so the first user
  // click on "Generate Map" doesn't have an extra round-trip to load the module.
  useEffect(() => {
    // Fire-and-forget WASM preload; errors are silently ignored here
    void tryLoadWasm();
    void generate(configRef.current);
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const regenerate = useCallback(async () => {
    await generate(configRef.current);
  }, [generate]);

  return { mapData, isLoading, error, generationTimeMs, regenerate, usingWasm, isWasmAvailable };
}
