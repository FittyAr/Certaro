<script setup lang="ts">
import { listen } from '@tauri-apps/api/event'
import ConfirmDialog from 'primevue/confirmdialog'
import Toast from 'primevue/toast'
import { onErrorCaptured, onMounted, onUnmounted, ref } from 'vue'

import AppShell from '@/components/layout/AppShell.vue'
import { useConfigStore } from '@/stores/useConfigStore'
import { useUiStore } from '@/stores/useUiStore'
import ErrorView from '@/views/errors/ErrorView.vue'

const ui = useUiStore()
const config = useConfigStore()

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
      ui.markReady()
    }),
  )
  unlisteners.push(
    await listen<string>('app://fatal', (event) => {
      ui.markFailed(event.payload)
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
