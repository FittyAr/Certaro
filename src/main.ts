import { definePreset } from '@primeuix/themes'
import Aura from '@primeuix/themes/aura'
import { createPinia } from 'pinia'
import PrimeVue from 'primevue/config'
import ConfirmationService from 'primevue/confirmationservice'
import ToastService from 'primevue/toastservice'
import { createApp } from 'vue'

import App from './App.vue'
import { i18n } from './i18n'
import { createAppRouter } from './router'

import './assets/main.css'

/**
 * PrimeVue reads the same CSS variables as Tailwind, so a PrimeVue button and a Shadcn button
 * cannot end up different colours. See `docs/16-frontend.md` §3.4.
 */
const preset = definePreset(Aura, {
  semantic: {
    primary: {
      50: 'hsl(var(--primary) / 0.05)',
      100: 'hsl(var(--primary) / 0.1)',
      200: 'hsl(var(--primary) / 0.2)',
      300: 'hsl(var(--primary) / 0.3)',
      400: 'hsl(var(--primary) / 0.6)',
      500: 'hsl(var(--primary))',
      600: 'hsl(var(--primary))',
      700: 'hsl(var(--primary))',
      800: 'hsl(var(--primary))',
      900: 'hsl(var(--primary))',
      950: 'hsl(var(--primary))',
    },
    colorScheme: {
      light: {
        surface: {
          0: 'hsl(var(--background))',
          50: 'hsl(var(--surface-raised))',
          100: 'hsl(var(--surface-raised))',
          200: 'hsl(var(--border))',
        },
      },
      dark: {
        surface: {
          0: 'hsl(var(--background))',
          50: 'hsl(var(--surface-raised))',
          100: 'hsl(var(--surface-raised))',
          200: 'hsl(var(--border))',
        },
      },
    },
  },
})

const app = createApp(App)

/**
 * The seed screen is decided at build time rather than from configuration: the router is built
 * before the backend has finished bootstrapping, and a route that must not exist in a release
 * build is better left unregistered than registered and then hidden.
 */
const router = createAppRouter({
  seedEnabled: import.meta.env.DEV,
  appName: () => 'ElectroObra',
})

app.use(createPinia())
app.use(router)
app.use(i18n)
app.use(PrimeVue, {
  theme: {
    preset,
    options: {
      darkModeSelector: '.dark',
      // This layer order is what lets a Tailwind utility override a PrimeVue style without
      // `!important`. Without it, styling becomes a fight.
      cssLayer: {
        name: 'primevue',
        order: 'tailwind-base, primevue, tailwind-utilities',
      },
    },
  },
  // A desktop application, not a touch app.
  ripple: false,
})
app.use(ToastService)
app.use(ConfirmationService)

app.mount('#app')
