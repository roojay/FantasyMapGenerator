/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        'map-bg': '#c8e8f8',
        'map-land': '#a8c890',
        'map-water': '#4488cc',
      },
    },
  },
  plugins: [],
}
