import { startTransition, useCallback, useEffect, useRef, useState } from "react";
import {
  ActionIcon,
  Box,
  Drawer,
  MantineProvider,
  Tooltip,
  useMantineColorScheme,
} from "@mantine/core";
import { useDisclosure, useLocalStorage } from "@mantine/hooks";
import {
  IconAdjustmentsHorizontal,
  IconLayoutSidebarLeftCollapse,
  IconLayoutSidebarLeftExpand,
} from "@tabler/icons-react";
import { useTranslation } from "react-i18next";

import { ControlPanel } from "@/components/ControlPanel";
import { LayerControl } from "@/components/LayerControl";
import { LoadingOverlay } from "@/components/LoadingOverlay";
import { MapRenderer, type MapRendererHandle } from "@/components/MapRenderer";
import { MapStats } from "@/components/MapStats";
import { StatusBar } from "@/components/StatusBar";
import { TopToolbar } from "@/components/TopToolbar";
import { cn } from "@/lib/cn";
import { downloadBlob, downloadDataUrl } from "@/lib/download";
import "@/lib/i18n";
import { parseMapExportJson } from "@/lib/mapScenePacket";
import {
  buildPresentationExportSlug,
  createDefaultPresentationPreset,
  parseStoredPresentationPreset,
  PRESENTATION_PRESET_STORAGE_KEY,
  serializePresentationPreset,
} from "@/lib/presentationPreset";
import {
  APP_LANGUAGE_STORAGE_KEY,
  deserializeAppLanguage,
  getBrowserLanguage,
  serializeAppLanguage,
} from "@/lib/language";
import { useMapGenerator } from "@/hooks/useMapGenerator";
import { colorSchemeManager, cssVariablesResolver, theme } from "@/theme";
import type {
  AppLanguage,
  MapConfig,
  MapPresentationPreset,
  MapScenePacket,
  RenderBackend,
  RendererRuntimeBackend,
  StatusMessage,
} from "@/types/map";

const defaultConfig: MapConfig = {
  seed: 0,
  width: 1920,
  height: 1080,
  resolution: 0.08,
  cities: 5,
  towns: 10,
  drawScale: 1,
};

interface BlockingTaskState {
  message: string;
  detail: string;
}

function waitForNextPaint() {
  return new Promise<void>((resolve) => {
    requestAnimationFrame(() => resolve());
  });
}

