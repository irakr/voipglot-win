import { defineConfig } from "vite";

export default defineConfig({
  clearScreen: false,
  // Set the root directory to src where index.html is located
  root: "src",
  server: {
    port: 1420,
    strictPort: true,
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: process.env.TAURI_PLATFORM == "windows" ? "chrome105" : "safari13",
    minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_DEBUG,
    // Build output goes to dist/ relative to project root
    outDir: "../dist",
  },
}); 