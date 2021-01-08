
let main = ' #15152b'
const colors = require('tailwindcss/colors')
module.exports = {
  purge: ['./index.html', './src/**/*.{vue,js,ts,jsx,tsx}'],
  darkMode: 'media', // or 'media' or 'class'
  theme: {
    extend: {
      backgroundColor: {
        'main': main,
        'accent': colors.rose[600]
      },
      colors: {
        'accent': colors.rose[600]
      }
    },
  },
  variants: {
    extend: {},
  },
  plugins: [],
}
