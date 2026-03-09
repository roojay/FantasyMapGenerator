import { Group, Text, Badge, ActionIcon, Tooltip } from '@mantine/core';
import { getT } from '../i18n';
import type { Language, ColorScheme } from '../types/map';

interface HeaderProps {
  language: Language;
  colorScheme: ColorScheme;
  seed: number;
  generationTimeMs: number | null;
  polygonCount: number;
  usingWasm: boolean;
  onToggleColorScheme: () => void;
  onToggleLanguage: () => void;
}

export function Header({
  language,
  colorScheme,
  seed,
  generationTimeMs,
  polygonCount,
  usingWasm,
  onToggleColorScheme,
  onToggleLanguage,
}: HeaderProps) {
  const t = getT(language);

  return (
    <header className="h-12 flex items-center px-4 border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900 shadow-sm">
      <Text fw={700} size="lg" className="text-blue-700 dark:text-blue-400 mr-4 hidden sm:block">
        {t.title}
      </Text>

      <Group gap="xs" className="flex-1">
        {generationTimeMs !== null && (
          <Badge variant="light" color="teal" size="sm">
            {t.header.generationTime}: {generationTimeMs}
            {t.header.ms}
          </Badge>
        )}
        {polygonCount > 0 && (
          <Badge variant="light" color="blue" size="sm" className="hidden md:inline-flex">
            {t.header.polygons}: {polygonCount.toLocaleString()}
          </Badge>
        )}
        <Badge variant="light" color="gray" size="sm" className="hidden md:inline-flex">
          {t.header.seed}: {seed}
        </Badge>
        <Badge variant="light" color="violet" size="sm" className="hidden lg:inline-flex">
          {t.header.version}: 0.1.0
        </Badge>
        <Tooltip label={usingWasm ? t.header.wasmActive : t.header.wasmFallback}>
          <Badge
            variant="filled"
            color={usingWasm ? 'green' : 'orange'}
            size="sm"
            className="hidden sm:inline-flex cursor-default"
          >
            {usingWasm ? 'WASM' : 'Static'}
          </Badge>
        </Tooltip>
      </Group>

      <Group gap="xs">
        <Tooltip label={t.lang[language === 'en' ? 'zh' : 'en']}>
          <ActionIcon
            variant="subtle"
            size="sm"
            onClick={onToggleLanguage}
            aria-label="Toggle language"
          >
            <span className="text-xs font-bold">{language.toUpperCase()}</span>
          </ActionIcon>
        </Tooltip>
        <Tooltip label={colorScheme === 'light' ? t.theme.dark : t.theme.light}>
          <ActionIcon
            variant="subtle"
            size="sm"
            onClick={onToggleColorScheme}
            aria-label="Toggle color scheme"
          >
            <span className="text-sm">{colorScheme === 'light' ? '🌙' : '☀️'}</span>
          </ActionIcon>
        </Tooltip>
      </Group>
    </header>
  );
}
