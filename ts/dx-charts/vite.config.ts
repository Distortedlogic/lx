import { defineConfig } from "vite";

export default defineConfig({
  build: {
    lib: {
      entry: "src/index.ts",
      name: "DxCharts",
      formats: ["iife"],
      fileName: () => "dx-charts.js",
    },
    outDir: "dist",
    minify: false,
    sourcemap: true,
  },
});
