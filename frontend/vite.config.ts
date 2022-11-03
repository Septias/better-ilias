import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import ViteComponents from 'vite-plugin-components'
import ViteIcons, { ViteIconsResolver } from 'vite-plugin-icons'

export default defineConfig({
  plugins: [
    vue(),
    ViteComponents({
      customComponentResolvers: [
        // https://github.com/antfu/vite-plugin-icons
        ViteIconsResolver({
          componentPrefix: '',
          // enabledCollections: ['carbon']
        }),
      ],
    }),
    ViteIcons(),
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
