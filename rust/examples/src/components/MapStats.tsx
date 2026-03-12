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
    <Box className="pointer-events-none absolute left-1/2 top-4 z-30 -translate-x-1/2 lg:top-6">
      <Box
        className={cn(
          "pointer-events-auto rounded-lg border px-3 py-2 shadow-md",
          "backdrop-blur-xl",
        )}
        style={{
          backgroundColor: "var(--mantine-color-body)",
          borderColor: "rgb(var(--app-border))",
        }}
      >
        <Box className="grid auto-cols-max grid-flow-col items-center gap-7">
          {stats.map((stat, index) => (
            <Box key={stat.label} className="relative">
              {index > 0 && (
                <Box
                  className="absolute -left-[0.875rem] top-1/2 h-4 w-px -translate-y-1/2"
                  style={{ backgroundColor: "rgba(var(--app-border), 0.6)" }}
                />
              )}
              <Box className="flex items-end gap-1.5">
                <Text size="xs" c="dimmed" className="whitespace-nowrap leading-none">
                  {stat.label}
                </Text>
                <Text
                  size="sm"
                  fw={700}
                  className="tabular-nums leading-none"
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
