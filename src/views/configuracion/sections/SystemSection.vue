<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog'
import Column from 'primevue/column'
import DataTable from 'primevue/datatable'
import { onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useToast } from 'primevue/usetoast'

import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import { useConfirmDelete } from '@/composables/useConfirmDelete'
import { useExport } from '@/composables/useExport'
import { useSistemaStore, type BackupItem } from '@/stores/useSistemaStore'

/**
 * System section of settings. See `docs/09` §3.15 and `docs/13` §4–5.
 *
 * Backups, JSON export/import, and system info. Restore and import are destructive: the backend
 * takes an automatic backup before either runs, and the frontend confirms twice.
 */

const { t } = useI18n()
const toast = useToast()
const { confirmDelete } = useConfirmDelete()
const sistema = useSistemaStore()
const { exportar } = useExport()

const backups = ref<BackupItem[]>([])
const verificando = ref<string | null>(null)
const isDev = import.meta.env.DEV
const sembrando = ref(false)

async function cargar(): Promise<void> {
  backups.value = await sistema.loadBackups()
  await sistema.loadSistema()
}

async function sembrarDatos(): Promise<void> {
  sembrando.value = true
  try {
    const resumen = await sistema.seedDemoData()
    await sistema.loadSistema()
    // Idempotent seed returns all zeros when data already exists
    const isNoop = (resumen.clientes + resumen.obras + resumen.movimientos) === 0
    if (isNoop) {
      toast.add({
        severity: 'info',
        summary: t('General.Success') ?? t('Seed.Success'),
        detail: t('Seed.AlreadySeeded'),
        life: 5000,
      })
      return
    }
    toast.add({
      severity: 'success',
      summary: t('Seed.Success'),
      detail: `${resumen.clientes} clientes, ${resumen.obras} obras, ${resumen.movimientos} movimientos`,
      life: 4000,
    })
  } catch (err: unknown) {
    const api = (err as { messageKey?: string })?.messageKey
      ? (err as { messageKey: string; params?: Record<string, string> })
      : null
    toast.add({
      severity: 'error',
      summary: t('General.Error'),
      detail: api ? t(api.messageKey, api.params ?? {}) : String(err),
      life: 5000,
    })
  } finally {
    sembrando.value = false
  }
}

async function crearBackup(): Promise<void> {
  const item = await sistema.createBackup()
  backups.value.unshift(item)
  toast.add({ severity: 'success', summary: t('Backup.Created'), detail: item.nombre, life: 3000 })
}

async function verificar(nombre: string): Promise<void> {
  verificando.value = nombre
  try {
    const resultado = await sistema.verifyBackup(nombre)
    toast.add({
      severity: resultado.ok ? 'success' : 'error',
      summary: resultado.ok ? t('Backup.VerifyOk') : t('Backup.VerifyFail'),
      detail: resultado.detalle,
      life: 5000,
    })
  } finally {
    verificando.value = null
  }
}

function restaurar(nombre: string): void {
  confirmDelete({
    entityKey: 'Backup.Restore',
    label: nombre,
    action: async () => {
      await sistema.restoreBackup(nombre)
      toast.add({ severity: 'success', summary: t('Backup.RestoreDone'), life: 5000 })
    },
  })
}

async function exportarJson(): Promise<void> {
  await exportar({
    reporte: 'backup',
    formato: 'Json',
    run: async (destino) => {
      const resumen = await sistema.exportJson(destino)
      return { ruta: destino, bytes: 0, registros: resumen.filas }
    },
  })
}