function AppContent() {
  const { t, i18n } = useTranslation();
  const fileInputRef = useRef<HTMLInputElement>(null);
  const rendererRef = useRef<MapRendererHandle>(null);
  const { toggleColorScheme } = useMantineColorScheme();
  const {
    generate: generateMap,
    exportJson: exportMapJson,
    isReady: generatorReady,
    readyError,
  } = useMapGenerator();

  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [mobilePanelOpened, { open: openMobilePanel, close: closeMobilePanel }] =
    useDisclosure(false);
  const [language, setLanguage] = useLocalStorage<AppLanguage>({
    key: APP_LANGUAGE_STORAGE_KEY,
    defaultValue: getBrowserLanguage(),
    getInitialValueInEffect: false,
    deserialize: deserializeAppLanguage,
    serialize: serializeAppLanguage,
  });

  const [config, setConfig] = useState(defaultConfig);
  const [presentation, setPresentation] = useState<MapPresentationPreset | null>(null);
  const [mapData, setMapData] = useState<MapScenePacket | null>(null);
  const [availableModes, setAvailableModes] = useState<RenderBackend[]>(["svg"]);
  const [renderMode, setRenderMode] = useState<RenderBackend | null>(null);
  const [actualBackend, setActualBackend] = useState<RendererRuntimeBackend>("unknown");
  const [loading, setLoading] = useState(false);
  const [exporting, setExporting] = useState(false);
  const [rendererSwitching, setRendererSwitching] = useState(false);
  const [blockingTask, setBlockingTask] = useState<BlockingTaskState | null>(null);
  const [status, setStatus] = useState<StatusMessage>({
    tone: "neutral",
    text: t("status.booting"),
  });
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    // 仅在语言实际变化时切换，避免初始化时的重复设置
    if (i18n.language !== language) {
      void i18n.changeLanguage(language);
    }
    document.documentElement.lang = language;
  }, [i18n, language]);

  useEffect(() => {
    if (readyError) {
      setError(t("errors.wasmInit", { message: readyError }));
      setStatus({ tone: "error", text: t("status.ready") });
      return;
    }

    if (generatorReady) {
      setError(null);
      setStatus({ tone: "success", text: t("status.wasmReady") });
      return;
    }

    setStatus({ tone: "neutral", text: t("status.booting") });
  }, [generatorReady, readyError, t]);

  useEffect(() => {
    const storedValue =
      typeof window !== "undefined"
        ? window.localStorage.getItem(PRESENTATION_PRESET_STORAGE_KEY)
        : null;
    const parsedStored = parseStoredPresentationPreset(storedValue);
    setPresentation(parsedStored ?? createDefaultPresentationPreset());
  }, [t]);

  useEffect(() => {
    if (!presentation || typeof window === "undefined") {
      return;
    }

    window.localStorage.setItem(
      PRESENTATION_PRESET_STORAGE_KEY,
      serializePresentationPreset(presentation),
    );
  }, [presentation]);

  const updateConfig = useCallback(
    <Key extends keyof MapConfig>(key: Key, value: MapConfig[Key]) => {
      setConfig((current) => ({ ...current, [key]: value }));
    },
    [],
  );

  const updatePresentation = useCallback(
    <Key extends keyof MapPresentationPreset>(key: Key, value: MapPresentationPreset[Key]) => {
      setPresentation((current) => (current ? { ...current, [key]: value } : current));
    },
    [],
  );

  const updateLayer = useCallback((key: keyof MapPresentationPreset["layers"], value: boolean) => {
    setPresentation((current) =>
      current
        ? {
            ...current,
            layers: {
              ...current.layers,
              [key]: value,
            },
          }
        : current,
    );
  }, []);

  const hydrateMap = useCallback((packet: MapScenePacket) => {
    startTransition(() => {
      setMapData(packet);
    });
  }, []);

  const generateMapData = useCallback(async () => {
    setLoading(true);
    setError(null);
    setStatus({ tone: "neutral", text: t("status.generating") });
    setBlockingTask({
      message: t("status.generating"),
      detail: t("messages.generatingHint"),
    });

    try {
      const result = await generateMap({
        seed: config.seed,
        width: config.width,
        height: config.height,
        resolution: config.resolution,
        drawScale: config.drawScale,
        cities: config.cities,
        towns: config.towns,
      });

      setConfig((current) => ({ ...current, seed: result.seed }));
      hydrateMap(result.packet);
      setStatus({ tone: "neutral", text: t("status.rendering") });
      setBlockingTask({
        message: t("status.rendering"),
        detail: t("messages.renderingHint"),
      });
    } catch (cause) {
      setLoading(false);
      setBlockingTask(null);
      setError(
        t("errors.generator", { message: cause instanceof Error ? cause.message : String(cause) }),
      );
      setStatus({ tone: "error", text: t("status.ready") });
    }
  }, [config, generateMap, hydrateMap, t]);

  const handleGenerate = useCallback(async () => {
    await generateMapData();
  }, [generateMapData]);

  const handleImportFile = useCallback(
    async (event: React.ChangeEvent<HTMLInputElement>) => {
      const file = event.target.files?.[0];
      if (!file) return;

      setLoading(true);
      setError(null);
      setStatus({ tone: "neutral", text: t("status.importing") });
      setBlockingTask({
        message: t("status.importing"),
        detail: t("messages.importingHint"),
      });

      try {
        const json = await file.text();
        hydrateMap(parseMapExportJson(json));
        setStatus({ tone: "neutral", text: t("status.rendering") });
        setBlockingTask({
          message: t("status.rendering"),
          detail: t("messages.renderingHint"),
        });
      } catch (cause) {
        setLoading(false);
        setBlockingTask(null);
        setError(
          t("errors.import", { message: cause instanceof Error ? cause.message : String(cause) }),
        );
        setStatus({ tone: "error", text: t("status.ready") });
      } finally {
        event.target.value = "";
      }
    },
    [hydrateMap, t],
  );

  const handleExport = useCallback(
    async (format: "json" | "png" | "svg") => {
      try {
        if (!mapData) return;
        if (!presentation) return;
        const exportSlug = buildPresentationExportSlug(presentation);
        const exportTimestamp = Date.now();
        const exportLabel = format.toUpperCase();

        setExporting(true);
        setError(null);
        setStatus({ tone: "info", text: t(`status.exporting${exportLabel}`) });
        setBlockingTask({
          message: t(`status.exporting${exportLabel}`),
          detail: t("messages.exportingHint", { format: exportLabel }),
        });
        await waitForNextPaint();

        if (format === "json") {
          const json =
            mapData.mapJson ??
            (mapData.generatedFrom ? await exportMapJson(mapData.generatedFrom) : null);
          if (!json) {
            throw new Error("JSON export is unavailable for this map");
          }

          if (!mapData.mapJson) {
            startTransition(() => {
              setMapData((current) => {
                if (current !== mapData) {
                  return current;
                }

                return {
                  ...current,
                  mapJson: json,
                };
              });
            });
          }

          downloadBlob(
            new Blob([json], { type: "application/json" }),
            `fantasy_map_${exportSlug}_${exportTimestamp}.json`,
          );
          setExporting(false);
          setBlockingTask(null);
          setStatus({ tone: "success", text: t("status.exported") });
          return;
        }

        if (format === "png") {
          const png = await rendererRef.current?.exportToPNG();
          if (!png) throw new Error("Renderer is not ready");

          downloadDataUrl(png, `fantasy_map_${exportSlug}_${exportTimestamp}.png`);
          setExporting(false);
          setBlockingTask(null);
          setStatus({ tone: "success", text: t("status.exported") });
          return;
        }

        const svg = await rendererRef.current?.exportToSVG();
        if (!svg) throw new Error("Renderer is not ready");

        downloadBlob(
          new Blob([svg], { type: "image/svg+xml;charset=utf-8" }),
          `fantasy_map_${exportSlug}_${exportTimestamp}.svg`,
        );
        setExporting(false);
        setBlockingTask(null);
        setStatus({ tone: "success", text: t("status.exported") });
      } catch (cause) {
        setExporting(false);
        setBlockingTask(null);
        setError(
          t("errors.export", { message: cause instanceof Error ? cause.message : String(cause) }),
        );
        setStatus({ tone: "error", text: t("status.ready") });
      }
    },
    [exportMapJson, mapData, presentation, t],
  );

  const toggleTheme = useCallback(() => {
    toggleColorScheme();
  }, [toggleColorScheme]);

  const blockingBusy = loading || rendererSwitching || exporting;
  const controlsBusy = blockingBusy || !generatorReady;

  const sidebar = presentation ? (
    <ControlPanel
      config={config}
      isBusy={controlsBusy}
      onConfigChange={updateConfig}
      onGenerate={handleGenerate}
    />
  ) : null;

  return (
    <Box className="grid h-dvh w-full overflow-hidden lg:grid-cols-[auto_1fr]">
      <input
        ref={fileInputRef}
        type="file"
        accept=".json,application/json"
        className="hidden"
        onChange={handleImportFile}
      />
      {/* Desktop sidebar */}
      <Box
        component="aside"
        className={cn(
          "hidden overflow-hidden",
          "border-r transition-[width] duration-300 ease-in-out",
          "lg:block",
          sidebarOpen ? "w-[320px] 2xl:w-[360px]" : "w-0 border-r-0",
        )}
        style={{
          borderColor: "rgb(var(--app-border))",
          backgroundColor: "var(--mantine-color-body)",
        }}
      >
        <Box className="h-full w-[320px] 2xl:w-[360px]">{sidebar}</Box>
      </Box>

      {/* Main content */}
      <Box component="main" className="relative overflow-hidden">
        {presentation ? (
          <MapRenderer
            ref={rendererRef}
            mapData={mapData}
            presentation={presentation}
            onRendererSwitchStateChange={(switching) => {
              setRendererSwitching(switching);
              if (switching) {
                setError(null);
                setStatus({ tone: "info", text: t("status.switching") });
                setBlockingTask({
                  message: t("status.switching"),
                  detail: presentation
                    ? t("messages.switchingHint", { mode: t(`renderers.${presentation.renderer}`) })
                    : t("messages.operationLocked"),
                });
              }
            }}
            onRendererStateChange={({ mode, actualBackend: backend, availableModes: modes }) => {
              setRenderMode(mode);
              setActualBackend(backend);
              setAvailableModes(modes);
            }}
            onRenderComplete={(mode) => {
              setLoading(false);
              setRendererSwitching(false);
              setBlockingTask(null);
              setError(null);
              setRenderMode(mode);
              setStatus({ tone: "success", text: t("status.rendered") });
            }}
            onRenderError={(message) => {
              setLoading(false);
              setExporting(false);
              setRendererSwitching(false);
              setBlockingTask(null);
              setError(t("errors.render", { message }));
              setStatus({ tone: "error", text: t("status.ready") });
            }}
          />
        ) : null}

        {/* Sidebar toggle */}
        <Box className="absolute left-3 top-3 z-40 hidden lg:block">
          <Tooltip
            label={sidebarOpen ? t("toolbar.hideSidebar") : t("toolbar.showSidebar")}
            position="right"
          >
            <ActionIcon
              size="lg"
              radius="md"
              variant="light"
              color="gray"
              className={cn(
                "border shadow-md cursor-pointer",
                "hover:scale-105 transition-transform duration-200",
              )}
              style={{
                backgroundColor: "var(--mantine-color-body)",
                borderColor: "rgb(var(--app-border))",
              }}
              onClick={() => setSidebarOpen((v) => !v)}
            >
              {sidebarOpen ? (
                <IconLayoutSidebarLeftCollapse size={18} />
              ) : (
                <IconLayoutSidebarLeftExpand size={18} />
              )}
            </ActionIcon>
          </Tooltip>
        </Box>

        {/* Map stats */}
        <MapStats mapData={mapData} />

        {/* Status bar */}
        {presentation ? (
          <StatusBar
            status={status}
            error={error}
            renderMode={renderMode}
            actualBackend={actualBackend}
            availableModes={availableModes}
            loading={blockingBusy}
            rendererPreference={presentation.renderer}
            onRendererChange={(mode) => updatePresentation("renderer", mode)}
          />
        ) : null}

        {/* Layer control */}
        {presentation ? (
          <LayerControl
            presentation={presentation}
            onLayerChange={updateLayer}
          />
        ) : null}

        {/* Top toolbar */}
        <TopToolbar
          language={language}
          canExport={Boolean(mapData)}
          onImport={() => fileInputRef.current?.click()}
          onExport={handleExport}
          onFitView={() => rendererRef.current?.fitToScreen()}
          onResetView={() => rendererRef.current?.resetView()}
          onLanguageChange={setLanguage}
          onToggleTheme={toggleTheme}
        />

        {/* Mobile FAB */}
        <Box className="absolute bottom-6 right-6 z-40 lg:hidden">
          <ActionIcon
            size="xl"
            radius="xl"
            variant="light"
            color="gray"
            className={cn(
              "border shadow-md cursor-pointer",
              "hover:scale-110 active:scale-95 transition-transform duration-200",
            )}
            style={{
              backgroundColor: "var(--mantine-color-body)",
              borderColor: "rgb(var(--app-border))",
            }}
            onClick={openMobilePanel}
            aria-label={t("helpers.mobilePanel")}
          >
            <IconAdjustmentsHorizontal size={20} />
          </ActionIcon>
        </Box>
      </Box>

      {/* Mobile drawer */}
      <Drawer
        opened={mobilePanelOpened}
        onClose={closeMobilePanel}
        title={t("sections.map")}
        padding="sm"
        size="min(100vw, 32rem)"
        hiddenFrom="lg"
      >
        {sidebar}
      </Drawer>

      {/* Loading overlay */}
      <LoadingOverlay
        visible={blockingBusy}
        message={blockingTask?.message ?? status.text}
        detail={blockingTask?.detail ?? t("messages.operationLocked")}
      />
    </Box>
  );
}

export default function App() {
  return (
    <MantineProvider
      defaultColorScheme="auto"
      colorSchemeManager={colorSchemeManager}
      theme={theme}
      cssVariablesResolver={cssVariablesResolver}
    >
      <AppContent />
    </MantineProvider>
  );
}
