const colors = require('tailwindcss/colors')


let light_main = '#232349'
let main = '#15152b'
let accent = colors.rose[600]


module.exports = {
  purge: ['./index.html', './src/**/*.{vue,js,ts,jsx,tsx}'],
  darkMode: 'media', // or 'media' or 'class'
  theme: {
    extend: {
      backgroundColor: {
        'main': main,
        'light-main': light_main,
        'accent': accent

      },
      colors: {
        'accent': accent
      },
      outline: {
        'main': main,
        'light-main': light_main,
        accent
      }
    },
  },
  variants: {
    extend: {},
  },
  plugins: [],
}
