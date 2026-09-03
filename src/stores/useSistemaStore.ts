import { defineStore } from 'pinia'
import { ref } from 'vue'
import { useStorage } from '@vueuse/core'

import {
  backupCreate,
  backupExportJson,
  backupImportJson,
  backupList,
  backupRestore,
  backupVerify,
  configGetAll,
  configReset,
  configSet,
  devSeedDatabase,
  sistemaDetectLegacyDb,
  sistemaInfo,
  sistemaRunLegacyImport,
  type BackupItem,
  type Cambios,
  type EstadoSistema,
  type ImportResumen,
  type LegacyDbCandidate,
  type LegacyImportSummary,
  type SeedSummary,
  type VerificacionBackup,
} from '@/api/sistema'
import type { AppConfig } from '@/api/types'

export type {
  BackupItem,
  Cambios,
  EstadoSistema,
  ImportResumen,
  LegacyDbCandidate,
  LegacyImportSummary,
  SeedSummary,
  VerificacionBackup,
}

/**
 * System operations: configuration, backups, JSON export/import. See `docs/13` §4–5.
 *
 * The config part extends what `useConfigStore` loads at startup with write capabilities. The two
 * stores share the same `AppConfig` shape but serve different moments: `useConfigStore` is the
 * read-only snapshot the UI consumes, `useSistemaStore` is the settings screen that writes.
 */
export const useSistemaStore = defineStore('sistema', () => {
  const config = ref<AppConfig | null>(null)
  const sistema = ref<EstadoSistema | null>(null)
  const backups = ref<BackupItem[]>([])
  const loading = ref(false)
  const mostrarColumnaNumeroProyectos = useStorage('certaro_mostrar_columna_numero_proyectos', false)

  // ── Config ──────────────────────────────────────────────────────────────

  async function loadConfig(): Promise<AppConfig> {
    config.value = await configGetAll()
    return config.value
  }

  async function applyConfig(cambios: Cambios): Promise<AppConfig> {
    config.value = await configSet(cambios)
    return config.value
  }

  async function resetConfig(claves: string[]): Promise<AppConfig> {
    config.value = await configReset(claves)
    return config.value
  }

  // ── Sistema ─────────────────────────────────────────────────────────────

  async function loadSistema(): Promise<EstadoSistema> {
    sistema.value = await sistemaInfo()
    return sistema.value
  }

  // ── Backup ──────────────────────────────────────────────────────────────

  async function loadBackups(): Promise<BackupItem[]> {
    backups.value = await backupList()
    return backups.value
  }

  async function createBackup(): Promise<BackupItem> {
    loading.value = true
    try {
      const item = await backupCreate()
      backups.value.unshift(item)
      return item
    } finally {
      loading.value = false
    }
  }

  async function verifyBackup(nombre: string): Promise<VerificacionBackup> {
    return backupVerify(nombre)
  }

  async function restoreBackup(nombre: string): Promise<void> {
    loading.value = true
    try {
      await backupRestore(nombre)
    } finally {
      loading.value = false
    }
  }

  async function exportJson(destino: string): Promise<ImportResumen> {
    loading.value = true
    try {
      return await backupExportJson(destino)
    } finally {
      loading.value = false
    }
  }

  async function importJson(origen: string): Promise<ImportResumen> {
    loading.value = true
    try {
      return await backupImportJson(origen)
    } finally {
      loading.value = false
    }
  }

  // ── Migración de datos legados ──────────────────────────────────────────

  async function detectLegacyDb(): Promise<LegacyDbCandidate | null> {
    return await sistemaDetectLegacyDb()
  }

  async function runLegacyImport(
    origen: string,
    allowOrphans = true,
  ): Promise<LegacyImportSummary> {
    loading.value = true
    try {
      return await sistemaRunLegacyImport(origen, allowOrphans)
    } finally {
      loading.value = false
    }
  }

  // ── Sembrado de datos de prueba ───────────────────────────────────────────

  async function seedDemoData(): Promise<SeedSummary> {
    loading.value = true
    try {
      return await devSeedDatabase()
    } finally {
      loading.value = false
    }
  }

  return {
    config,
    sistema,
    backups,
    loading,
    loadConfig,
    applyConfig,
    resetConfig,
    loadSistema,
    loadBackups,
    createBackup,
    verifyBackup,
    restoreBackup,
    exportJson,
    importJson,
    detectLegacyDb,
    runLegacyImport,
    seedDemoData,
    mostrarColumnaNumeroProyectos,
  }
})
