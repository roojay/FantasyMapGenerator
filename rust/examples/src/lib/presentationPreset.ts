import type { MapLayers, MapPresentationPreset, RendererPreference } from "@/types/map";

export const PRESENTATION_PRESET_STORAGE_KEY = "fantasy-map-presentation-preset";

const VALID_RENDERER_PREFERENCES = new Set<RendererPreference>(["auto", "webgpu", "svg"]);
const LAYER_KEYS = ["slope", "river", "contour", "border", "city", "town", "label"] as const;

export function createDefaultPresentationPreset(): MapPresentationPreset {
  return {
    renderer: "auto",
    layers: {
      slope: true,
      river: true,
      contour: true,
      border: true,
      city: true,
      town: true,
      label: true,
    },
  };
}

function isPresentationLayers(value: unknown): value is MapLayers {
  if (!value || typeof value !== "object") {
    return false;
  }

  const candidate = value as Partial<Record<(typeof LAYER_KEYS)[number], unknown>>;
  for (const key of LAYER_KEYS) {
    if (typeof candidate[key] !== "boolean") {
      return false;
    }
  }
  return true;
}

function normalizeRendererPreference(value: unknown): RendererPreference | null {
  if (typeof value !== "string" || !VALID_RENDERER_PREFERENCES.has(value as RendererPreference)) {
    return null;
  }
  return value as RendererPreference;
}

function isPresentationPreset(value: unknown): value is MapPresentationPreset {
  if (!value || typeof value !== "object") {
    return false;
  }

  const candidate = value as Partial<MapPresentationPreset> & {
    svgPluginId?: unknown;
    svgPluginConfigs?: unknown;
  };
  const renderer = normalizeRendererPreference(candidate.renderer);
  return (
    !!renderer &&
    VALID_RENDERER_PREFERENCES.has(renderer) &&
    isPresentationLayers(candidate.layers)
  );
}

export function parseStoredPresentationPreset(value: string | null): MapPresentationPreset | null {
  if (value === null) {
    return null;
  }

  try {
    const parsed: unknown = JSON.parse(value);
    if (!isPresentationPreset(parsed)) {
      return null;
    }

    const candidate = parsed as MapPresentationPreset & { renderer: unknown };
    const renderer = normalizeRendererPreference(candidate.renderer);
    if (!renderer) {
      return null;
    }

    return {
      renderer,
      layers: candidate.layers,
    };
  } catch {
    return null;
  }
}

export function serializePresentationPreset(value: MapPresentationPreset): string {
  return JSON.stringify(value);
}

export function parsePresentationPresetJson(value: string): MapPresentationPreset {
  const parsed = parseStoredPresentationPreset(value);
  if (!parsed) {
    throw new Error("Invalid presentation preset JSON");
  }
  return parsed;
}

export function buildPresentationExportSlug(presentation: MapPresentationPreset): string {
  return (
    presentation.renderer.replace(/[^a-z0-9_-]+/gi, "_").replace(/^_+|_+$/g, "") || "render"
  );
}
