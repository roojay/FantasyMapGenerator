import { useCallback, useEffect, useRef, useState } from "react";

interface GenerateOptions {
  seed: number;
  width: number;
  height: number;
  resolution: number;
  drawScale: number;
  cities: number;
  towns: number;
  includeRasterData: boolean;
}

interface GenerateResult {
  json: string;
  seed: number;
}

export function useMapGenerator() {
  const workerRef = useRef<Worker | null>(null);
  const [isGenerating, setIsGenerating] = useState(false);

  useEffect(() => {
    // 创建 worker
    workerRef.current = new Worker(
      new URL("../workers/mapGenerator.worker.ts", import.meta.url),
      { type: "module" }
    );

    return () => {
      workerRef.current?.terminate();
      workerRef.current = null;
    };
  }, []);

  const generate = useCallback(
    (options: GenerateOptions): Promise<GenerateResult> => {
      return new Promise((resolve, reject) => {
        if (!workerRef.current) {
          reject(new Error("Worker not initialized"));
          return;
        }

        setIsGenerating(true);

        const handleMessage = (e: MessageEvent) => {
          const { type, json, seed, error } = e.data;

          workerRef.current?.removeEventListener("message", handleMessage);
          setIsGenerating(false);

          if (type === "success" && json && seed !== undefined) {
            resolve({ json, seed });
          } else {
            reject(new Error(error || "Unknown error"));
          }
        };

        workerRef.current.addEventListener("message", handleMessage);
        workerRef.current.postMessage({
          type: "generate",
          ...options
        });
      });
    },
    []
  );

  return { generate, isGenerating };
}
