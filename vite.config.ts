import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import path from "path";

export default defineConfig({
  plugins: [vue()],
  clearScreen: false,
  server: {
    host: "localhost",
    port: 5173,
    strictPort: true,
    // 开发时把 /api 转发到 Axum 后端（8080），便于浏览器直接访问 5173 时也能调接口
    proxy: {
      "/api": "http://127.0.0.1:8080",
    },
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: ["chrome105", "safari13"],
    minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_DEBUG,
    outDir: "dist",
    rollupOptions: {
      input: {
        main: path.resolve(__dirname, "index.html"),
      },
    },
  },
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "src"),
    },
  },
});
