<script setup lang="ts">
import { listen } from '@tauri-apps/api/event'
import ConfirmDialog from 'primevue/confirmdialog'
import Toast from 'primevue/toast'
import { onErrorCaptured, onMounted, onUnmounted, ref } from 'vue'

import AppShell from '@/components/layout/AppShell.vue'
import { useConfigStore } from '@/stores/useConfigStore'
import { useDashboardStore, type Cotizacion } from '@/stores/useDashboardStore'
import { useUiStore } from '@/stores/useUiStore'
import { useVersionCheck } from '@/composables/useVersionCheck'
import ErrorView from '@/views/errors/ErrorView.vue'

const ui = useUiStore()
const config = useConfigStore()
const dashboard = useDashboardStore()
const versionCheck = useVersionCheck()

const unlisteners: Array<() => void> = []

/** Set by the error barrier; clearing it remounts the shell. See `docs/16-frontend.md` §6.4. */
const renderError = ref<string | null>(null)

onErrorCaptured((error) => {
  console.error('[render]', error)
  renderError.value = error instanceof Error ? error.message : String(error)
  return false
})

onMounted(async () => {
  // The backend bootstraps the database in the background and announces the outcome. Until then
  // the interface shows an initialising state rather than an empty screen.
  unlisteners.push(
    await listen('app://ready', async () => {
      await config.load()
      ui.privacyMode = config.config?.dashboard.privacyMode ?? false
      ui.markReady()
      // The status bar shows the rate, so it is loaded once here instead of only by the dashboard.
      // A failure is expected and ignored: the bar just does not show it (doc 13 §2.4).
      dashboard.fetchCotizaciones().catch(() => undefined)
      // Check for a newer version on GitHub. Short timeout, silent degradation.
      versionCheck.check()
    }),
  )
  // The backend refreshes the rate on startup and announces it when it arrives.
  unlisteners.push(
    await listen<Cotizacion[]>('cotizaciones:updated', (event) => {
      dashboard.cotizaciones = event.payload
    }),
  )
  unlisteners.push(
    await listen<string>('app://fatal', (event) => {
      ui.markFailed(event.payload)
    }),
  )
  // The maintenance task runs in the background after bootstrap. Its result is informational:
  // a backup was created, old backups were deleted, attachment trash was purged. None of these
  // warrant a dialog, but the status bar could show them.
  unlisteners.push(
    await listen<Record<string, unknown>>('mantenimiento:done', () => {
      // The result is available if a future status bar wants to display it.
    }),
  )
})

onUnmounted(() => {
  unlisteners.forEach((off) => off())
})
</script>

<template>
  <ErrorView v-if="renderError" :detail="renderError" @retry="renderError = null" />
  <AppShell v-else />
  <Toast position="bottom-right" />
  <ConfirmDialog />
</template>
