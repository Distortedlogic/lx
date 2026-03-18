import { defineConfig } from "vite";

export default defineConfig({
  build: {
    ssr: "src/extension.ts",
    outDir: "dist",
    sourcemap: true,
    rollupOptions: {
      external: ["vscode"],
      output: {
        entryFileNames: "extension.js",
        format: "cjs",
        esModule: false,
      },
    },
    minify: false,
  },
});
