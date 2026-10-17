import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Tauri 开发环境配置：固定端口，避免与 tauri 前端地址冲突
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    target: "es2021",
    outDir: "dist",
  },
});
