import { defineConfig } from "vite";

export default defineConfig({
  build: {
    lib: {
      entry: "src/index.ts",
      name: "AudioPlayback",
      formats: ["es"],
      fileName: () => "audio-playback.js",
    },
    outDir: "dist",
    minify: false,
    sourcemap: true,
  },
});
