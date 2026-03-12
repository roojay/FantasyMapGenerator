import { useCallback, useEffect, useRef, useState } from "react";

import type { MapScenePacket } from "@/types/map";

interface GenerateOptions {
  seed: number;
  width: number;
  height: number;
  resolution: number;
  drawScale: number;
  cities: number;
  towns: number;
}

interface GenerateResult {
  packet: MapScenePacket;
  seed: number;
}

export function useMapGenerator() {
  const workerRef = useRef<Worker | null>(null);
  const [isGenerating, setIsGenerating] = useState(false);

  useEffect(() => {
    workerRef.current = new Worker(new URL("../workers/mapGenerator.worker.ts", import.meta.url), {
      type: "module",
    });

    return () => {
      workerRef.current?.terminate();
      workerRef.current = null;
    };
  }, []);

  const generate = useCallback((options: GenerateOptions): Promise<GenerateResult> => {
    return new Promise((resolve, reject) => {
      if (!workerRef.current) {
        reject(new Error("Worker not initialized"));
        return;
      }

      setIsGenerating(true);

      const handleMessage = (
        event: MessageEvent<{
          type: "success" | "error";
          packet?: MapScenePacket;
          seed?: number;
          error?: string;
        }>,
      ) => {
        const { type, packet, seed, error } = event.data;
        workerRef.current?.removeEventListener("message", handleMessage);
        setIsGenerating(false);

        if (type === "success" && packet && seed !== undefined) {
          resolve({ packet, seed });
          return;
        }

        reject(new Error(error || "Unknown error"));
      };

      workerRef.current.addEventListener("message", handleMessage);
      workerRef.current.postMessage({
        type: "generate",
        ...options,
      });
    });
  }, []);

  return { generate, isGenerating };
}
