import { defineConfig } from "vite";

export default defineConfig({
  build: {
    lib: {
      entry: "src/index.ts",
      name: "AudioCapture",
      formats: ["es"],
      fileName: () => "audio-capture.js",
    },
    outDir: "dist",
    minify: false,
    sourcemap: true,
  },
});
