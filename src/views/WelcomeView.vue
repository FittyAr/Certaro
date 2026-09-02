<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog'
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'

import AppIcon from '@/components/ui/AppIcon.vue'
import { useApiError } from '@/composables/useApiError'
import {
  useSistemaStore,
  type LegacyDbCandidate,
  type LegacyImportSummary,
} from '@/stores/useSistemaStore'

/**
 * Welcome screen on first launch with auto-detection of legacy database and guided migration wizard.
 * See `docs/15-migracion-de-datos.md` and `docs/19-roadmap.md` §12.
 */

const { t } = useI18n()
const router = useRouter()
const { notify } = useApiError()
const sistema = useSistemaStore()

type Step = 'welcome' | 'confirm' | 'importing' | 'summary'

const step = ref<Step>('welcome')
const detectedDb = ref<LegacyDbCandidate | null>(null)
const selectedDbPath = ref<string>('')
const selectedDbSize = ref<number>(0)
const allowOrphans = ref<boolean>(true)
const importSummary = ref<LegacyImportSummary | null>(null)
const isCheckingDb = ref<boolean>(true)

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`
}

onMounted(async () => {
  try {
    const candidate = await sistema.detectLegacyDb()
    if (candidate) {
      detectedDb.value = candidate
    }
  } catch (e) {
    // Non-blocking detection error
    console.warn('Legacy DB detection error:', e)
  } finally {
    isCheckingDb.value = false
  }
})

async function onStartFresh(): Promise<void> {
  localStorage.setItem('eo:welcomed', 'true')
  await router.push('/')
}

function onSelectDetected(): void {
  if (!detectedDb.value) return
  selectedDbPath.value = detectedDb.value.path
  selectedDbSize.value = detectedDb.value.sizeBytes
  step.value = 'confirm'
}

async function onSelectOtherDb(): Promise<void> {
  const file = await open({
    multiple: false,
    title: t('Welcome.SelectOtherDb'),
    filters: [{ name: 'SQLite Database', extensions: ['db', 'sqlite', 'sqlite3'] }],
  })
  if (!file) return

  selectedDbPath.value = file
  selectedDbSize.value = 0
  step.value = 'confirm'
}

async function onImportJson(): Promise<void> {
  const origen = await open({
    multiple: false,
    title: t('Backup.ImportJson'),
    filters: [{ name: 'JSON', extensions: ['json'] }],
  })
  if (!origen) return

  step.value = 'importing'
  try {
    await sistema.importJson(origen)
    localStorage.setItem('eo:welcomed', 'true')
    await router.push('/')
  } catch (e) {
    notify(e)
    step.value = 'welcome'
  }
}

async function onStartLegacyMigration(): Promise<void> {
  if (!selectedDbPath.value) return

  step.value = 'importing'
  try {
    const summary = await sistema.runLegacyImport(selectedDbPath.value, allowOrphans.value)
    importSummary.value = summary
    step.value = 'summary'
  } catch (e) {
    notify(e)
    step.value = 'confirm'
  }
}

async function onFinish(): Promise<void> {
  localStorage.setItem('eo:welcomed', 'true')
  await router.push('/')
}
</script>

<template>
  <div class="flex min-h-screen items-center justify-center bg-background p-6">
    <!-- Step 1: Welcome & Choices -->
    <div
      v-if="step === 'welcome'"
      class="flex w-full max-w-xl flex-col items-center gap-6 text-center"
    >
      <div>
        <h1 class="text-3xl font-bold">{{ $t('Welcome.Title') }}</h1>
        <p class="mt-2 text-muted-foreground">{{ $t('Welcome.Subtitle') }}</p>
      </div>

      <!-- Detected Legacy Database Card -->
      <div
        v-if="detectedDb"
        class="w-full rounded-xl border-2 border-primary/40 bg-surface-raised p-5 text-left shadow-sm"
      >
        <div class="flex items-start gap-4">
          <div class="rounded-lg bg-primary/10 p-3 text-primary">
            <AppIcon name="database" :size="28" />
          </div>
          <div class="flex-1">
            <h2 class="font-semibold text-foreground">
              {{ $t('Welcome.LegacyDetectedTitle') }}
            </h2>
            <p class="mt-1 break-all text-xs text-muted-foreground">
              {{ detectedDb.path }}
            </p>
            <div class="mt-2 flex items-center gap-3 text-xs text-muted-foreground">
              <span>{{ $t('Welcome.Size') }}: {{ formatBytes(detectedDb.sizeBytes) }}</span>
            </div>
          </div>
        </div>

        <div class="mt-4 flex flex-col gap-2 sm:flex-row sm:justify-end">
          <button
            class="flex items-center justify-center gap-2 rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground transition-opacity hover:opacity-90"
            @click="onSelectDetected"
          >
            <AppIcon name="sparkles" :size="16" />
            {{ $t('Welcome.MigrateDetected') }}
          </button>
        </div>
      </div>

      <!-- Options Grid -->
      <div class="grid w-full grid-cols-1 gap-4 sm:grid-cols-2">
        <button
          class="flex flex-col items-center gap-3 rounded-lg border border-border bg-surface-raised p-5 text-center transition-colors hover:border-primary"
          @click="onStartFresh"
        >
          <AppIcon name="plus-circle" :size="28" class="text-primary" />
          <div>
            <p class="font-medium text-foreground">{{ $t('Welcome.Start') }}</p>
            <p class="mt-1 text-xs text-muted-foreground">{{ $t('Welcome.StartHint') }}</p>
          </div>
        </button>

        <button
          class="flex flex-col items-center gap-3 rounded-lg border border-border bg-surface-raised p-5 text-center transition-colors hover:border-primary"
          @click="onSelectOtherDb"
        >
          <AppIcon name="folder" :size="28" class="text-primary" />
          <div>
            <p class="font-medium text-foreground">{{ $t('Welcome.SelectOtherDb') }}</p>
            <p class="mt-1 text-xs text-muted-foreground">{{ $t('Welcome.ImportHint') }}</p>
          </div>
        </button>
      </div>

      <button
        class="text-xs text-muted-foreground underline hover:text-foreground"
        @click="onImportJson"
      >
        {{ $t('Welcome.ImportJson') }}
      </button>
    </div>

    <!-- Step 2: Confirmation & Migration Parameters -->
    <div
      v-else-if="step === 'confirm'"
      class="flex w-full max-w-lg flex-col gap-6 rounded-xl border border-border bg-surface-raised p-6 shadow-sm"
    >
      <div>
        <h2 class="text-xl font-bold text-foreground">{{ $t('Welcome.Import') }}</h2>
        <p class="mt-1 text-sm text-muted-foreground">{{ $t('Welcome.ImportHint') }}</p>
      </div>

      <div class="rounded-lg bg-surface p-4 text-left">
        <label class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
          {{ $t('Welcome.Source') }}
        </label>
        <p class="mt-1 break-all font-mono text-sm text-foreground">{{ selectedDbPath }}</p>
        <p v-if="selectedDbSize > 0" class="mt-1 text-xs text-muted-foreground">
          {{ $t('Welcome.Size') }}: {{ formatBytes(selectedDbSize) }}
        </p>
      </div>

      <label class="flex items-start gap-3 rounded-lg border border-border p-3 text-left">
        <input
          v-model="allowOrphans"
          type="checkbox"
          class="mt-1 h-4 w-4 rounded border-border text-primary focus:ring-primary"
        />
        <div>
          <span class="text-sm font-medium text-foreground">{{ $t('Welcome.AllowOrphans') }}</span>
          <p class="text-xs text-muted-foreground">{{ $t('Welcome.AllowOrphansHint') }}</p>
        </div>
      </label>

      <div class="flex items-center justify-between gap-4 pt-2">
        <button
          class="rounded-lg border border-border px-4 py-2 text-sm font-medium text-foreground hover:bg-surface"
          @click="step = 'welcome'"
        >
          {{ $t('Welcome.Back') }}
        </button>

        <button
          class="flex items-center gap-2 rounded-lg bg-primary px-5 py-2 text-sm font-medium text-primary-foreground hover:opacity-90"
          @click="onStartLegacyMigration"
        >
          <AppIcon name="play" :size="16" />
          {{ $t('Welcome.Import') }}
        </button>
      </div>
    </div>

    <!-- Step 3: Migration Progress -->
    <div
      v-else-if="step === 'importing'"
      class="flex w-full max-w-md flex-col items-center gap-6 rounded-xl border border-border bg-surface-raised p-8 text-center shadow-sm"
    >
      <div class="animate-spin text-primary">
        <AppIcon name="refresh-cw" :size="48" />
      </div>
      <div>
        <h2 class="text-xl font-bold text-foreground">{{ $t('Welcome.ImportProgress') }}</h2>
        <p class="mt-2 text-sm text-muted-foreground">{{ $t('Welcome.ImportProgressHint') }}</p>
      </div>
    </div>

    <!-- Step 4: Summary Report -->
    <div
      v-else-if="step === 'summary' && importSummary"
      class="flex w-full max-w-xl flex-col gap-6 rounded-xl border border-border bg-surface-raised p-6 shadow-sm"
    >
      <div class="flex items-center gap-4">
        <div class="rounded-full bg-success/10 p-3 text-success">
          <AppIcon name="check-circle" :size="32" />
        </div>
        <div>
          <h2 class="text-xl font-bold text-foreground">
            {{
              importSummary.outcome === 'AlreadyMigrated'
                ? $t('Welcome.AlreadyMigratedTitle')
                : $t('Welcome.ImportSuccessTitle')
            }}
          </h2>
          <p class="text-sm text-muted-foreground">
            {{
              importSummary.outcome === 'AlreadyMigrated'
                ? $t('Welcome.AlreadyMigratedSubtitle')
                : $t('Welcome.ImportSuccessSubtitle', { total: importSummary.totalRows })
            }}
          </p>
        </div>
      </div>

      <!-- Tables Count Summary -->
      <div
        v-if="importSummary.outcome !== 'AlreadyMigrated'"
        class="rounded-lg border border-border bg-surface-card p-4"
      >
        <h3 class="mb-3 text-xs font-semibold uppercase tracking-wider text-muted-foreground">
          {{ $t('Welcome.ImportSummaryTables') }}
        </h3>
        <div class="max-h-48 overflow-y-auto divide-y divide-border">
          <div
            v-for="table in importSummary.tables"
            :key="table.target"
            class="flex items-center justify-between py-1.5 text-xs"
          >
            <span class="font-medium text-foreground">{{ table.target }}</span>
            <span class="text-muted-foreground">{{ table.targetRows }} filas</span>
          </div>
        </div>
      </div>

      <!-- Warnings Collapsible if any -->
      <div
        v-if="importSummary && importSummary.warnings && importSummary.warnings.length > 0"
        class="rounded-lg border border-warning/30 bg-warning/5 p-4 text-left"
      >
        <h3 class="text-xs font-semibold text-warning">
          {{ $t('Welcome.ImportWarnings', { count: importSummary.warnings.length }) }}
        </h3>
        <div class="mt-2 max-h-32 overflow-y-auto text-xs text-muted-foreground">
          <div
            v-for="(w, idx) in importSummary.warnings"
            :key="idx"
            class="py-1"
          >
            <span class="font-mono text-warning">[{{ w.table }}]</span> {{ w.detail }}
          </div>
        </div>
      </div>

      <button
        class="flex w-full items-center justify-center gap-2 rounded-lg bg-primary py-2.5 text-sm font-medium text-primary-foreground hover:opacity-90"
        @click="onFinish"
      >
        <AppIcon name="arrow-right" :size="16" />
        {{ $t('Welcome.EnterApp') }}
      </button>
    </div>
  </div>
</template>
