import {
  createTheme,
  localStorageColorSchemeManager,
  type CSSVariablesResolver,
  type MantineColorsTuple,
} from "@mantine/core";

const brand: MantineColorsTuple = [
  "#eef7ff",
  "#d8e8ff",
  "#b0d0ff",
  "#84b7ff",
  "#5f9dff",
  "#4a8fff",
  "#4088ff",
  "#3375e4",
  "#2668cc",
  "#1359b6",
];

export const colorSchemeManager = localStorageColorSchemeManager({
  key: "fantasy-map-color-scheme",
});

export const theme = createTheme({
  primaryColor: "brand",
  colors: {
    brand,
  },
  defaultRadius: "md",
  fontFamily:
    "'Inter Variable', 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif",
  headings: {
    fontFamily:
      "'Inter Variable', 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif",
  },
});

export const cssVariablesResolver: CSSVariablesResolver = () => ({
  variables: {
    "--app-accent": "74 144 255",
    "--app-accent-strong": "33 102 255",
    "--app-success": "16 185 129",
    "--app-danger": "239 68 68",
    "--app-grid-line": "148 163 184",
  },
  light: {},
  dark: {},
});
