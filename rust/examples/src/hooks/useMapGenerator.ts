import { useCallback, useEffect, useRef, useState } from "react";

import type { GeneratedMapSource, MapScenePacket } from "@/types/map";

interface GenerateOptions extends GeneratedMapSource {}

interface GenerateResult {
  packet: MapScenePacket;
  seed: number;
}

interface ReadyMessage {
  type: "ready";
}

interface WasmMemoryLog {
  afterInit: number;
  beforeGenerate: number;
  afterGenerateRenderPacket: number;
  afterPacketExtraction: number;
  afterFree: number;
}

interface WorkerTimingLog {
  wasmGenerateMs: number;
  packetExtractMs: number;
  totalWorkerMs: number;
}

interface GenerateSuccessMessage {
  type: "generate-success";
  requestId: number;
  packet: MapScenePacket;
  seed: number;
  wasmMemoryLog?: WasmMemoryLog;
  workerTiming?: WorkerTimingLog;
}

interface ExportJsonSuccessMessage {
  type: "json-success";
  requestId: number;
  json: string;
}

interface ErrorMessage {
  type: "error";
  requestId: number;
  error: string;
}

type WorkerResponse =
  | ReadyMessage
  | GenerateSuccessMessage
  | ExportJsonSuccessMessage
  | ErrorMessage;

interface PendingGenerateRequest {
  kind: "generate";
  resolve: (result: GenerateResult) => void;
  reject: (error: Error) => void;
}

interface PendingJsonRequest {
  kind: "json";
  resolve: (json: string) => void;
  reject: (error: Error) => void;
}

type PendingRequest = PendingGenerateRequest | PendingJsonRequest;

