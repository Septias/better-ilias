import { ViteSSG } from 'vite-ssg'
import devalue from '@nuxt/devalue'
import routes from 'virtual:generated-pages'
import App from './App.vue'
import type { UserModule } from './types'

import '@unocss/reset/tailwind.css'
import 'uno.css'

// https://github.com/antfu/vite-ssg
export const createApp = ViteSSG(
  App,
  { routes, base: import.meta.env.BASE_URL },
  (ctx) => {
    // install all modules under `modules/`
    Object.values(import.meta.glob<{ install: UserModule }>('./modules/*.ts', { eager: true }))
      .forEach(i => i.install?.(ctx))
  },
  {
    transformState(state) {
      return import.meta.env.SSR ? devalue(state) : state
    },
  },
)
