import type { AppLanguage } from "@/types/map";

export const APP_LANGUAGE_STORAGE_KEY = "fantasy-map-language";

const APP_LANGUAGES = new Set<AppLanguage>(["zh-CN", "en"]);

export function isAppLanguage(value: unknown): value is AppLanguage {
  return typeof value === "string" && APP_LANGUAGES.has(value as AppLanguage);
}

export function parseStoredAppLanguage(value: string | null): AppLanguage | null {
  if (value === null) {
    return null;
  }

  if (isAppLanguage(value)) {
    return value;
  }

  try {
    const parsed: unknown = JSON.parse(value);
    return isAppLanguage(parsed) ? parsed : null;
  } catch {
    return null;
  }
}

export function getBrowserLanguage(): AppLanguage {
  if (typeof navigator === "undefined") {
    return "en";
  }

  return navigator.language.toLowerCase().startsWith("zh") ? "zh-CN" : "en";
}

export function getStoredAppLanguage(): AppLanguage | null {
  if (typeof window === "undefined") {
    return null;
  }

  return parseStoredAppLanguage(window.localStorage.getItem(APP_LANGUAGE_STORAGE_KEY));
}

export function getInitialAppLanguage(): AppLanguage {
  return getStoredAppLanguage() ?? getBrowserLanguage();
}

export function deserializeAppLanguage(value: string | undefined): AppLanguage {
  return parseStoredAppLanguage(value ?? null) ?? getBrowserLanguage();
}

export function serializeAppLanguage(value: AppLanguage): string {
  return value;
}
