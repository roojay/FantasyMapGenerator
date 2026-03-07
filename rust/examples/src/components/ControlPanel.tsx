import {
  Accordion,
  ActionIcon,
  Box,
  Button,
  Group,
  NumberInput,
  Slider,
  Stack,
  Text,
  Tooltip
} from "@mantine/core";
import { IconRefresh } from "@tabler/icons-react";
import { useTranslation } from "react-i18next";

import { cn } from "@/lib/cn";
import type { MapConfig } from "@/types/map";

interface ControlPanelProps {
  config: MapConfig;
  isBusy: boolean;
  onConfigChange: <Key extends keyof MapConfig>(key: Key, value: MapConfig[Key]) => void;
  onGenerate: () => void;
}

const presets = [
  { label: "1080p", width: 1920, height: 1080 },
  { label: "1440p", width: 2560, height: 1440 },
  { label: "4K", width: 3840, height: 2160 },
  { label: "8K", width: 7680, height: 4320 }
];

export function ControlPanel({
  config,
  isBusy,
  onConfigChange,
  onGenerate
}: ControlPanelProps) {
  const { t } = useTranslation();

  return (
    <Box className="grid h-full grid-rows-[auto_1fr_auto]" style={{ backgroundColor: "var(--mantine-color-body)" }}>
      {/* Header */}
      <Box className="border-b px-4 py-4" style={{ borderColor: "rgb(var(--app-border))" }}>
        <Text fw={700} size="lg">
          {t("app.title")}
        </Text>
        <Text c="dimmed" size="xs">
          {t("app.subtitle")}
        </Text>
      </Box>

      {/* Scrollable controls */}
      <Box className="overflow-y-auto px-3 py-3 sidebar-scroll">
        <Accordion
          defaultValue={["map", "locations", "rendering"]}
          multiple
          variant="separated"
          classNames={{
            item: cn(
              "border-b border-[rgb(var(--app-border))]",
              "rounded-lg overflow-hidden"
            ),
            control: cn(
              "hover:bg-[var(--mantine-color-gray-1)] dark:hover:bg-[var(--mantine-color-gray-8)]",
              "transition-colors duration-200"
            ),
            label: "text-xs font-semibold uppercase tracking-wider",
            chevron: "text-[rgb(var(--app-muted))]"
          }}
        >
          <Accordion.Item value="map">
            <Accordion.Control>{t("sections.map")}</Accordion.Control>
            <Accordion.Panel>
              <Stack gap="md">
                <div>
                  <Group justify="space-between" mb={6}>
                    <Text size="sm" fw={600}>
                      {t("fields.seed")}
                    </Text>
                    <Tooltip label={t("helpers.seedHint")}>
                      <ActionIcon
                        variant="subtle"
                        color="gray"
                        size="sm"
                        className="cursor-pointer"
                        onClick={() => onConfigChange("seed", Math.floor(Math.random() * 999999) + 1)}
                      >
                        <IconRefresh size={14} />
                      </ActionIcon>
                    </Tooltip>
                  </Group>
                  <NumberInput
                    size="xs"
                    value={config.seed}
                    min={0}
                    max={999999}
                    clampBehavior="strict"
                    classNames={{
                      input: cn(
                        "bg-white dark:bg-gray-800",
                        "border-gray-300 dark:border-gray-600",
                        "focus:border-brand-5 focus:ring-2 focus:ring-brand-5/20",
                        "transition-all duration-200"
                      )
                    }}
                    onChange={(value) => onConfigChange("seed", Number(value) || 0)}
                  />
                </div>

                <div>
                  <Text size="sm" fw={600} mb={6}>
                    {t("fields.size")}
                  </Text>
                  <div className="grid grid-cols-2 gap-2">
                    <NumberInput
                      size="xs"
                      value={config.width}
                      min={640}
                      max={16384}
                      onChange={(value) => onConfigChange("width", Number(value) || 1920)}
                    />
                    <NumberInput
                      size="xs"
                      value={config.height}
                      min={480}
                      max={16384}
                      onChange={(value) => onConfigChange("height", Number(value) || 1080)}
                    />
                  </div>
                  <div className="mt-2 grid grid-cols-4 gap-2">
                    {presets.map((preset) => (
                      <Button
                        key={preset.label}
                        size="xs"
                        variant="light"
                        color="gray"
                        className="cursor-pointer"
                        classNames={{
                          root: cn(
                            "hover:scale-105 active:scale-95",
                            "transition-transform duration-200"
                          )
                        }}
                        onClick={() => {
                          onConfigChange("width", preset.width);
                          onConfigChange("height", preset.height);
                        }}
                      >
                        {preset.label}
                      </Button>
                    ))}
                  </div>
                </div>

                <div>
                  <Group justify="space-between" mb={6}>
                    <Text size="sm" fw={600}>
                      {t("fields.detail")}
                    </Text>
                    <ActionIcon
                      variant="subtle"
                      color="gray"
                      size="sm"
                      className="cursor-pointer"
                      onClick={() => onConfigChange("resolution", Number((Math.random() * 0.19 + 0.01).toFixed(2)))}
                    >
                      <IconRefresh size={14} />
                    </ActionIcon>
                  </Group>
                  <NumberInput
                    size="xs"
                    value={config.resolution}
                    min={0.01}
                    max={0.2}
                    step={0.01}
                    decimalScale={2}
                    fixedDecimalScale
                    description={t("helpers.detailHint")}
                    onChange={(value) => onConfigChange("resolution", Number(value) || 0.08)}
                  />
                </div>
              </Stack>
            </Accordion.Panel>
          </Accordion.Item>

          <Accordion.Item value="locations">
            <Accordion.Control>{t("sections.locations")}</Accordion.Control>
            <Accordion.Panel>
              <Stack gap="lg">
                <div>
                  <Group justify="space-between" mb={6}>
                    <Text size="xs" fw={600}>
                      {t("fields.cities")}
                    </Text>
                    <Text c="dimmed" size="xs" className="tabular-nums">
                      {config.cities}
                    </Text>
                  </Group>
                  <Slider min={0} max={20} value={config.cities} onChange={(value) => onConfigChange("cities", value)} />
                </div>

                <div>
                  <Group justify="space-between" mb={6}>
                    <Text size="xs" fw={600}>
                      {t("fields.towns")}
                    </Text>
                    <Text c="dimmed" size="xs" className="tabular-nums">
                      {config.towns}
                    </Text>
                  </Group>
                  <Slider min={0} max={50} value={config.towns} onChange={(value) => onConfigChange("towns", value)} />
                </div>
              </Stack>
            </Accordion.Panel>
          </Accordion.Item>

          <Accordion.Item value="rendering">
            <Accordion.Control>{t("sections.rendering")}</Accordion.Control>
            <Accordion.Panel>
              <Stack gap="md">
                <div>
                  <Group justify="space-between" mb={6}>
                    <Text size="xs" fw={600}>
                      {t("fields.lineScale")}
                    </Text>
                    <Text c="dimmed" size="xs" className="tabular-nums">
                      {config.drawScale.toFixed(1)}
                    </Text>
                  </Group>
                  <Slider
                    min={0.5}
                    max={3}
                    step={0.1}
                    value={config.drawScale}
                    onChange={(value) => onConfigChange("drawScale", value)}
                  />
                </div>
              </Stack>
            </Accordion.Panel>
          </Accordion.Item>
        </Accordion>
      </Box>

      {/* Action button */}
      <Box className="px-3 pb-6 pt-3">
        <Button
          size="xss"
          fullWidth
          loading={isBusy}
          className="cursor-pointer"
          classNames={{
            root: cn(
              "shadow-md hover:shadow-lg",
              "hover:scale-[1.02] active:scale-[0.98]",
              "transition-all duration-200"
            )
          }}
          onClick={() => onGenerate()}
        >
          {t("actions.generate")}
        </Button>
      </Box>
    </Box>
  );
}