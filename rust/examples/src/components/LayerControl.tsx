import { Box, Checkbox } from "@mantine/core";
import { useTranslation } from "react-i18next";

import { cn } from "@/lib/cn";
import type { MapPresentationPreset } from "@/types/map";

interface LayerControlProps {
  presentation: MapPresentationPreset;
  onLayerChange: (key: keyof MapPresentationPreset["layers"], value: boolean) => void;
}

const LAYER_KEYS = ["slope", "river", "contour", "border", "city", "town", "label"] as const;

export function LayerControl({ presentation, onLayerChange }: LayerControlProps) {
  const { t } = useTranslation();
  const layerKeys: ReadonlyArray<keyof MapPresentationPreset["layers"]> = LAYER_KEYS;

  const renderCheckbox = (key: keyof MapPresentationPreset["layers"]) => (
    <Checkbox
      key={key}
      size="xs"
      label={t(`layers.${key}`)}
      checked={presentation.layers[key]}
      onChange={(event) => onLayerChange(key, event.currentTarget.checked)}
      classNames={{
        root: cn(
          "rounded-md px-1.5 py-1 -mx-1.5 -my-1 cursor-pointer",
          "layer-toggle-hover",
          "transition-colors duration-200",
        ),
        label: cn("cursor-pointer select-none text-xs", "whitespace-nowrap"),
        input: "cursor-pointer",
      }}
    />
  );

  return (
    <Box className="pointer-events-none absolute bottom-4 left-1/2 z-30 w-[calc(100%-2rem)] max-w-2xl -translate-x-1/2 lg:bottom-6 lg:w-auto">
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
        <Box className="hidden sm:grid sm:auto-cols-max sm:grid-flow-col sm:items-center sm:justify-center sm:gap-4">
          {layerKeys.map(renderCheckbox)}
        </Box>

        <Box className="block sm:hidden">
          <Box className="grid grid-cols-2 gap-x-4 gap-y-2">{layerKeys.map(renderCheckbox)}</Box>
        </Box>
      </Box>
    </Box>
  );
}