async function importarJson(): Promise<void> {
  const origen = await open({
    multiple: false,
    title: t('Backup.ImportJson'),
    filters: [{ name: 'JSON', extensions: ['json'] }],
  })
  if (!origen) return

  confirmDelete({
    entityKey: 'Backup.ImportJson',
    label: origen,
    action: async () => {
      const resumen = await sistema.importJson(origen)
      toast.add({
        severity: 'success',
        summary: t('Backup.ImportJson'),
        detail: `${resumen.tablas} tablas, ${resumen.filas} filas`,
        life: 5000,
      })
    },
  })
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

onMounted(() => void cargar())
</script>

<template>
  <div class="flex max-w-4xl flex-col gap-6">
    <!-- Estado del Sistema -->
    <section v-if="sistema.sistema" class="rounded-lg border border-border bg-surface-card p-6 shadow-sm">
      <div class="mb-4 flex items-center gap-2 border-b border-border pb-3">
        <AppIcon name="database" :size="18" class="text-primary" />
        <h3 class="text-sm font-semibold text-foreground">{{ $t('Sistema.Title') }}</h3>
      </div>

      <dl class="grid grid-cols-2 gap-4 text-sm sm:grid-cols-4">
        <div class="flex flex-col gap-1 rounded-lg border border-border bg-surface-raised p-3">
          <dt class="text-xs text-muted-foreground">{{ $t('Sistema.Version') }}</dt>
          <dd class="font-medium text-foreground tabular-nums">{{ sistema.sistema.version }}</dd>
        </div>

        <div class="flex flex-col gap-1 rounded-lg border border-border bg-surface-raised p-3">
          <dt class="text-xs text-muted-foreground">{{ $t('Dashboard.EstadoBase') }}</dt>
          <dd :class="sistema.sistema.baseSaludable ? 'text-success font-medium' : 'text-destructive font-medium'">
            {{ sistema.sistema.baseSaludable ? $t('Sistema.BaseSaludable') : $t('Sistema.BaseCorrupta') }}
          </dd>
        </div>

        <div class="flex flex-col gap-1 rounded-lg border border-border bg-surface-raised p-3">
          <dt class="text-xs text-muted-foreground">{{ $t('Sistema.Migraciones') }}</dt>
          <dd class="font-medium text-foreground tabular-nums">{{ sistema.sistema.migraciones }}</dd>
        </div>

        <div class="flex flex-col gap-1 rounded-lg border border-border bg-surface-raised p-3">
          <dt class="text-xs text-muted-foreground">{{ $t('Sistema.Tamano') }}</dt>
          <dd class="font-medium text-foreground tabular-nums">{{ formatSize(sistema.sistema.tamanoBytes) }}</dd>
        </div>
      </dl>
    </section>

    <!-- Sembrado de datos de prueba (solo visible en DEV) -->
    <section v-if="isDev" class="rounded-lg border border-border bg-surface-card p-6 shadow-sm">
      <div class="mb-4 flex items-center justify-between border-b border-border pb-3">
        <div class="flex items-center gap-2">
          <AppIcon name="database" :size="18" class="text-primary" />
          <h3 class="text-sm font-semibold text-foreground">{{ $t('Seed.Title') }}</h3>
        </div>
        <span class="rounded bg-muted px-2 py-0.5 text-xs font-medium text-muted-foreground uppercase tracking-wider">
          DEV ONLY
        </span>
      </div>

      <p class="mb-4 text-xs text-muted-foreground leading-relaxed">
        {{ $t('Seed.WarningText') }}
      </p>

      <Button
        variant="outline"
        size="sm"
        :disabled="sembrando"
        class="flex items-center gap-2"
        @click="sembrarDatos"
      >
        <AppIcon v-if="!sembrando" name="play" :size="14" />
        <AppIcon v-else name="loader" :size="14" class="animate-spin" />
        <span>{{ sembrando ? $t('Seed.Loading') : $t('Seed.Button') }}</span>
      </Button>
    </section>

    <!-- Copias de Seguridad -->
    <section class="rounded-lg border border-border bg-surface-card p-6 shadow-sm">
      <div class="mb-4 flex flex-wrap items-center justify-between gap-3 border-b border-border pb-3">
        <div class="flex items-center gap-2">
          <AppIcon name="shield" :size="18" class="text-primary" />
          <h3 class="text-sm font-semibold text-foreground">{{ $t('Backup.Title') }}</h3>
        </div>

        <div class="flex flex-wrap gap-2">
          <Button variant="outline" size="sm" class="flex items-center gap-1.5" @click="exportarJson">
            <AppIcon name="download" :size="14" />
            {{ $t('Backup.ExportJson') }}
          </Button>
          <Button variant="outline" size="sm" class="flex items-center gap-1.5" @click="importarJson">
            <AppIcon name="upload" :size="14" />
            {{ $t('Backup.ImportJson') }}
          </Button>
          <Button size="sm" class="flex items-center gap-1.5" :disabled="sistema.loading" @click="crearBackup">
            <AppIcon name="plus" :size="14" />
            {{ $t('Backup.Create') }}
          </Button>
        </div>
      </div>

      <DataTable :value="backups" :empty-message="$t('Backup.Empty')" size="small" class="text-sm">
        <Column field="nombre" :header="$t('Adjuntos.Nombre')" />
        <Column field="bytes" :header="$t('Adjuntos.Tamano')">
          <template #body="{ data }">{{ formatSize(data.bytes) }}</template>
        </Column>
        <Column :header="$t('General.Actions')" :style="{ width: '120px' }">
          <template #body="{ data }">
            <div class="flex gap-1">
              <Button
                variant="ghost"
                size="icon"
                :disabled="verificando === data.nombre"
                :title="$t('Backup.Verify')"
                @click="verificar(data.nombre)"
              >
                <AppIcon name="shield-check" :size="14" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                :title="$t('Backup.Restore')"
                @click="restaurar(data.nombre)"
              >
                <AppIcon name="rotate-ccw" :size="14" />
              </Button>
            </div>
          </template>
        </Column>
      </DataTable>
    </section>
  </div>
</template>
