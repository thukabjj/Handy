/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        text: "var(--color-text)",
        background: "var(--color-background)",
        "logo-primary": "var(--color-logo-primary)",
        "logo-stroke": "var(--color-logo-stroke)",
        "text-stroke": "var(--color-text-stroke)",
      },
      // Typography scale for consistent font sizes
      fontSize: {
        // Heading sizes
        "heading-lg": ["1.5rem", { lineHeight: "2rem", fontWeight: "600" }],
        "heading-md": ["1.25rem", { lineHeight: "1.75rem", fontWeight: "600" }],
        "heading-sm": ["1.125rem", { lineHeight: "1.5rem", fontWeight: "600" }],
        // Body sizes (already defined by Tailwind, these are semantic aliases)
        "body-lg": ["1rem", { lineHeight: "1.5rem", fontWeight: "400" }],
        "body-md": ["0.875rem", { lineHeight: "1.25rem", fontWeight: "400" }],
        "body-sm": ["0.75rem", { lineHeight: "1rem", fontWeight: "400" }],
        // Label/caption
        caption: ["0.6875rem", { lineHeight: "0.875rem", fontWeight: "500" }],
      },
      // Spacing scale for consistent gaps/padding
      spacing: {
        // Semantic spacing tokens
        "section-gap": "1.5rem", // 24px - between major sections
        "card-padding": "1rem", // 16px - inside cards/containers
        "input-padding": "0.5rem", // 8px - inside inputs
        "element-gap": "0.5rem", // 8px - between related elements
        "tight-gap": "0.25rem", // 4px - tight spacing
      },
      // Border radius scale
      borderRadius: {
        button: "0.375rem", // 6px - for buttons
        card: "0.5rem", // 8px - for cards
        input: "0.375rem", // 6px - for inputs
        pill: "9999px", // fully rounded
      },
    },
  },
  plugins: [],
};
