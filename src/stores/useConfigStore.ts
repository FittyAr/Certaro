import { defineStore } from 'pinia'
import { ref } from 'vue'

import { getAppConfig, getAppInfo, type AppInfo } from '@/api/app'
import type { AppConfig } from '@/api/types'
import { isSupportedLocale, setLocale } from '@/i18n'

import { useUiStore } from './useUiStore'

/**
 * The application configuration, loaded once at startup. Every screen that needs a currency
 * symbol, a date format or a business default reads it from here rather than hardcoding one.
 */
export const useConfigStore = defineStore('config', () => {
  const config = ref<AppConfig | null>(null)
  const info = ref<AppInfo | null>(null)
  const loading = ref(false)

  async function load(): Promise<void> {
    loading.value = true
    try {
      const [loadedConfig, loadedInfo] = await Promise.all([getAppConfig(), getAppInfo()])
      config.value = loadedConfig
      info.value = loadedInfo
      applyPresentation(loadedConfig)
    } finally {
      loading.value = false
    }
  }

  /** Language and theme come from configuration, so they must be applied as soon as it arrives. */
  function applyPresentation(loaded: AppConfig): void {
    const ui = useUiStore()
    ui.setTheme(loaded.application.theme)
    if (isSupportedLocale(loaded.locale.language)) {
      setLocale(loaded.locale.language)
    }
  }

  return { config, info, loading, load }
})
