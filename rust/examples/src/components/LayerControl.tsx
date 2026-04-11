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
          "rounded-md px-2 py-1.5 -mx-1.5 -my-1 cursor-pointer",
          "layer-toggle-hover",
          "transition-colors duration-200",
        ),
        label: cn(
          "cursor-pointer select-none text-xs leading-tight",
          "max-w-[7.5rem] break-words sm:max-w-none sm:whitespace-nowrap",
        ),
        input: "cursor-pointer",
      }}
    />
  );

  return (
    <Box className="pointer-events-none absolute bottom-[max(0.75rem,var(--app-safe-bottom))] left-1/2 z-30 w-[calc(100%-1rem)] max-w-3xl -translate-x-1/2 sm:w-[calc(100%-2rem)] md:w-auto md:max-w-[calc(100%-2rem)] lg:bottom-6">
      <Box
        className={cn(
          "pointer-events-auto rounded-lg border px-3 py-2 shadow-md",
          "pr-14 sm:pr-3 md:pr-3",
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
          <Box className="grid grid-cols-2 gap-x-3 gap-y-1.5">{layerKeys.map(renderCheckbox)}</Box>
        </Box>
      </Box>
    </Box>
  );
}
