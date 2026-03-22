import { defineConfig } from "vite";

export default defineConfig({
  build: {
    lib: {
      entry: "src/index.ts",
      name: "VoiceClient",
      formats: ["iife"],
      fileName: () => "voice-client.js",
    },
    outDir: "dist",
    minify: false,
    sourcemap: true,
  },
});
