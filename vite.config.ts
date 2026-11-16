import path from "path"
import react from "@vitejs/plugin-react"
import tailwindcss from "@tailwindcss/vite"
import { defineConfig } from "vite"

// Tauri expects a fixed dev port (see devUrl in src-tauri/tauri.conf.json).
export default defineConfig({
  plugins: [tailwindcss(), react()],
  clearScreen: false,
  server: { port: 5173, strictPort: true },
  resolve: {
    alias: {
      "@": path.resolve(import.meta.dirname, "./src"),
    },
  },
})
