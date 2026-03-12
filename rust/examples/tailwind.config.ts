import type { Config } from "tailwindcss";

const config: Config = {
  darkMode: ["selector", '[data-mantine-color-scheme="dark"]'],
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        app: {
          bg: "rgb(var(--app-bg) / <alpha-value>)",
          surface: "rgb(var(--app-surface) / <alpha-value>)",
          elevated: "rgb(var(--app-elevated) / <alpha-value>)",
          text: "rgb(var(--app-text) / <alpha-value>)",
          muted: "rgb(var(--app-muted) / <alpha-value>)",
          border: "rgb(var(--app-border) / <alpha-value>)",
          accent: "rgb(var(--app-accent) / <alpha-value>)",
          accentStrong: "rgb(var(--app-accent-strong) / <alpha-value>)",
          success: "rgb(var(--app-success) / <alpha-value>)",
          danger: "rgb(var(--app-danger) / <alpha-value>)",
        },
      },
      boxShadow: {
        ambient: "0 24px 80px rgba(15, 23, 42, 0.18)",
        panel: "0 20px 60px rgba(2, 8, 23, 0.28)",
      },
      fontFamily: {
        display: ["var(--font-display)"],
        sans: ["var(--font-sans)"],
        mono: ["var(--font-mono)"],
      },
      backgroundImage: {
        "app-grid":
          "linear-gradient(to right, rgba(var(--app-grid-line), 0.2) 1px, transparent 1px), linear-gradient(to bottom, rgba(var(--app-grid-line), 0.16) 1px, transparent 1px)",
      },
      animation: {
        "soft-rise": "soft-rise 500ms ease-out",
        "orb-float": "orb-float 9s ease-in-out infinite",
      },
      keyframes: {
        "soft-rise": {
          "0%": { opacity: "0", transform: "translateY(14px)" },
          "100%": { opacity: "1", transform: "translateY(0)" },
        },
        "orb-float": {
          "0%, 100%": { transform: "translate3d(0, 0, 0)" },
          "50%": { transform: "translate3d(0, -18px, 0)" },
        },
      },
    },
  },
  plugins: [],
};

export default config;
