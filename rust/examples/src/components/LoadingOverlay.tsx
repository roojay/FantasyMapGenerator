import { Box, Text } from "@mantine/core";
import { useTranslation } from "react-i18next";

import { cn } from "@/lib/cn";

interface LoadingOverlayProps {
  visible: boolean;
  message?: string;
}

export function LoadingOverlay({ visible, message }: LoadingOverlayProps) {
  const { t } = useTranslation();

  if (!visible) return null;

  return (
    <Box
      className={cn(
        "fixed inset-0 z-50 grid place-items-center",
        "loading-overlay"
      )}
      style={{
        backgroundColor: "rgba(var(--app-bg), 0.8)",
        backdropFilter: "blur(8px)"
      }}
    >
      <Box
        className={cn(
          "rounded-lg border px-6 py-5 shadow-lg",
          "backdrop-blur-xl"
        )}
        style={{
          backgroundColor: "var(--mantine-color-body)",
          borderColor: "rgb(var(--app-border))"
        }}
      >
        <Box className="grid place-items-center gap-4">
          {/* Spinner */}
          <Box
            className="loading-spinner h-10 w-10 rounded-full border-4"
            style={{
              borderColor: "rgba(var(--app-accent), 0.2)",
              borderTopColor: "rgb(var(--app-accent))"
            }}
          />
          
          {/* Message */}
          <Text size="sm" fw={500} className="text-center">
            {message || t("status.generating")}
          </Text>
        </Box>
      </Box>
    </Box>
  );
}
