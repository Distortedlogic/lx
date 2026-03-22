import { defineConfig } from "vite";

export default defineConfig({
  build: {
    lib: {
      entry: "src/index.ts",
      name: "WidgetBridge",
      formats: ["iife"],
      fileName: () => "widget-bridge.js",
    },
    outDir: "dist",
    minify: false,
    sourcemap: true,
    cssCodeSplit: false,
  },
});
