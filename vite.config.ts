import { defineConfig } from "vite";

// Tauri sert le frontend depuis un port fixe en developpement, pour que le
// binaire Rust pointe toujours vers le meme endroit.
export default defineConfig({
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  envPrefix: ["VITE_", "TAURI_ENV_*"],
  build: {
    target: "es2022",
    minify: "esbuild",
    sourcemap: true,
  },
});
