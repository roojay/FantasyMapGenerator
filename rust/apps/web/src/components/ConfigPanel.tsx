import { useState } from 'react';
import {
  NumberInput,
  Button,
  Stack,
  Text,
  Slider,
  Drawer,
  ActionIcon,
  Tooltip,
} from '@mantine/core';
import { getT } from '../i18n';
import type { MapConfig, Language } from '../types/map';

interface ConfigPanelProps {
  config: MapConfig;
  onConfigChange: (config: MapConfig) => void;
  onGenerate: () => void;
  isGenerating: boolean;
  language: Language;
}

function ConfigForm({
  config,
  onConfigChange,
  onGenerate,
  isGenerating,
  language,
}: ConfigPanelProps) {
  const t = getT(language);
  const [local, setLocal] = useState<MapConfig>(config);

  const update = <K extends keyof MapConfig>(key: K, value: MapConfig[K]) => {
    const updated = { ...local, [key]: value };
    setLocal(updated);
    onConfigChange(updated);
  };

  return (
    <Stack gap="md" p="md">
      <Text fw={600}>{t.config.title}</Text>

      <NumberInput
        label={t.config.seed}
        value={local.seed}
        onChange={v => update('seed', Number(v) || 0)}
        min={0}
        max={999999}
      />

      <div>
        <Text size="sm" fw={500} mb={4}>
          {t.config.resolution}
        </Text>
        <Slider
          value={local.resolution}
          onChange={v => update('resolution', v)}
          min={0.04}
          max={0.15}
          step={0.01}
          marks={[
            { value: 0.04, label: 'Fine' },
            { value: 0.08, label: 'Med' },
            { value: 0.15, label: 'Coarse' },
          ]}
        />
      </div>

      <NumberInput
        label={t.config.cities}
        value={local.cities}
        onChange={v => update('cities', Number(v) || 1)}
        min={1}
        max={20}
      />

      <NumberInput
        label={t.config.towns}
        value={local.towns}
        onChange={v => update('towns', Number(v) || 1)}
        min={1}
        max={50}
      />

      <NumberInput
        label={t.config.erosionSteps}
        value={local.erosionSteps}
        onChange={v => update('erosionSteps', Number(v) || 1)}
        min={1}
        max={10}
      />

      <Button fullWidth onClick={onGenerate} loading={isGenerating} color="blue">
        {isGenerating ? t.config.generating : t.config.generate}
      </Button>
    </Stack>
  );
}

export function ConfigPanel(props: ConfigPanelProps) {
  const [drawerOpen, setDrawerOpen] = useState(false);
  const t = getT(props.language);

  return (
    <>
      {/* Desktop: always visible sidebar */}
      <aside className="hidden md:flex flex-col w-64 bg-white dark:bg-gray-900 border-r border-gray-200 dark:border-gray-700 overflow-y-auto">
        <ConfigForm {...props} />
      </aside>

      {/* Mobile: drawer */}
      <div className="md:hidden">
        <Tooltip label={t.config.openConfig}>
          <ActionIcon
            variant="filled"
            color="blue"
            size="lg"
            className="fixed left-4 bottom-20 z-10 shadow-lg"
            onClick={() => setDrawerOpen(true)}
            aria-label={t.config.openConfig}
          >
            ⚙️
          </ActionIcon>
        </Tooltip>
        <Drawer
          opened={drawerOpen}
          onClose={() => setDrawerOpen(false)}
          title={t.config.title}
          position="left"
          size="xs"
        >
          <ConfigForm {...props} />
        </Drawer>
      </div>
    </>
  );
}
