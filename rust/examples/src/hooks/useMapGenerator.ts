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

interface GenerateSuccessMessage {
  type: "generate-success";
  requestId: number;
  packet: MapScenePacket;
  seed: number;
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
  const readyPromiseRef = useRef<Promise<void> | null>(null);
  const readyResolverRef = useRef<(() => void) | null>(null);
  const readyRejectorRef = useRef<((error: Error) => void) | null>(null);
  const [isGenerating, setIsGenerating] = useState(false);
  const [isReady, setIsReady] = useState(false);
  const [readyError, setReadyError] = useState<string | null>(null);

  useEffect(() => {
    const worker = new Worker(new URL("../workers/mapGenerator.worker.ts", import.meta.url), {
      type: "module",
    });
    workerRef.current = worker;

    readyPromiseRef.current = new Promise<void>((resolve, reject) => {
      readyResolverRef.current = resolve;
      readyRejectorRef.current = reject;
    });

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
        pending.resolve({
          packet: event.data.packet,
          seed: event.data.seed,
        });
        return;
      }

      if (event.data.type === "json-success" && pending.kind === "json") {
        pending.resolve(event.data.json);
        return;
      }

      if (event.data.type === "error") {
        pending.reject(new Error(event.data.error || "Unknown error"));
        return;
      }

      pending.reject(new Error("Unexpected worker response"));
    };

    const handleError = (event: ErrorEvent) => {
      const error = new Error(event.message || "Map generator worker failed");
      setReadyError(error.message);
      setIsReady(false);
      readyRejectorRef.current?.(error);
      readyResolverRef.current = null;
      readyRejectorRef.current = null;
      rejectAllPending(error);
    };

    worker.addEventListener("message", handleMessage);
    worker.addEventListener("error", handleError);
    worker.postMessage({ type: "init" });

    return () => {
      worker.removeEventListener("message", handleMessage);
      worker.removeEventListener("error", handleError);
      rejectAllPending(new Error("Map generator worker terminated"));
      worker.terminate();
      workerRef.current = null;
      readyPromiseRef.current = null;
      readyResolverRef.current = null;
      readyRejectorRef.current = null;
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
