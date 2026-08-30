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

async function cargar(): Promise<void> {
  backups.value = await sistema.loadBackups()
  await sistema.loadSistema()
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
  <div class="flex max-w-2xl flex-col gap-6 p-4">
    <!-- System info -->
    <section v-if="sistema.sistema" class="flex flex-col gap-2">
      <h3 class="text-sm font-medium">{{ $t('Sistema.Title') }}</h3>
      <div class="grid grid-cols-2 gap-2 text-sm">
        <span class="text-muted-foreground">{{ $t('Sistema.Version') }}</span>
        <span>{{ sistema.sistema.version }}</span>
        <span class="text-muted-foreground">{{ $t('Sistema.BaseSaludable') }}</span>
        <span :class="sistema.sistema.baseSaludable ? 'text-positive' : 'text-negative'">
          {{ sistema.sistema.baseSaludable ? $t('Sistema.BaseSaludable') : $t('Sistema.BaseCorrupta') }}
        </span>
        <span class="text-muted-foreground">{{ $t('Sistema.Migraciones') }}</span>
        <span>{{ sistema.sistema.migraciones }}</span>
        <span class="text-muted-foreground">{{ $t('Sistema.Tamano') }}</span>
        <span>{{ formatSize(sistema.sistema.tamanoBytes) }}</span>
      </div>
    </section>

    <!-- Backups -->
    <section class="flex flex-col gap-3">
      <div class="flex items-center justify-between">
        <h3 class="text-sm font-medium">{{ $t('Backup.Title') }}</h3>
        <div class="flex gap-2">
          <Button variant="secondary" size="sm" @click="exportarJson">
            <AppIcon name="download" :size="14" />
            {{ $t('Backup.ExportJson') }}
          </Button>
          <Button variant="secondary" size="sm" @click="importarJson">
            <AppIcon name="upload" :size="14" />
            {{ $t('Backup.ImportJson') }}
          </Button>
          <Button size="sm" :disabled="sistema.loading" @click="crearBackup">
            <AppIcon name="plus" :size="14" />
            {{ $t('Backup.Create') }}
          </Button>
        </div>
      </div>

      <DataTable :value="backups" :empty-message="$t('Backup.Empty')" size="small">
        <Column field="nombre" header="Nombre" />
        <Column field="bytes" header="Tamaño">
          <template #body="{ data }">{{ formatSize(data.bytes) }}</template>
        </Column>
        <Column header="Acciones" style="width: 200px">
          <template #body="{ data }">
            <div class="flex gap-1">
              <Button
                variant="ghost"
                size="sm"
                :disabled="verificando === data.nombre"
                :title="$t('Backup.Verify')"
                @click="verificar(data.nombre)"
              >
                <AppIcon name="shield-check" :size="14" />
              </Button>
              <Button
                variant="ghost"
                size="sm"
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
