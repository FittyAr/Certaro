import { defineStore } from 'pinia'
import { computed, ref, watchEffect } from 'vue'

import type { ThemePreference } from '@/api/types'

/**
 * Cross-cutting interface state: theme, sidebar, command palette. Nothing here is business data.
 */
export const useUiStore = defineStore('ui', () => {
  const theme = ref<ThemePreference>('system')
  const sidebarExpanded = ref(true)
  const commandPaletteOpen = ref(false)
  const bootstrapState = ref<'initializing' | 'ready' | 'failed'>('initializing')
  const bootstrapErrorKey = ref<string | null>(null)

  const prefersDark =
    typeof window !== 'undefined' && typeof window.matchMedia === 'function'
      ? window.matchMedia('(prefers-color-scheme: dark)')
      : null

  const systemIsDark = ref(prefersDark?.matches ?? false)
  prefersDark?.addEventListener('change', (e) => {
    systemIsDark.value = e.matches
  })

  const isDark = computed(() =>
    theme.value === 'system' ? systemIsDark.value : theme.value === 'dark',
  )

  // The class on `<html>` is what both Tailwind's `darkMode: 'class'` and PrimeVue's
  // `darkModeSelector` read, so switching it is the whole of theme switching.
  watchEffect(() => {
    if (typeof document === 'undefined') return
    document.documentElement.classList.toggle('dark', isDark.value)
  })

  function setTheme(next: ThemePreference): void {
    theme.value = next
  }

  function cycleTheme(): void {
    const order: ThemePreference[] = ['light', 'dark', 'system']
    const index = order.indexOf(theme.value)
    theme.value = order[(index + 1) % order.length] ?? 'system'
  }

  function toggleSidebar(): void {
    sidebarExpanded.value = !sidebarExpanded.value
  }

  function markReady(): void {
    bootstrapState.value = 'ready'
    bootstrapErrorKey.value = null
  }

  function markFailed(messageKey: string): void {
    bootstrapState.value = 'failed'
    bootstrapErrorKey.value = messageKey
  }

  return {
    theme,
    isDark,
    sidebarExpanded,
    commandPaletteOpen,
    bootstrapState,
    bootstrapErrorKey,
    setTheme,
    cycleTheme,
    toggleSidebar,
    markReady,
    markFailed,
  }
})
