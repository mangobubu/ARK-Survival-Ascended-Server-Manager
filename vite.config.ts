import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // Rust 编译产物在 Windows 上可能正被占用，无需交给 Vite 监听。
      ignored: ['**/src-tauri/target/**'],
    },
  },
  envPrefix: ['VITE_', 'TAURI_ENV_*'],
  build: {
    target: process.env.TAURI_ENV_PLATFORM === 'windows' ? 'chrome105' : 'safari13',
    minify: process.env.TAURI_ENV_DEBUG ? false : 'oxc',
    sourcemap: Boolean(process.env.TAURI_ENV_DEBUG),
  },
})
