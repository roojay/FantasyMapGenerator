import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  build: {
    outDir: 'dist',
    sourcemap: true,
    rollupOptions: {
      // The WASM glue script is a runtime asset served from public/wasm/.
      // It is loaded via a dynamic import at runtime (with the /* @vite-ignore */
      // comment in wasm-bridge.ts) and must NOT be bundled by Rollup.
      external: ['/wasm/fantasy_map_renderer_wasm.js'],
    },
  },
})
