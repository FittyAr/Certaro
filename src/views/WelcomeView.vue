<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog'
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'

import AppIcon from '@/components/ui/AppIcon.vue'
import { useApiError } from '@/composables/useApiError'
import { useSistemaStore } from '@/stores/useSistemaStore'

/**
 * Welcome screen on first launch. See `docs/19-roadmap.md` §12.
 *
 * Offers two paths: import from the legacy system or start fresh. The import path opens a file
 * dialog for the legacy JSON export and runs the import.
 */

const { t } = useI18n()
const router = useRouter()
const { notify } = useApiError()
const sistema = useSistemaStore()
const importing = ref(false)

async function onStartFresh(): Promise<void> {
  await router.push('/')
}

async function onImport(): Promise<void> {
  const origen = await open({
    multiple: false,
    title: t('Backup.ImportJson'),
    filters: [{ name: 'JSON', extensions: ['json'] }],
  })
  if (!origen) return

  importing.value = true
  try {
    await sistema.importJson(origen)
    await router.push('/')
  } catch (e) {
    notify(e)
  } finally {
    importing.value = false
  }
}
</script>

<template>
  <div class="flex h-screen items-center justify-center bg-background p-6">
    <div class="flex max-w-lg flex-col items-center gap-8 text-center">
      <div>
        <h1 class="text-3xl font-bold">{{ $t('Welcome.Title') }}</h1>
        <p class="mt-2 text-muted-foreground">{{ $t('Welcome.Subtitle') }}</p>
      </div>

      <div class="flex w-full flex-col gap-4 sm:flex-row">
        <button
          class="flex flex-1 flex-col items-center gap-3 rounded-lg border border-border bg-surface-raised p-6 transition-colors hover:border-primary"
          @click="onStartFresh"
        >
          <AppIcon name="plus-circle" :size="32" class="text-primary" />
          <div>
            <p class="font-medium">{{ $t('Welcome.Start') }}</p>
            <p class="text-sm text-muted-foreground">{{ $t('Welcome.StartHint') }}</p>
          </div>
        </button>

        <button
          class="flex flex-1 flex-col items-center gap-3 rounded-lg border border-border bg-surface-raised p-6 transition-colors hover:border-primary"
          :disabled="importing"
          @click="onImport"
        >
          <AppIcon name="database" :size="32" class="text-primary" />
          <div>
            <p class="font-medium">{{ $t('Welcome.Import') }}</p>
            <p class="text-sm text-muted-foreground">{{ $t('Welcome.ImportHint') }}</p>
          </div>
        </button>
      </div>
    </div>
  </div>
</template>
