import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    host: process.env.TAURI_DEV_HOST || false,
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
    hmr: process.env.TAURI_DEV_HOST
      ? {
          host: process.env.TAURI_DEV_HOST,
          port: 1421,
          protocol: 'ws',
        }
      : undefined,
  },
})
