import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// Em dev, proxy /api pro backend axum rodando em 8087.
// Em produção, o Nginx (ver Dockerfile) faz esse mesmo papel.
export default defineConfig({
  plugins: [vue()],
  server: {
    port: 5173,
    proxy: {
      '/api': {
        target: 'http://localhost:8087',
        changeOrigin: true,
      },
    },
  },
})
