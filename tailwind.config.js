/** @type {import('tailwindcss').Config} */
export default {
  darkMode: ["class"],
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        border: "hsl(var(--border))",
        input: "hsl(var(--input))",
        ring: "hsl(var(--ring))",
        background: "hsl(var(--background))",
        foreground: "hsl(var(--foreground))",
        primary: { DEFAULT: "hsl(var(--primary))", foreground: "hsl(var(--primary-foreground))" },
        secondary: { DEFAULT: "hsl(var(--secondary))", foreground: "hsl(var(--secondary-foreground))" },
        muted: { DEFAULT: "hsl(var(--muted))", foreground: "hsl(var(--muted-foreground))" },
        accent: { DEFAULT: "hsl(var(--accent))", foreground: "hsl(var(--accent-foreground))" },
        card: { DEFAULT: "hsl(var(--card))", foreground: "hsl(var(--card-foreground))" },
        popover: { DEFAULT: "hsl(var(--popover))", foreground: "hsl(var(--popover-foreground))" },
        status: {
          working: "hsl(var(--status-working))",
          needsYou: "hsl(var(--status-needs-you))",
          done: "hsl(var(--status-done))",
          idle: "hsl(var(--status-idle))",
          dead: "hsl(var(--status-dead))",
          unknown: "hsl(var(--status-unknown))",
        },
        brand: {
          claude: "hsl(var(--brand-claude))",
          codex: "hsl(var(--brand-codex))",
          opencode: "hsl(var(--brand-opencode))",
          cursor: "hsl(var(--brand-cursor))",
          gemini: "hsl(var(--brand-gemini))",
          copilot: "hsl(var(--brand-copilot))",
        },
      },
      borderRadius: { lg: "var(--radius)", md: "calc(var(--radius) - 2px)", sm: "calc(var(--radius) - 4px)" },
      fontFamily: {
        sans: ["-apple-system", "BlinkMacSystemFont", "SF Pro Text", "Segoe UI", "sans-serif"],
        mono: ["SF Mono", "ui-monospace", "Menlo", "monospace"],
      },
      keyframes: {
        "pulse-ring": {
          "0%, 100%": { boxShadow: "0 0 0 0 hsl(var(--status-needs-you) / 0.35)" },
          "50%": { boxShadow: "0 0 0 7px hsl(var(--status-needs-you) / 0)" },
        },
      },
      animation: { "pulse-ring": "pulse-ring 1.9s ease-in-out infinite" },
    },
  },
  plugins: [require("tailwindcss-animate")],
};
