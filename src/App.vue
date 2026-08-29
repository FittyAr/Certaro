<script setup lang="ts">
import { listen } from '@tauri-apps/api/event'
import ConfirmDialog from 'primevue/confirmdialog'
import Toast from 'primevue/toast'
import { onMounted, onUnmounted } from 'vue'

import { useConfigStore } from '@/stores/useConfigStore'
import { useUiStore } from '@/stores/useUiStore'

const ui = useUiStore()
const config = useConfigStore()

const unlisteners: Array<() => void> = []

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
  <RouterView />
  <Toast position="bottom-right" />
  <ConfirmDialog />
</template>
