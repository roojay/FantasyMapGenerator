import { Box, Text } from "@mantine/core";
import { useTranslation } from "react-i18next";

import { cn } from "@/lib/cn";

interface LoadingOverlayProps {
  visible: boolean;
  message?: string;
  detail?: string;
}

export function LoadingOverlay({ visible, message, detail }: LoadingOverlayProps) {
  const { t } = useTranslation();

  if (!visible) return null;

  return (
    <Box
      className={cn("fixed inset-0 z-[90] overflow-hidden", "app-loading-overlay")}
      role="alert"
      aria-busy="true"
      aria-live="assertive"
    >
      <Box className="app-loading-backdrop absolute inset-0" />
      <Box className="app-loading-grid absolute inset-0 opacity-70" />
      <Box className="app-loading-beam absolute inset-y-0 -left-1/4 w-1/4" />

      <Box className="absolute inset-0 grid place-items-center px-4">
        <Box
          className={cn(
            "w-[min(92vw,30rem)] overflow-hidden rounded-[24px] border px-5 py-6 shadow-2xl sm:rounded-[28px] sm:px-7 sm:py-8",
            "app-loading-shell",
          )}
        >
          <Box className="absolute inset-x-0 top-0 h-px app-loading-edge" />
          <Box className="grid place-items-center gap-5 text-center">
            <Box className="app-loading-orb grid h-16 w-16 place-items-center rounded-full sm:h-20 sm:w-20">
              <Box className="app-loading-spinner h-9 w-9 rounded-full sm:h-11 sm:w-11" />
            </Box>

            <Box className="space-y-3">
              <Text size="lg" fw={800} lh={1.15} className="sm:text-2xl">
                {message || t("status.generating")}
              </Text>
              <Text size="sm" c="dimmed" maw={360} className="text-xs sm:text-sm">
                {detail || t("messages.operationLocked")}
              </Text>
            </Box>

            <Box className="app-loading-progress h-1.5 w-full overflow-hidden rounded-full">
              <Box className="app-loading-progress-bar h-full rounded-full" />
            </Box>

            <Text size="xs" className="app-loading-caption tracking-[0.18em] uppercase">
              {t("messages.operationLockedShort")}
            </Text>
          </Box>
        </Box>
      </Box>
    </Box>
  );
}
