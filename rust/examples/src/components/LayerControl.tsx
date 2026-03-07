import { Box, Checkbox, Group } from "@mantine/core";
import { useTranslation } from "react-i18next";

import { cn } from "@/lib/cn";
import type { MapLayers } from "@/types/map";

interface LayerControlProps {
  layers: MapLayers;
  onLayerChange: <Key extends keyof MapLayers>(key: Key, value: MapLayers[Key]) => void;
}

export function LayerControl({ layers, onLayerChange }: LayerControlProps) {
  const { t } = useTranslation();

  const layerKeys = Object.keys(layers) as Array<keyof MapLayers>;

  const renderCheckbox = (key: keyof MapLayers) => (
    <Checkbox
      key={key}
      size="xs"
      label={t(`layers.${key}`)}
      checked={layers[key]}
      onChange={(event) => onLayerChange(key, event.currentTarget.checked)}
      classNames={{
        root: cn(
          "rounded-md px-1.5 py-1 -mx-1.5 -my-1 cursor-pointer",
          "hover:bg-[rgba(var(--app-accent),0.08)] dark:hover:bg-[rgba(var(--app-accent),0.12)]",
          "transition-colors duration-200"
        ),
        label: cn(
          "cursor-pointer select-none text-xs",
          "whitespace-nowrap"
        ),
        input: "cursor-pointer"
      }}
    />
  );

  return (
    <Box className="pointer-events-none absolute bottom-4 left-1/2 z-30 w-[calc(100%-2rem)] max-w-2xl -translate-x-1/2 lg:bottom-6 lg:w-auto">
      <Box
        className={cn(
          "pointer-events-auto rounded-lg border px-3 py-2 shadow-md",
          "backdrop-blur-xl"
        )}
        style={{
          backgroundColor: "var(--mantine-color-body)",
          borderColor: "rgb(var(--app-border))"
        }}
      >
        {/* Desktop: single row - hidden on mobile */}
        <Box className="hidden sm:grid sm:auto-cols-max sm:grid-flow-col sm:items-center sm:justify-center sm:gap-4">
          {layerKeys.map(renderCheckbox)}
        </Box>

        {/* Mobile: grid layout - hidden on desktop */}
        <Box className="block sm:hidden">
          <Box className="grid grid-cols-2 gap-x-4 gap-y-2">
            {layerKeys.map(renderCheckbox)}
          </Box>
        </Box>
      </Box>
    </Box>
  );
}
