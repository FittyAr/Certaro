import { createI18n } from 'vue-i18n'

import en from '@/locales/en.json'
import es from '@/locales/es.json'

/**
 * See `docs/14-configuracion-e-i18n.md` §4.
 *
 * `es` is the canonical locale and `en` must have exactly the same keys; `pnpm i18n:check` enforces
 * it. A missing key fails the test in development and renders as `[Some.Key]` in production, never
 * as an empty string, so a gap is visible instead of silent.
 */
export const SUPPORTED_LOCALES = ['es', 'en'] as const
export type SupportedLocale = (typeof SUPPORTED_LOCALES)[number]

export const i18n = createI18n({
  legacy: false,
  locale: 'es',
  fallbackLocale: 'es',
  missingWarn: import.meta.env.DEV,
  fallbackWarn: import.meta.env.DEV,
  messages: { es, en },
  missing: (_locale, key) => `[${key}]`,
})

export function setLocale(locale: SupportedLocale): void {
  i18n.global.locale.value = locale
  document.documentElement.setAttribute('lang', locale)
}

export function isSupportedLocale(value: string): value is SupportedLocale {
  return (SUPPORTED_LOCALES as readonly string[]).includes(value)
}
