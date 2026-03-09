/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/**/*.{js,ts,jsx,tsx,mdx}",
  ],
  theme: {
    extend: {
      colors: {
        // Align with MedusaJS brand palette
        "ui-fg-base": "rgb(var(--fg-base) / <alpha-value>)",
        "ui-bg-base": "rgb(var(--bg-base) / <alpha-value>)",
      },
    },
  },
  plugins: [],
};
