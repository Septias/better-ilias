import { ViteSSG } from 'vite-ssg'
import devalue from '@nuxt/devalue'
import routes from 'virtual:generated-pages'
import App from './App.vue'
import type { UserModule } from './types'

import "~/styles/main.css"
import '@unocss/reset/tailwind.css'
import 'uno.css'

// https://github.com/antfu/vite-ssg
export const createApp = ViteSSG(
  App,
  { routes, base: import.meta.env.BASE_URL },
)
