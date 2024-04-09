import {
  defineConfig,
  extractorSplit,
  presetAttributify,
  presetIcons,
  presetTypography,
  presetUno,
  presetWebFonts,
  transformerDirectives,
  transformerVariantGroup,
} from 'unocss'
import extractorPug from '@unocss/extractor-pug'

export default defineConfig({
  theme: {
    colors: {
      my: {
        lightgray: 'var(--my-lightgray)',
        darkgray: 'var(--my-darkgray)',
        gray: 'var(--my-gray)',
        text: 'var(--my-text)',
        accent: 'var(--my-accent)',
        accentlight: 'var(--my-accent-light)',
        heading: 'var(--my-heading)',
      },
    },
  },
  presets: [
    presetUno(),
    presetAttributify(),
    presetIcons({
      scale: 1.2,
      warn: true,
    }),
    presetTypography(),
    presetWebFonts({
      fonts: {
        sans: 'DM Sans',
        serif: 'DM Serif Display',
        mono: 'DM Mono',
      },
    }),
  ],
  extractors: [
    extractorPug(),
    extractorSplit,
  ],
  transformers: [
    transformerDirectives(),
    transformerVariantGroup(),
  ],
})