export function useMapGenerator() {
  const workerRef = useRef<Worker | null>(null);
  const requestIdRef = useRef(0);
  const pendingRequestsRef = useRef(new Map<number, PendingRequest>());
  const cleanupWorkerRef = useRef<(() => void) | null>(null);
  const restartQueuedRef = useRef(false);
  const unmountedRef = useRef(false);
  const readyPromiseRef = useRef<Promise<void> | null>(null);
  const readyResolverRef = useRef<(() => void) | null>(null);
  const readyRejectorRef = useRef<((error: Error) => void) | null>(null);
  const [isGenerating, setIsGenerating] = useState(false);
  const [isReady, setIsReady] = useState(false);
  const [readyError, setReadyError] = useState<string | null>(null);

  useEffect(() => {
    unmountedRef.current = false;
    const rejectAllPending = (error: Error) => {
      let hadGenerateRequest = false;
      for (const request of pendingRequestsRef.current.values()) {
        request.reject(error);
        if (request.kind === "generate") {
          hadGenerateRequest = true;
        }
      }
      pendingRequestsRef.current.clear();
      if (hadGenerateRequest) {
        setIsGenerating(false);
      }
    };

    const clearWorkerReferences = () => {
      workerRef.current = null;
      cleanupWorkerRef.current = null;
      readyPromiseRef.current = null;
      readyResolverRef.current = null;
      readyRejectorRef.current = null;
    };

    const disposeWorker = (rejectPending: boolean) => {
      cleanupWorkerRef.current?.();
      if (rejectPending) {
        rejectAllPending(new Error("Map generator worker terminated"));
      }
      clearWorkerReferences();
    };

    const scheduleWorkerRestart = () => {
      if (restartQueuedRef.current || unmountedRef.current) {
        return;
      }
      restartQueuedRef.current = true;

      queueMicrotask(() => {
        restartQueuedRef.current = false;
        if (unmountedRef.current || pendingRequestsRef.current.size > 0) {
          return;
        }

        disposeWorker(false);
        startWorker(true);
      });
    };

    const startWorker = (preserveReadyState: boolean) => {
      const worker = new Worker(new URL("../workers/mapGenerator.worker.ts", import.meta.url), {
        type: "module",
      });
      workerRef.current = worker;

      readyPromiseRef.current = new Promise<void>((resolve, reject) => {
        readyResolverRef.current = resolve;
        readyRejectorRef.current = reject;
      });

      if (!preserveReadyState) {
        setIsReady(false);
      }
      setReadyError(null);

      const handleMessage = (event: MessageEvent<WorkerResponse>) => {
        if (event.data.type === "ready") {
          setIsReady(true);
          setReadyError(null);
          readyResolverRef.current?.();
          readyResolverRef.current = null;
          readyRejectorRef.current = null;
          return;
        }

        const pending = pendingRequestsRef.current.get(event.data.requestId);
        if (!pending) {
          return;
        }

        pendingRequestsRef.current.delete(event.data.requestId);
        if (pending.kind === "generate") {
          const stillGenerating = [...pendingRequestsRef.current.values()].some(
            (request) => request.kind === "generate",
          );
          if (!stillGenerating) {
            setIsGenerating(false);
          }
        }

        if (event.data.type === "generate-success" && pending.kind === "generate") {
          // Store WASM memory log on window for debugging
          if (event.data.wasmMemoryLog) {
            (window as unknown as Record<string, unknown>).__wasmMemoryLog = event.data.wasmMemoryLog;
            const log = event.data.wasmMemoryLog;
            const toMB = (b: number) => (b / (1024 * 1024)).toFixed(1);
            console.log(`[WASM-MEM] afterInit=${toMB(log.afterInit)}MB beforeGenerate=${toMB(log.beforeGenerate)}MB afterGenerate=${toMB(log.afterGenerateRenderPacket)}MB afterExtract=${toMB(log.afterPacketExtraction)}MB afterFree=${toMB(log.afterFree)}MB`);
          }
          // Store worker timing on window for performance analysis
          if (event.data.workerTiming) {
            (window as unknown as Record<string, unknown>).__workerTiming = event.data.workerTiming;
            const wt = event.data.workerTiming;
            console.log(`[PERF] Worker: wasmGenerate=${wt.wasmGenerateMs}ms packetExtract=${wt.packetExtractMs}ms total=${wt.totalWorkerMs}ms`);
          }
          pending.resolve({
            packet: event.data.packet,
            seed: event.data.seed,
          });
        } else if (event.data.type === "json-success" && pending.kind === "json") {
          pending.resolve(event.data.json);
        } else if (event.data.type === "error") {
          pending.reject(new Error(event.data.error || "Unknown error"));
        } else {
          pending.reject(new Error("Unexpected worker response"));
        }

        if (pendingRequestsRef.current.size === 0) {
          scheduleWorkerRestart();
        }
      };

      const handleError = (event: ErrorEvent) => {
        const error = new Error(event.message || "Map generator worker failed");
        setReadyError(error.message);
        setIsReady(false);
        readyRejectorRef.current?.(error);
        readyResolverRef.current = null;
        readyRejectorRef.current = null;
        rejectAllPending(error);
        disposeWorker(false);

        if (!unmountedRef.current) {
          startWorker(false);
        }
      };

      cleanupWorkerRef.current = () => {
        worker.removeEventListener("message", handleMessage);
        worker.removeEventListener("error", handleError);
        worker.terminate();
      };

      worker.addEventListener("message", handleMessage);
      worker.addEventListener("error", handleError);
      worker.postMessage({ type: "init" });
    };

    startWorker(false);

    return () => {
      unmountedRef.current = true;
      disposeWorker(true);
    };
  }, []);

  const generate = useCallback(async (options: GenerateOptions): Promise<GenerateResult> => {
    if (!workerRef.current || !readyPromiseRef.current) {
      throw new Error("Worker not initialized");
    }

    await readyPromiseRef.current;

    return new Promise((resolve, reject) => {
      const requestId = ++requestIdRef.current;
      pendingRequestsRef.current.set(requestId, { kind: "generate", resolve, reject });
      setIsGenerating(true);

      workerRef.current?.postMessage({
        type: "generate",
        requestId,
        ...options,
      });
    });
  }, []);

  const exportJson = useCallback(async (options: GenerateOptions): Promise<string> => {
    if (!workerRef.current || !readyPromiseRef.current) {
      throw new Error("Worker not initialized");
    }

    await readyPromiseRef.current;

    return new Promise((resolve, reject) => {
      const requestId = ++requestIdRef.current;
      pendingRequestsRef.current.set(requestId, { kind: "json", resolve, reject });

      workerRef.current?.postMessage({
        type: "export-json",
        requestId,
        ...options,
      });
    });
  }, []);

  return { generate, exportJson, isGenerating, isReady, readyError };
}
