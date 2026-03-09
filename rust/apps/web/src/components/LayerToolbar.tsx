import { Group, Switch, Text, Paper } from '@mantine/core';
import { getT } from '../i18n';
import type { LayerVisibility, Language } from '../types/map';

interface LayerToolbarProps {
  layers: LayerVisibility;
  onToggleLayer: (layer: keyof LayerVisibility) => void;
  language: Language;
}

export function LayerToolbar({ layers, onToggleLayer, language }: LayerToolbarProps) {
  const t = getT(language);
  const layerKeys = Object.keys(layers) as (keyof LayerVisibility)[];

  return (
    <Paper
      shadow="sm"
      className="fixed bottom-4 left-1/2 -translate-x-1/2 z-10 px-4 py-2 rounded-full"
      style={{ maxWidth: '95vw', overflowX: 'auto' }}
    >
      <Group gap="md" wrap="nowrap">
        <Text size="xs" fw={600} c="dimmed" className="hidden sm:inline">
          {t.layers.title}:
        </Text>
        {layerKeys.map(key => (
          <Switch
            key={key}
            label={<span className="text-xs">{t.layers[key]}</span>}
            checked={layers[key]}
            onChange={() => onToggleLayer(key)}
            size="xs"
          />
        ))}
      </Group>
    </Paper>
  );
}
