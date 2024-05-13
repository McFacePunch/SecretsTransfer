/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './**/*.{html,js}', // Adjust the path based on your project structure
    './*.html', // Ensure to include the root HTML files
  ],
  theme: {
    extend: {
      colors: {
        'cool-blue-100': '#1A1F36',
        'cool-blue-200': '#252A40',
        'cool-blue-300': '#313859',
        'cool-blue-400': '#3D476F',
        'cool-blue-500': '#495486',
        'cool-blue-600': '#56629D',
        'cool-blue-700': '#6471B5',
        'cool-blue-800': '#7481CE',
        'cool-blue-900': '#8593E7',
        'accent-teal': '#00D1C1',
        'accent-cyan': '#00AEEF',
        'accent-indigo': '#5C6BC0',
        'accent-purple': '#9C27B0',
      },
      fontFamily: {
        sans: ['Roboto', 'sans-serif'],
        mono: ['Courier New', 'monospace'],
      },
      boxShadow: {
        'neon': '0 4px 30px rgba(0, 223, 255, 0.1), 0 4px 12px rgba(0, 223, 255, 0.3)'
      }
    },
  },
  plugins: [],
}
