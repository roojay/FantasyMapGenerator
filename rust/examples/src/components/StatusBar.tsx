import { Box, Menu, Text } from "@mantine/core";
import { IconChevronDown } from "@tabler/icons-react";
import { useTranslation } from "react-i18next";

import { cn } from "@/lib/cn";
import type { RenderBackend, RendererPreference, StatusMessage } from "@/types/map";

interface StatusBarProps {
  status: StatusMessage;
  error: string | null;
  renderMode: RenderBackend | null;
  availableModes: RenderBackend[];
  loading: boolean;
  onRendererChange: (mode: RendererPreference) => void;
}

const statusColors = {
  neutral: {
    bg: "rgb(251, 191, 36)",
    text: "rgb(120, 53, 15)",
    border: "rgb(251, 191, 36)"
  },
  success: {
    bg: "rgb(34, 197, 94)",
    text: "rgb(255, 255, 255)",
    border: "rgb(34, 197, 94)"
  },
  info: {
    bg: "rgb(59, 130, 246)",
    text: "rgb(255, 255, 255)",
    border: "rgb(59, 130, 246)"
  },
  error: {
    bg: "rgb(239, 68, 68)",
    text: "rgb(255, 255, 255)",
    border: "rgb(239, 68, 68)"
  }
};

export function StatusBar({ status, error, renderMode, availableModes, loading, onRendererChange }: StatusBarProps) {
  const { t } = useTranslation();
  const colors = statusColors[status.tone];

  const glassStyle: React.CSSProperties = {
    backgroundColor: "var(--mantine-color-body)",
    borderColor: "rgb(var(--app-border))",
    backdropFilter: "blur(12px)"
  };

  const rendererOptions = (["auto", "webgpu", "webgl", "canvas", "svg"] as RendererPreference[]).map((mode) => ({
    value: mode,
    label: t(`renderers.${mode}`),
    disabled: mode !== "auto" && !availableModes.includes(mode as RenderBackend)
  }));

  return (
    <Box className="pointer-events-none absolute bottom-4 left-4 z-30 lg:bottom-6 lg:left-6">
      <Box
        className={cn(
          "pointer-events-auto rounded-lg border px-3 py-2 shadow-md",
          "backdrop-blur-xl max-w-[calc(100vw-8rem)] sm:max-w-none"
        )}
        style={glassStyle}
      >
        <Box className="grid auto-cols-max grid-flow-col items-center gap-2">
          {/* Status dot with breathing animation */}
          <Box
            className={cn(
              "h-2 w-2 rounded-full shrink-0",
              loading && "status-breathing"
            )}
            style={{
              backgroundColor: colors.bg,
              boxShadow: `0 0 8px ${colors.bg}`
            }}
          />
          
          {/* Status text */}
          <Text size="xs" className="whitespace-nowrap truncate max-w-[120px] sm:max-w-none">
            {status.text}
          </Text>

          {/* Renderer mode with dropdown */}
          {renderMode && (
            <>
              <Box
                className="h-3 w-px shrink-0"
                style={{ backgroundColor: "rgba(var(--app-border), 0.6)" }}
              />
              <Menu shadow="md" width={160}>
                <Menu.Target>
                  <Box className="grid auto-cols-max grid-flow-col items-center gap-1 cursor-pointer hover:opacity-80 transition-opacity">
                    <Text size="xs" className="uppercase tabular-nums">
                      {renderMode}
                    </Text>
                    <IconChevronDown size={12} />
                  </Box>
                </Menu.Target>
                <Menu.Dropdown>
                  <Menu.Label>{t("fields.renderer")}</Menu.Label>
                  {rendererOptions.map((option) => (
                    <Menu.Item
                      key={option.value}
                      disabled={option.disabled}
                      className="cursor-pointer"
                      onClick={() => onRendererChange(option.value)}
                    >
                      {option.label}
                    </Menu.Item>
                  ))}
                </Menu.Dropdown>
              </Menu>
            </>
          )}
        </Box>
      </Box>

      {/* Error message - separate box below on mobile */}
      {error && (
        <Box
          className="pointer-events-auto rounded-lg border px-3 py-2 shadow-md backdrop-blur-xl mt-2"
          style={{
            backgroundColor: "rgba(239, 68, 68, 0.1)",
            borderColor: "rgb(239, 68, 68)"
          }}
        >
          <Text c="red" size="xs" className="max-w-[calc(100vw-8rem)] truncate sm:max-w-none">
            {error}
          </Text>
        </Box>
      )}
    </Box>
  );
}
