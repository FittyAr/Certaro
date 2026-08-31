<script setup lang="ts">
import { listen } from '@tauri-apps/api/event'
import ConfirmDialog from 'primevue/confirmdialog'
import Toast from 'primevue/toast'
import { onErrorCaptured, onMounted, onUnmounted, ref } from 'vue'
import { useRouter } from 'vue-router'

import AppShell from '@/components/layout/AppShell.vue'
import AppSplash from '@/components/layout/AppSplash.vue'
import { appIsReady } from '@/api/app'
import { isTauri } from '@/api/client'
import { useConfigStore } from '@/stores/useConfigStore'
import { useDashboardStore, type Cotizacion } from '@/stores/useDashboardStore'
import { useSistemaStore } from '@/stores/useSistemaStore'
import { useUiStore } from '@/stores/useUiStore'
import { useVersionCheck } from '@/composables/useVersionCheck'
import ErrorView from '@/views/errors/ErrorView.vue'

const ui = useUiStore()
const config = useConfigStore()
const dashboard = useDashboardStore()
const sistema = useSistemaStore()
const versionCheck = useVersionCheck()
const router = useRouter()

const unlisteners: Array<() => void> = []

/** Set by the error barrier; clearing it remounts the shell. See `docs/16-frontend.md` §6.4. */
const renderError = ref<string | null>(null)

onErrorCaptured((error) => {
  console.error('[render]', error)
  renderError.value = error instanceof Error ? error.message : String(error)
  return false
})

let isInitialized = false

async function onAppReady(): Promise<void> {
  if (isInitialized) return
  isInitialized = true

  try {
    await config.load()
    ui.privacyMode = config.config?.dashboard.privacyMode ?? false

    // If first launch, check whether a legacy database exists to guide the user
    const welcomed = localStorage.getItem('eo:welcomed')
    if (!welcomed) {
      try {
        const candidate = await sistema.detectLegacyDb()
        if (candidate) {
          await router.push('/welcome')
        }
      } catch {
        // Non-blocking detection failure
      }
    }

    ui.markReady()
    dashboard.fetchCotizaciones().catch(() => undefined)
    versionCheck.check()
  } catch (e) {
    console.error('Error during startup bootstrap:', e)
    ui.markReady()
  }
}

onMounted(async () => {
  if (isTauri()) {
    try {
      // 1. Attach background event listeners
      unlisteners.push(
        await listen('app://ready', async () => {
          await onAppReady()
        }),
      )
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
      unlisteners.push(
        await listen<Record<string, unknown>>('mantenimiento:done', () => {
          // Maintenance completed
        }),
      )
    } catch (err) {
      console.error('Failed to attach Tauri event listeners:', err)
    }

    // 2. Actively check if backend is already ready
    try {
      const ready = await appIsReady()
      if (ready) {
        await onAppReady()
        return
      }
    } catch {
      // Fallback if command cannot be invoked yet
    }

    // 3. Fallback poll in case the event was emitted right around mount
    const pollId = setInterval(async () => {
      if (isInitialized) {
        clearInterval(pollId)
        return
      }
      try {
        if (await appIsReady()) {
          clearInterval(pollId)
          await onAppReady()
        }
      } catch {
        // Keep waiting
      }
    }, 250)

    // Clear poll after 10 seconds
    setTimeout(() => {
      clearInterval(pollId)
      if (!isInitialized) {
        void onAppReady()
      }
    }, 10000)
  } else {
    console.warn('[app] Running outside Tauri desktop container; activating web preview mode.')
    ui.markReady()
  }
})

onUnmounted(() => {
  unlisteners.forEach((off) => off())
})
</script>

<template>
  <ErrorView v-if="renderError" :detail="renderError" @retry="renderError = null" />
  <ErrorView
    v-else-if="ui.bootstrapState === 'failed'"
    :detail="$t(ui.bootstrapErrorKey ?? 'Error.Unexpected')"
  />
  <AppSplash v-else-if="ui.bootstrapState === 'initializing'" />
  <AppShell v-else />
  <Toast position="bottom-right" />
  <ConfirmDialog />
</template>
