import { callCommand } from './client'
import type { AppConfig, Instant } from './types'

/** See `docs/11-contratos-tauri.md` §5.13 and `docs/13-servicios-externos-y-archivos.md`. */

export interface BackupItem {
  nombre: string
  creadoEn: Instant
  bytes: number
}

export interface VerificacionBackup {
  ok: boolean
  /** What `PRAGMA integrity_check` answered, verbatim. */
  detalle: string
}

export interface ImportResumen {
  tablas: number
  filas: number
}

export interface EstadoSistema {
  version: string
  baseSaludable: boolean
  /** An i18n key, not a sentence. */
  estado: string
  migraciones: number
  tamanoBytes: number
}

/** Partial config update: only the keys that changed. */
export type Cambios = Record<string, string>

// ── Config ──────────────────────────────────────────────────────────────────

export function configGetAll(): Promise<AppConfig> {
  return callCommand('config_get_all')
}

export function configSet(cambios: Cambios): Promise<AppConfig> {
  return callCommand('config_set', { cambios })
}

export function configReset(claves: string[]): Promise<AppConfig> {
  return callCommand('config_reset', { claves })
}

// ── Sistema ─────────────────────────────────────────────────────────────────

export function sistemaInfo(): Promise<EstadoSistema> {
  return callCommand('sistema_info')
}

// ── Backup ──────────────────────────────────────────────────────────────────

export function backupList(): Promise<BackupItem[]> {
  return callCommand('backup_list')
}

export function backupCreate(): Promise<BackupItem> {
  return callCommand('backup_create')
}

export function backupVerify(nombre: string): Promise<VerificacionBackup> {
  return callCommand('backup_verify', { nombre })
}

export function backupRestore(nombre: string): Promise<void> {
  return callCommand('backup_restore', { nombre })
}

export function backupExportJson(destino: string): Promise<ImportResumen> {
  return callCommand('backup_export_json', { destino })
}

export function backupImportJson(origen: string): Promise<ImportResumen> {
  return callCommand('backup_import_json', { origen })
}

// ── Migración de datos legados ──────────────────────────────────────────────

export interface LegacyDbCandidate {
  path: string
  filename: string
  sizeBytes: number
  modifiedAt?: string | null
}

export interface LegacyImportWarning {
  code: string
  table: string
  rowId?: string | null
  detail: string
}

export interface LegacyImportTable {
  source: string
  target: string
  sourceRows: number
  targetRows: number
}

export interface LegacyImportSummary {
  success: boolean
  outcome: string
  totalRows: number
  warningsCount: number
  blockingIssues: string[]
  warnings: LegacyImportWarning[]
  tables: LegacyImportTable[]
}

export function sistemaDetectLegacyDb(): Promise<LegacyDbCandidate | null> {
  return callCommand('sistema_detect_legacy_db')
}

export function sistemaRunLegacyImport(
  origen: string,
  allowOrphans?: boolean,
): Promise<LegacyImportSummary> {
  return callCommand('sistema_run_legacy_import', { origen, allowOrphans })
}

// ── Sembrado de datos de prueba ───────────────────────────────────────────

export interface SeedSummary {
  categorias: number
  tiposMovimiento: number
  empleados: number
  clientes: number
  proyectos: number
  trabajos: number
  ordenesTrabajo: number
  movimientos: number
  facturas: number
  liquidaciones: number
}

export function devSeedDatabase(): Promise<SeedSummary> {
  return callCommand('dev_seed_database')
}

