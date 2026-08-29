import type { Config } from 'tailwindcss'
import primeui from 'tailwindcss-primeui'

/**
 * Every colour is a CSS variable defined in `src/assets/tokens.css`. Nothing here is a literal
 * colour, and nothing in a `.vue` file may be either: the system has a light and a dark theme, and
 * a literal that looks right in one looks wrong in the other. See `docs/16-frontend.md` §3.
 */
const config: Config = {
  darkMode: ['class'],
  content: ['./index.html', './src/**/*.{vue,ts}'],
  theme: {
    container: {
      center: true,
      padding: '2rem',
      screens: { '2xl': '1400px' },
    },
    extend: {
      colors: {
        background: 'hsl(var(--background))',
        foreground: 'hsl(var(--foreground))',
        muted: {
          DEFAULT: 'hsl(var(--muted))',
          foreground: 'hsl(var(--muted-foreground))',
        },
        surface: {
          card: 'hsl(var(--surface-card))',
          raised: 'hsl(var(--surface-raised))',
          overlay: 'hsl(var(--surface-overlay))',
        },
        border: 'hsl(var(--border))',
        input: 'hsl(var(--input))',
        ring: 'hsl(var(--ring))',
        primary: {
          DEFAULT: 'hsl(var(--primary))',
          foreground: 'hsl(var(--primary-foreground))',
        },
        secondary: {
          DEFAULT: 'hsl(var(--secondary))',
          foreground: 'hsl(var(--secondary-foreground))',
        },
        accent: {
          DEFAULT: 'hsl(var(--accent))',
          foreground: 'hsl(var(--accent-foreground))',
        },
        destructive: {
          DEFAULT: 'hsl(var(--destructive))',
          foreground: 'hsl(var(--destructive-foreground))',
        },
        warning: {
          DEFAULT: 'hsl(var(--warning))',
          foreground: 'hsl(var(--warning-foreground))',
        },
        success: {
          DEFAULT: 'hsl(var(--success))',
          foreground: 'hsl(var(--success-foreground))',
        },
        money: {
          positive: 'hsl(var(--money-positive))',
          negative: 'hsl(var(--money-negative))',
          neutral: 'hsl(var(--money-neutral))',
        },
        state: {
          draft: 'hsl(var(--state-draft))',
          issued: 'hsl(var(--state-issued))',
          paid: 'hsl(var(--state-paid))',
          partial: 'hsl(var(--state-partial))',
          overdue: 'hsl(var(--state-overdue))',
          void: 'hsl(var(--state-void))',
          active: 'hsl(var(--state-active))',
          paused: 'hsl(var(--state-paused))',
          finished: 'hsl(var(--state-finished))',
          cancelled: 'hsl(var(--state-cancelled))',
        },
      },
      borderRadius: {
        lg: 'var(--radius)',
        md: 'calc(var(--radius) - 2px)',
        sm: 'calc(var(--radius) - 4px)',
      },
      fontFamily: {
        sans: ['Inter', 'Segoe UI', 'system-ui', 'sans-serif'],
        mono: ['JetBrains Mono', 'Consolas', 'monospace'],
      },
      keyframes: {
        'accordion-down': {
          from: { height: '0' },
          to: { height: 'var(--reka-accordion-content-height)' },
        },
        'accordion-up': {
          from: { height: 'var(--reka-accordion-content-height)' },
          to: { height: '0' },
        },
      },
      animation: {
        'accordion-down': 'accordion-down 0.2s ease-out',
        'accordion-up': 'accordion-up 0.2s ease-out',
      },
    },
  },
  plugins: [primeui],
}

export default config
