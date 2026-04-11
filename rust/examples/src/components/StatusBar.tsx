import { Box, Menu, Text } from "@mantine/core";
import { IconCheck, IconChevronDown } from "@tabler/icons-react";
import { useTranslation } from "react-i18next";

import { cn } from "@/lib/cn";
import type {
  RenderBackend,
  RendererPreference,
  RendererRuntimeBackend,
  StatusMessage,
} from "@/types/map";

interface StatusBarProps {
  status: StatusMessage;
  error: string | null;
  renderMode: RenderBackend | null;
  actualBackend: RendererRuntimeBackend;
  availableModes: RenderBackend[];
  loading: boolean;
  rendererPreference: RendererPreference;
  onRendererChange: (mode: RendererPreference) => void;
}

const statusColors = {
  neutral: {
    bg: "rgb(251, 191, 36)",
    text: "rgb(120, 53, 15)",
    border: "rgb(251, 191, 36)",
  },
  success: {
    bg: "rgb(34, 197, 94)",
    text: "rgb(255, 255, 255)",
    border: "rgb(34, 197, 94)",
  },
  info: {
    bg: "rgb(59, 130, 246)",
    text: "rgb(255, 255, 255)",
    border: "rgb(59, 130, 246)",
  },
  error: {
    bg: "rgb(239, 68, 68)",
    text: "rgb(255, 255, 255)",
    border: "rgb(239, 68, 68)",
  },
};

export function StatusBar({
  status,
  error,
  renderMode,
  actualBackend,
  availableModes,
  loading,
  rendererPreference,
  onRendererChange,
}: StatusBarProps) {
  const { t } = useTranslation();
  const colors = statusColors[status.tone];
  const activeRendererOption: RendererPreference =
    rendererPreference === "svg" || actualBackend === "svg"
      ? "svg"
      : rendererPreference === "webgpu" || actualBackend === "webgpu" || actualBackend === "webgl2"
        ? "webgpu"
        : "auto";

  const glassStyle: React.CSSProperties = {
    backgroundColor: "color-mix(in srgb, var(--mantine-color-body) 88%, transparent)",
    borderColor: "rgb(var(--app-border))",
    backdropFilter: "blur(16px)",
  };

  const rendererOptions = (["auto", "webgpu", "svg"] as RendererPreference[]).map((mode) => ({
    value: mode,
    label: t(`renderers.${mode}`),
    disabled: mode !== "auto" && !availableModes.includes(mode as RenderBackend),
  }));

  return (
    <Box className="pointer-events-none absolute right-2 top-[calc(var(--app-safe-top)+4.4rem)] z-30 w-[min(calc(100%-1rem),21rem)] sm:right-4 sm:top-[calc(var(--app-safe-top)+4.9rem)] sm:w-[min(calc(100%-2rem),22rem)] lg:right-6 lg:top-[calc(var(--app-safe-top)+5.6rem)] lg:w-[22rem]">
      <Box
        className={cn(
          "pointer-events-auto w-full rounded-2xl border px-3.5 py-2.5 shadow-lg",
          "backdrop-blur-xl",
        )}
        style={glassStyle}
      >
        <Box className="flex min-w-0 items-center gap-2.5">
          <Box
            className={cn("h-2.5 w-2.5 rounded-full shrink-0", loading && "status-breathing")}
            style={{
              backgroundColor: colors.bg,
              boxShadow: `0 0 10px ${colors.bg}`,
            }}
          />

          {renderMode && (
            <Menu shadow="md" width={170} position="bottom-end">
              <Menu.Target>
                <Box
                  className={cn(
                    "grid auto-cols-max grid-flow-col items-center gap-1 rounded-full border px-2 py-1",
                    "cursor-pointer transition-colors hover:bg-black/4 dark:hover:bg-white/6",
                  )}
                  style={{
                    borderColor: "rgba(var(--app-border), 0.75)",
                    backgroundColor: "rgba(var(--app-accent), 0.08)",
                  }}
                >
                  <Text size="10px" fw={700} className="uppercase tabular-nums tracking-[0.18em]">
                    {renderMode}
                  </Text>
                  <IconChevronDown size={11} />
                </Box>
              </Menu.Target>
              <Menu.Dropdown>
                <Menu.Label>{t("fields.renderer")}</Menu.Label>
                {rendererOptions.map((option) => (
                  <Menu.Item
                    key={option.value}
                    disabled={option.disabled}
                    className="cursor-pointer"
                    leftSection={
                      <Box className="grid h-3.5 w-3.5 place-items-center">
                        {option.value === activeRendererOption ? (
                          <IconCheck size={12} stroke={2.5} />
                        ) : null}
                      </Box>
                    }
                    rightSection={
                      option.value === "webgpu" && actualBackend === "webgl2" ? (
                        <Text size="10px" className="uppercase tracking-[0.16em] opacity-65">
                          WebGL2
                        </Text>
                      ) : undefined
                    }
                    onClick={() => onRendererChange(option.value)}
                  >
                    {option.label}
                  </Menu.Item>
                ))}
              </Menu.Dropdown>
            </Menu>
          )}

          <Box
            className="h-4 w-px shrink-0"
            style={{ backgroundColor: "rgba(var(--app-border), 0.6)" }}
          />

          <Text size="sm" fw={500} className="truncate text-balance">
            {status.text}
          </Text>
        </Box>
      </Box>

      {error && (
        <Box
          className={cn(
            "pointer-events-auto mt-2 w-full rounded-2xl border px-3.5 py-2.5 shadow-lg backdrop-blur-xl",
          )}
          style={{
            backgroundColor:
              "color-mix(in srgb, rgb(var(--app-danger-bg)) 88%, var(--mantine-color-body))",
            borderColor: "rgb(var(--app-danger-border))",
          }}
        >
          <Text c="red" size="xs" fw={500} className="break-words leading-relaxed">
            {error}
          </Text>
        </Box>
      )}
    </Box>
  );
}
