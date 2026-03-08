import { ActionIcon, Box, Button, Menu, SegmentedControl, Tooltip, useMantineColorScheme } from "@mantine/core";
import {
  IconChevronDown,
  IconFileDownload,
  IconFileUpload,
  IconFocus2,
  IconMoon,
  IconRefresh,
  IconSun
} from "@tabler/icons-react";
import { useTranslation } from "react-i18next";

import { cn } from "@/lib/cn";
import type { AppLanguage } from "@/types/map";

interface TopToolbarProps {
  language: AppLanguage;
  canExport: boolean;
  onImport: () => void;
  onExport: (format: "json" | "png" | "svg") => void;
  onFitView: () => void;
  onResetView: () => void;
  onLanguageChange: (language: AppLanguage) => void;
  onToggleTheme: () => void;
}

export function TopToolbar({
  language,
  canExport,
  onImport,
  onExport,
  onFitView,
  onResetView,
  onLanguageChange,
  onToggleTheme
}: TopToolbarProps) {
  const { t } = useTranslation();
  const { colorScheme: resolvedScheme } = useMantineColorScheme();
  const isDark = resolvedScheme === "dark";

  const glassStyle: React.CSSProperties = {
    backgroundColor: "var(--mantine-color-body)",
    borderColor: "rgb(var(--app-border))"
  };

  return (
    <Box className="pointer-events-none absolute right-4 top-4 z-30 lg:right-6 lg:top-6">
      <Box
        className="pointer-events-auto grid auto-cols-max grid-flow-col items-center gap-2 rounded-lg border px-3 py-2 shadow-md backdrop-blur-xl"
        style={glassStyle}
      >
        {/* Language selector */}
        <SegmentedControl
          size="xs"
          className="shrink-0"
          classNames={{
            root: "min-w-[70px]",
            indicator: "transition-all duration-200 ease-out"
          }}
          value={language}
          onChange={(value) => onLanguageChange(value as AppLanguage)}
          data={[
            { label: "中文", value: "zh-CN" },
            { label: "EN", value: "en" }
          ]}
        />

        {/* Divider */}
        <Box
          className="h-4 w-px shrink-0"
          style={{ backgroundColor: "rgba(var(--app-border), 0.6)" }}
        />

        {/* Theme toggle */}
        <Tooltip label={t("toolbar.theme")}>
          <ActionIcon
            variant="subtle"
            color="gray"
            size="sm"
            onClick={onToggleTheme}
            className="cursor-pointer"
            classNames={{
              root: cn(
                "hover:scale-110 active:scale-90",
                "transition-transform duration-200"
              )
            }}
          >
            {isDark ? <IconSun size={14} /> : <IconMoon size={14} />}
          </ActionIcon>
        </Tooltip>

        {/* Divider */}
        <Box
          className="h-4 w-px shrink-0"
          style={{ backgroundColor: "rgba(var(--app-border), 0.6)" }}
        />

        {/* Fit view */}
        <Tooltip label={t("toolbar.fit")}>
          <ActionIcon
            variant="subtle"
            color="gray"
            size="sm"
            onClick={onFitView}
            className="cursor-pointer"
            classNames={{
              root: cn(
                "hover:scale-110 active:scale-90",
                "transition-transform duration-200"
              )
            }}
          >
            <IconFocus2 size={14} />
          </ActionIcon>
        </Tooltip>

        {/* Reset view */}
        <Tooltip label={t("toolbar.reset")}>
          <ActionIcon
            variant="subtle"
            color="gray"
            size="sm"
            onClick={onResetView}
            className="cursor-pointer"
            classNames={{
              root: cn(
                "hover:scale-110 active:scale-90",
                "transition-transform duration-200"
              )
            }}
          >
            <IconRefresh size={14} />
          </ActionIcon>
        </Tooltip>

        {/* Divider */}
        <Box
          className="h-4 w-px shrink-0"
          style={{ backgroundColor: "rgba(var(--app-border), 0.6)" }}
        />

        {/* Import button */}
        <Button
          size="xs"
          leftSection={<IconFileUpload size={14} />}
          variant="light"
          color="gray"
          onClick={onImport}
          className="cursor-pointer"
          classNames={{
            root: cn(
              "max-[479px]:!px-2",
              "hover:scale-105 active:scale-95",
              "transition-transform duration-200"
            )
          }}
        >
          <span className="max-[479px]:hidden">{t("toolbar.import")}</span>
        </Button>

        {/* Export menu */}
        <Menu shadow="lg" width={160}>
          <Menu.Target>
            <Button
              size="xs"
              rightSection={<IconChevronDown size={12} />}
              leftSection={<IconFileDownload size={14} />}
              disabled={!canExport}
              className="cursor-pointer"
              classNames={{
                root: cn(
                  "max-[479px]:!px-2",
                  "hover:scale-105 active:scale-95",
                  "transition-transform duration-200"
                )
              }}
            >
              <span className="max-[479px]:hidden">{t("toolbar.export")}</span>
            </Button>
          </Menu.Target>
          <Menu.Dropdown>
            <Menu.Item className="cursor-pointer" onClick={() => onExport("svg")}>SVG</Menu.Item>
            <Menu.Item className="cursor-pointer" onClick={() => onExport("png")}>PNG</Menu.Item>
            <Menu.Item className="cursor-pointer" onClick={() => onExport("json")}>JSON</Menu.Item>
          </Menu.Dropdown>
        </Menu>
      </Box>
    </Box>
  );
}