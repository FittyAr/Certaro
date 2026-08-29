<script setup lang="ts">
import { Moon, Sun, SunMoon } from 'lucide-vue-next'
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'

import { ping } from '@/api/app'
import { useConfigStore } from '@/stores/useConfigStore'
import { useUiStore } from '@/stores/useUiStore'

/**
 * Provisional screen for phase 0. It exists to prove three things before any real screen is
 * written: the IPC bridge answers, configuration reaches the frontend, and switching the theme
 * repaints everything through the tokens. Phase 8 replaces it with the real dashboard.
 */
const { t } = useI18n()
const ui = useUiStore()
const config = useConfigStore()

const pong = ref<string | null>(null)
const pingError = ref<string | null>(null)

const themeIcon = computed(() => {
  if (ui.theme === 'light') return Sun
  if (ui.theme === 'dark') return Moon
  return SunMoon
})

const themeLabel = computed(() => {
  if (ui.theme === 'light') return t('General.Theme.Light')
  if (ui.theme === 'dark') return t('General.Theme.Dark')
  return t('General.Theme.System')
})

onMounted(async () => {
  try {
    pong.value = await ping('electroobra')
  } catch {
    pingError.value = t('Error.Unexpected')
  }
})
</script>

<template>
  <main class="flex h-full flex-col gap-6 p-6">
    <header class="flex items-start justify-between gap-4">
      <div>
        <h1 class="text-2xl font-semibold tracking-tight">
          {{ config.config?.application.name ?? 'ElectroObra' }}
        </h1>
        <p class="text-sm text-muted-foreground">
          {{
            ui.bootstrapState === 'ready'
              ? t('App.Ready')
              : ui.bootstrapState === 'failed'
                ? t(ui.bootstrapErrorKey ?? 'Error.BootstrapFailed')
                : t('App.Initializing')
          }}
        </p>
      </div>

      <button
        type="button"
        class="inline-flex items-center gap-2 rounded-md border border-border bg-surface-card px-3 py-2 text-sm font-medium transition-colors hover:bg-surface-raised focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        :aria-label="t('General.Theme.Toggle')"
        @click="ui.cycleTheme()"
      >
        <component :is="themeIcon" class="size-4" aria-hidden="true" />
        {{ themeLabel }}
      </button>
    </header>

    <section class="rounded-lg border border-border bg-surface-card p-4">
      <dl class="grid gap-3 text-sm sm:grid-cols-2">
        <div>
          <dt class="font-medium">IPC</dt>
          <dd class="text-muted-foreground">
            {{ pingError ?? pong ?? t('General.Loading') }}
          </dd>
        </div>
        <div>
          <dt class="font-medium">{{ t('General.Version', { version: '' }) }}</dt>
          <dd class="text-muted-foreground tabular-nums">
            {{ config.info?.version ?? '—' }}
          </dd>
        </div>
      </dl>
    </section>

    <section class="grid gap-3 sm:grid-cols-3">
      <div class="rounded-lg border border-border bg-surface-card p-4">
        <p class="text-sm font-medium">Positive</p>
        <p class="text-lg font-semibold tabular-nums text-money-positive">$ 1.000,00</p>
      </div>
      <div class="rounded-lg border border-border bg-surface-card p-4">
        <p class="text-sm font-medium">Negative</p>
        <p class="text-lg font-semibold tabular-nums text-money-negative">-$ 240,75</p>
      </div>
      <div class="rounded-lg border border-border bg-surface-raised p-4">
        <p class="text-sm font-medium">Neutral</p>
        <p class="text-lg font-semibold tabular-nums text-money-neutral">$ 0,00</p>
      </div>
    </section>
  </main>
</template>
