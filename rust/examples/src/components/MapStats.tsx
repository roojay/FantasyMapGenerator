import { Box, Text } from "@mantine/core";
import { useTranslation } from "react-i18next";

import { cn } from "@/lib/cn";
import type { MapScenePacket } from "@/types/map";

interface MapStatsProps {
  mapData: MapScenePacket | null;
}

export function MapStats({ mapData }: MapStatsProps) {
  const { t } = useTranslation();

  if (!mapData) return null;

  const cityCount = mapData.metadata.cityCount;
  const townCount = mapData.metadata.townCount;
  const riverCount = mapData.metadata.riverCount;
  const territoryCount = mapData.metadata.territoryCount;

  const stats = [
    { label: t("stats.cities"), value: cityCount },
    { label: t("stats.towns"), value: townCount },
    { label: t("stats.rivers"), value: riverCount },
    { label: t("stats.territories"), value: territoryCount },
  ];

  return (
    <Box className="pointer-events-none absolute left-1/2 top-[calc(var(--app-safe-top)+4.25rem)] z-30 w-[calc(100%-1rem)] -translate-x-1/2 sm:top-4 sm:w-auto lg:top-6">
      <Box
        className={cn(
          "pointer-events-auto rounded-lg border px-3 py-2 shadow-md",
          "w-full sm:w-auto",
          "backdrop-blur-xl",
        )}
        style={{
          backgroundColor: "var(--mantine-color-body)",
          borderColor: "rgb(var(--app-border))",
        }}
      >
        <Box className="grid grid-cols-2 gap-x-4 gap-y-2 sm:auto-cols-max sm:grid-flow-col sm:items-center sm:gap-7">
          {stats.map((stat) => (
            <Box key={stat.label} className="min-w-0">
              <Box className="flex items-end gap-1.5">
                <Text size="xs" c="dimmed" className="truncate leading-none sm:whitespace-nowrap">
                  {stat.label}
                </Text>
                <Text
                  size="sm"
                  fw={700}
                  className="tabular-nums shrink-0 leading-none"
                  style={{ color: "rgb(var(--app-accent))" }}
                >
                  {stat.value}
                </Text>
              </Box>
            </Box>
          ))}
        </Box>
      </Box>
    </Box>
  );
}
