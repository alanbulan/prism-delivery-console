import path from "path";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

// Tauri 开发环境主机配置
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },

  // Vite 配置 - 针对 Tauri 开发优化
  //
  // 1. 防止 Vite 遮盖 Rust 错误信息
  clearScreen: false,
  // 2. Tauri 需要固定端口，端口不可用时直接报错
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. 忽略 src-tauri 目录的文件变更监听
      ignored: ["**/src-tauri/**"],
    },
  },
}));
