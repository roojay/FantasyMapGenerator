import {
  ActionIcon,
  Box,
  Button,
  Menu,
  SegmentedControl,
  Tooltip,
  useMantineColorScheme,
} from "@mantine/core";
import {
  IconChevronDown,
  IconFileDownload,
  IconFileUpload,
  IconFocus2,
  IconMoon,
  IconRefresh,
  IconSun,
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
  onToggleTheme,
}: TopToolbarProps) {
  const { t } = useTranslation();
  const { colorScheme: resolvedScheme } = useMantineColorScheme();
  const isDark = resolvedScheme === "dark";

  const glassStyle: React.CSSProperties = {
    backgroundColor: "var(--mantine-color-body)",
    borderColor: "rgb(var(--app-border))",
  };

  return (
    <Box className="pointer-events-none absolute left-1/2 top-[max(0.5rem,var(--app-safe-top))] z-30 w-[calc(100%-1rem)] -translate-x-1/2 sm:left-auto sm:right-4 sm:top-4 sm:w-auto sm:translate-x-0 lg:right-6 lg:top-6">
      <Box
        className="pointer-events-auto flex max-w-full flex-wrap items-center justify-end gap-1.5 rounded-xl border p-2 shadow-md backdrop-blur-xl sm:flex-nowrap sm:gap-2 sm:px-3 sm:py-2"
        style={glassStyle}
      >
        {/* Language selector */}
        <SegmentedControl
          size="xs"
          className="shrink-0"
          classNames={{
            root: "min-w-[88px]",
            indicator: "transition-all duration-200 ease-out",
          }}
          value={language}
          onChange={(value) => onLanguageChange(value as AppLanguage)}
          data={[
            { label: "中文", value: "zh-CN" },
            { label: "EN", value: "en" },
          ]}
        />

        {/* Divider */}
        <Box
          className="hidden h-4 w-px shrink-0 sm:block"
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
            aria-label={t("toolbar.theme")}
            classNames={{
              root: cn("hover:scale-110 active:scale-90", "transition-transform duration-200"),
            }}
          >
            {isDark ? <IconSun size={14} /> : <IconMoon size={14} />}
          </ActionIcon>
        </Tooltip>

        {/* Divider */}
        <Box
          className="hidden h-4 w-px shrink-0 sm:block"
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
            aria-label={t("toolbar.fit")}
            classNames={{
              root: cn("hover:scale-110 active:scale-90", "transition-transform duration-200"),
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
            aria-label={t("toolbar.reset")}
            classNames={{
              root: cn("hover:scale-110 active:scale-90", "transition-transform duration-200"),
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
              "h-8 min-w-[2.25rem] max-[479px]:!px-2",
              "hover:scale-105 active:scale-95",
              "transition-transform duration-200",
            ),
          }}
        >
          <span className="max-[479px]:hidden">{t("toolbar.import")}</span>
        </Button>

        {/* Export menu */}
        <Menu shadow="lg" width={160} position="bottom-end">
          <Menu.Target>
            <Button
              size="xs"
              rightSection={<IconChevronDown size={12} />}
              leftSection={<IconFileDownload size={14} />}
              disabled={!canExport}
              className="cursor-pointer"
              classNames={{
                root: cn(
                  "h-8 min-w-[2.25rem] max-[479px]:!px-2",
                  "hover:scale-105 active:scale-95",
                  "transition-transform duration-200",
                ),
              }}
            >
              <span className="max-[479px]:hidden">{t("toolbar.export")}</span>
            </Button>
          </Menu.Target>
          <Menu.Dropdown>
            <Menu.Item className="cursor-pointer" onClick={() => onExport("svg")}>
              SVG
            </Menu.Item>
            <Menu.Item className="cursor-pointer" onClick={() => onExport("png")}>
              PNG
            </Menu.Item>
            <Menu.Item className="cursor-pointer" onClick={() => onExport("json")}>
              JSON
            </Menu.Item>
          </Menu.Dropdown>
        </Menu>
      </Box>
    </Box>
  );
}
