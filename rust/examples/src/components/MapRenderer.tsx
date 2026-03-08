import { Text } from "@mantine/core";
import { forwardRef, useEffect, useImperativeHandle, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

import { cn } from "@/lib/cn";
import { FantasyMapThreeRenderer } from "@/lib/FantasyMapThreeRenderer";
import type {
  MapLayers,
  MapScenePacket,
  RenderBackend,
  RendererPreference,
  RendererRuntimeBackend
} from "@/types/map";

export interface MapRendererHandle {
  exportToPNG: () => Promise<string>;
  exportToSVG: () => Promise<string>;
  fitToScreen: () => void;
  resetView: () => void;
}

interface MapRendererProps {
  mapData: MapScenePacket | null;
  layers: MapLayers;
  preferredMode: RendererPreference;
  onRendererStateChange: (payload: {
    mode: RenderBackend | null;
    actualBackend: RendererRuntimeBackend;
    availableModes: RenderBackend[];
  }) => void;
  onRenderComplete: (mode: RenderBackend) => void;
  onRenderError: (message: string) => void;
  onRendererSwitchStateChange?: (switching: boolean) => void;
}

export const MapRenderer = forwardRef<MapRendererHandle, MapRendererProps>(function MapRenderer(
  { mapData, layers, preferredMode, onRendererStateChange, onRenderComplete, onRenderError, onRendererSwitchStateChange },
  ref
) {
  const { t } = useTranslation();
  const wrapperRef = useRef<HTMLDivElement>(null);
  const stageRef = useRef<HTMLDivElement>(null);
  const rendererRef = useRef<FantasyMapThreeRenderer | null>(null);
  const renderStateRef = useRef({ mapData, layers });
  const callbacksRef = useRef({
    onRendererStateChange,
    onRenderComplete,
    onRenderError,
    onRendererSwitchStateChange
  });
  const userAdjustedViewRef = useRef(false);
  const lastAutoFittedPacketRef = useRef<MapScenePacket | null>(null);
  const [switchState, setSwitchState] = useState<{
    active: boolean;
    from: RenderBackend | null;
    to: RendererPreference;
  }>({
    active: false,
    from: null,
    to: preferredMode
  });

  useEffect(() => {
    renderStateRef.current = { mapData, layers };
  }, [layers, mapData]);

  useEffect(() => {
    callbacksRef.current = {
      onRendererStateChange,
      onRenderComplete,
      onRenderError,
      onRendererSwitchStateChange
    };
  }, [onRenderComplete, onRenderError, onRendererStateChange, onRendererSwitchStateChange]);

  useImperativeHandle(ref, () => ({
    exportToPNG: async () => {
      if (!rendererRef.current) throw new Error("Renderer is not ready");
      return rendererRef.current.exportToPNG();
    },
    exportToSVG: async () => {
      if (!rendererRef.current) throw new Error("Renderer is not ready");
      return rendererRef.current.buildSVGString();
    },
    fitToScreen: () => {
      userAdjustedViewRef.current = false;
      rendererRef.current?.fitToScreen();
    },
    resetView: () => {
      userAdjustedViewRef.current = false;
      rendererRef.current?.resetView();
    }
  }), []);

  useEffect(() => {
    if (!stageRef.current) return;

    const renderer = new FantasyMapThreeRenderer(stageRef.current);
    rendererRef.current = renderer;

    return () => {
      renderer.destroy();
      rendererRef.current = null;
    };
  }, []);

  useEffect(() => {
    const stage = stageRef.current;
    if (!stage) return;

    const markAdjusted = () => {
      userAdjustedViewRef.current = true;
    };

    stage.addEventListener("pointerdown", markAdjusted);
    stage.addEventListener("wheel", markAdjusted, { passive: true });

    return () => {
      stage.removeEventListener("pointerdown", markAdjusted);
      stage.removeEventListener("wheel", markAdjusted);
    };
  }, []);

  useEffect(() => {
    const renderer = rendererRef.current;
    if (!renderer) return;

    let active = true;
    const shouldAnimateSwitch = Boolean(renderStateRef.current.mapData && renderer.currentMode);

    if (shouldAnimateSwitch) {
      setSwitchState({
        active: true,
        from: renderer.currentMode,
        to: preferredMode
      });
      callbacksRef.current.onRendererSwitchStateChange?.(true);
    }

    const initializeRenderer = async () => {
      try {
        const state = await renderer.initialize(preferredMode);
        if (!active) return;

        const currentState = renderStateRef.current;
        renderer.setLayers(currentState.layers);
        callbacksRef.current.onRendererStateChange({
          ...state,
          actualBackend: renderer.actualBackend
        });

        if (currentState.mapData) {
          renderer.loadMapData(currentState.mapData);
          await renderer.render();
          if (lastAutoFittedPacketRef.current !== currentState.mapData) {
            userAdjustedViewRef.current = false;
            renderer.fitToScreen();
            lastAutoFittedPacketRef.current = currentState.mapData;
          }
          if (shouldAnimateSwitch) {
            setSwitchState((current) => ({ ...current, active: false }));
            callbacksRef.current.onRendererSwitchStateChange?.(false);
          }
          callbacksRef.current.onRenderComplete(state.mode);
        } else if (shouldAnimateSwitch) {
          setSwitchState((current) => ({ ...current, active: false }));
          callbacksRef.current.onRendererSwitchStateChange?.(false);
        }
      } catch (error) {
        if (active) {
          if (shouldAnimateSwitch) {
            setSwitchState((current) => ({ ...current, active: false }));
            callbacksRef.current.onRendererSwitchStateChange?.(false);
          }
          callbacksRef.current.onRenderError(error instanceof Error ? error.message : String(error));
        }
      }
    };

    void initializeRenderer();

    return () => {
      active = false;
    };
  }, [preferredMode]);

  useEffect(() => {
    const renderer = rendererRef.current;
    if (!renderer?.currentMode || !mapData) return;

    let active = true;

    const renderMap = async () => {
      try {
        renderer.setLayers(layers);
        renderer.loadMapData(mapData);
        await renderer.render();
        if (!active || !renderer.currentMode) return;

        if (lastAutoFittedPacketRef.current !== mapData) {
          userAdjustedViewRef.current = false;
          renderer.fitToScreen();
          lastAutoFittedPacketRef.current = mapData;
        }
        callbacksRef.current.onRenderComplete(renderer.currentMode);
      } catch (error) {
        if (active) {
          callbacksRef.current.onRenderError(error instanceof Error ? error.message : String(error));
        }
      }
    };

    void renderMap();

    return () => {
      active = false;
    };
  }, [layers, mapData]);

  useEffect(() => {
    const wrapper = wrapperRef.current;
    const renderer = rendererRef.current;
    if (!wrapper || !renderer) return;

    const observer = new ResizeObserver(() => {
      renderer.resize();
      if (!userAdjustedViewRef.current) {
        renderer.fitToScreen();
      }
    });

    observer.observe(wrapper);
    return () => observer.disconnect();
  }, []);

  return (
    <div
      ref={wrapperRef}
      className="h-full w-full overflow-hidden bg-[rgb(var(--app-bg))]"
    >
      <div ref={stageRef} className={`map-stage h-full w-full ${mapData ? "" : "invisible"}`} />

      {mapData && (
        <div
          className={cn(
            "pointer-events-none absolute inset-0 z-20 transition-opacity duration-300",
            switchState.active ? "opacity-100" : "opacity-0"
          )}
          aria-hidden={!switchState.active}
        >
          <div className="renderer-switch-overlay absolute inset-0" />
          <div className="renderer-switch-beam absolute inset-y-0 -left-1/3 w-1/3" />
          <div className="absolute inset-x-0 top-1/2 flex -translate-y-1/2 justify-center px-6">
            <div className="renderer-switch-card max-w-sm rounded-2xl px-5 py-4 text-center">
              <Text size="xs" tt="uppercase" fw={700} className="renderer-switch-kicker tracking-[0.28em]">
                {t("status.switching")}
              </Text>
              <Text mt={8} size="lg" fw={700}>
                {switchState.from ? `${switchState.from.toUpperCase()} -> ` : ""}
                {t(`renderers.${switchState.to}`)}
              </Text>
              <Text mt={6} size="sm" c="dimmed">
                {t("messages.switchingHint", { mode: t(`renderers.${switchState.to}`) })}
              </Text>
            </div>
          </div>
        </div>
      )}

      {!mapData && (
        <div className="absolute inset-0 grid place-items-center p-6">
          <div
            className="max-w-md rounded-xl border bg-[light-dark(rgba(255,255,255,0.85),rgba(15,23,42,0.6))] px-8 py-10 text-center shadow-lg backdrop-blur-xl"
            style={{ borderColor: "rgb(var(--app-border))" }}
          >
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
