import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import ViteComponents from 'vite-plugin-components'

export default defineConfig({
  plugins: [
    vue(),
    ViteComponents()
  ],
  server: {
    proxy: {
      '/api': {
        target: 'http://localhost:2020',
        changeOrigin: true,
        secure: false,
      }
    }
  }
})
