import path from "node:path";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "src"),
    },
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (id.includes("node_modules")) {
            if (
              id.includes("/react/") ||
              id.includes("/react-dom/") ||
              id.includes("/scheduler/")
            ) {
              return "react-vendor";
            }

            if (
              id.includes("/@mantine/") ||
              id.includes("/@tabler/") ||
              id.includes("/i18next/") ||
              id.includes("/react-i18next/")
            ) {
              return "ui-vendor";
            }
          }

          return undefined;
        },
      },
    },
  },
  worker: {
    format: "es",
  },
  assetsInclude: ["**/*.wasm"],
  server: {
    host: "0.0.0.0",
    port: 5173,
  },
});
