import { Text } from "@mantine/core";
import { forwardRef, useCallback, useEffect, useImperativeHandle, useRef } from "react";
import { TransformComponent, TransformWrapper, type ReactZoomPanPinchContentRef } from "react-zoom-pan-pinch";
import { useTranslation } from "react-i18next";

import { FantasyMapCanvasRenderer } from "@/lib/FantasyMapCanvasRenderer";
import type { MapData, MapLayers, RenderBackend, RendererPreference } from "@/types/map";

export interface MapRendererHandle {
  exportToPNG: () => Promise<string>;
  exportToSVG: () => string;
  fitToScreen: () => void;
  resetView: () => void;
}

interface MapRendererProps {
  mapData: MapData | null;
  layers: MapLayers;
  preferredMode: RendererPreference;
  onRendererStateChange: (payload: { mode: RenderBackend | null; availableModes: RenderBackend[] }) => void;
  onRenderComplete: (mode: RenderBackend) => void;
  onRenderError: (message: string) => void;
}

export const MapRenderer = forwardRef<MapRendererHandle, MapRendererProps>(function MapRenderer(
  { mapData, layers, preferredMode, onRendererStateChange, onRenderComplete, onRenderError },
  ref
) {
  const MIN_SCALE = 0.1;
  const MAX_SCALE = 12;
  const { t } = useTranslation();
  const wrapperRef = useRef<HTMLDivElement>(null);
  const stageRef = useRef<HTMLDivElement>(null);
  const zoomRef = useRef<ReactZoomPanPinchContentRef | null>(null);
  const rendererRef = useRef<FantasyMapCanvasRenderer | null>(null);
  const userAdjustedViewRef = useRef(false);
  const lastAutoFittedMapRef = useRef<MapData | null>(null);
  const frameRef = useRef<number | null>(null);
  const callbacksRef = useRef({
    onRendererStateChange,
    onRenderComplete,
    onRenderError
  });

  useEffect(() => {
    callbacksRef.current = {
      onRendererStateChange,
      onRenderComplete,
      onRenderError
    };
  }, [onRendererStateChange, onRenderComplete, onRenderError]);

  const cancelScheduledFrame = useCallback(() => {
    if (frameRef.current !== null) {
      cancelAnimationFrame(frameRef.current);
      frameRef.current = null;
    }
  }, []);

  const scheduleViewportTask = useCallback((task: () => void) => {
    cancelScheduledFrame();
    frameRef.current = requestAnimationFrame(() => {
      frameRef.current = null;
      task();
    });
  }, [cancelScheduledFrame]);

  const calculateViewportTransform = useCallback((targetScale: number) => {
    const wrapper = wrapperRef.current;
    const contentSize = rendererRef.current?.getMapSize();
    if (!wrapper || !contentSize) return null;

    const scale = Math.min(MAX_SCALE, Math.max(MIN_SCALE, targetScale));
    return {
      scale,
      x: (wrapper.clientWidth - contentSize.width * scale) / 2,
      y: (wrapper.clientHeight - contentSize.height * scale) / 2
    };
  }, []);

  const resetView = useCallback((animationTime = 180) => {
    const centered = calculateViewportTransform(1);
    if (!centered) return;

    userAdjustedViewRef.current = false;
    zoomRef.current?.setTransform(centered.x, centered.y, centered.scale, animationTime, "easeOut");
  }, [calculateViewportTransform]);

  const fitToScreen = useCallback((animationTime = 220) => {
    const wrapper = wrapperRef.current;
    const contentSize = rendererRef.current?.getMapSize();
    if (!wrapper || !contentSize) return;

    const padding = 24;
    const scale = Math.min(
      (wrapper.clientWidth - padding * 2) / contentSize.width,
      (wrapper.clientHeight - padding * 2) / contentSize.height,
      1
    );
    const nextTransform = calculateViewportTransform(scale);
    if (!nextTransform) return;

    userAdjustedViewRef.current = false;
    zoomRef.current?.setTransform(nextTransform.x, nextTransform.y, nextTransform.scale, animationTime, "easeOut");
  }, [calculateViewportTransform]);

  useEffect(() => cancelScheduledFrame, [cancelScheduledFrame]);

  useImperativeHandle(ref, () => ({
    exportToPNG: async () => {
      if (!rendererRef.current) throw new Error("Renderer is not ready");
      return rendererRef.current.exportToPNG();
    },
    exportToSVG: () => {
      if (!rendererRef.current) throw new Error("Renderer is not ready");
      return rendererRef.current.buildSVGString();
    },
    fitToScreen: () => fitToScreen(),
    resetView: () => resetView()
  }), [fitToScreen, resetView]);

  useEffect(() => {
    if (!stageRef.current) return;

    const renderer = new FantasyMapCanvasRenderer(stageRef.current);
    rendererRef.current = renderer;

    return () => {
      renderer.destroy();
      rendererRef.current = null;
    };
  }, []);

  useEffect(() => {
    const renderer = rendererRef.current;
    if (!renderer) return;

    let active = true;

    const initializeRenderer = async () => {
      try {
        const state = await renderer.initialize(preferredMode);
        if (!active) return;

        renderer.setLayers(layers);
        callbacksRef.current.onRendererStateChange(state);

        if (mapData) {
          renderer.loadMapData(mapData);
          await renderer.render();
          if (!userAdjustedViewRef.current && lastAutoFittedMapRef.current !== mapData) {
            scheduleViewportTask(() => {
              fitToScreen(0);
              lastAutoFittedMapRef.current = mapData;
            });
          }
          callbacksRef.current.onRenderComplete(state.mode);
        }
      } catch (error) {
        if (active) {
          callbacksRef.current.onRenderError(error instanceof Error ? error.message : String(error));
        }
      }
    };

    void initializeRenderer();
    return () => { active = false; };
  }, [fitToScreen, layers, mapData, preferredMode, scheduleViewportTask]);

  useEffect(() => {
    const renderer = rendererRef.current;
    if (!renderer) return;

    if (!mapData) {
      lastAutoFittedMapRef.current = null;
      return;
    }

    let active = true;
    const shouldAutoFit = lastAutoFittedMapRef.current !== mapData;

    const renderMap = async () => {
      try {
        renderer.setLayers(layers);
        renderer.loadMapData(mapData);
        await renderer.render();

        if (!active || !renderer.currentMode) return;

        if (shouldAutoFit) {
          scheduleViewportTask(() => {
            fitToScreen(0);
            lastAutoFittedMapRef.current = mapData;
          });
        }
        callbacksRef.current.onRenderComplete(renderer.currentMode);
      } catch (error) {
        if (active) {
          callbacksRef.current.onRenderError(error instanceof Error ? error.message : String(error));
        }
      }
    };

    void renderMap();
    return () => { active = false; };
  }, [fitToScreen, layers, mapData, scheduleViewportTask]);

  useEffect(() => {
    const wrapper = wrapperRef.current;
    if (!wrapper || !mapData) return;

    const observer = new ResizeObserver(() => {
      if (!userAdjustedViewRef.current) {
        scheduleViewportTask(() => fitToScreen(0));
      }
    });

    observer.observe(wrapper);
    return () => observer.disconnect();
  }, [fitToScreen, mapData, scheduleViewportTask]);

  return (
    <div
      ref={wrapperRef}
      className="h-full w-full overflow-hidden bg-[rgb(var(--app-bg))]"
    >
      <TransformWrapper
        ref={zoomRef}
        minScale={MIN_SCALE}
        maxScale={MAX_SCALE}
        limitToBounds={false}
        centerOnInit={false}
        centerZoomedOut={false}
        doubleClick={{ disabled: true }}
        wheel={{ step: 0.05 }}
        pinch={{ step: 5 }}
        panning={{
          velocityDisabled: false,
          allowLeftClickPan: true,
          allowMiddleClickPan: true,
          wheelPanning: false
        }}
        onPanningStart={() => {
          userAdjustedViewRef.current = true;
          wrapperRef.current?.classList.add("is-panning");
        }}
        onPanningStop={() => {
          wrapperRef.current?.classList.remove("is-panning");
        }}
        onWheelStart={() => { userAdjustedViewRef.current = true; }}
        onPinchingStart={() => { userAdjustedViewRef.current = true; }}
      >
        <TransformComponent
          wrapperClass="map-viewport !h-full !w-full"
          contentClass="map-pan-layer !h-max !w-max"
          wrapperStyle={{ width: "100%", height: "100%" }}
          contentStyle={{ width: "max-content", height: "max-content" }}
        >
          <div ref={stageRef} className={`map-stage ${mapData ? "" : "invisible"}`} />
        </TransformComponent>
      </TransformWrapper>

      {/* Empty state */}
      {!mapData && (
        <div className="absolute inset-0 grid place-items-center p-6">
          <div className="max-w-md rounded-xl border bg-[light-dark(rgba(255,255,255,0.85),rgba(15,23,42,0.6))] px-8 py-10 text-center shadow-lg backdrop-blur-xl" style={{ borderColor: "rgb(var(--app-border))" }}>
            <Text fw={700} size="xl">
              {t("app.title")}
            </Text>
            <Text c="dimmed" mt="xs">
              {t("app.subtitle")}
            </Text>
          </div>
        </div>
      )}
    </div>
  );
});
