/**
 * Mirror of the Rust DTOs. See `docs/11-contratos-tauri.md` §4.
 *
 * Two rules that the rest of the frontend depends on:
 *
 * - A `Money` or a `Decimal4` arrives as a **string** with four decimals (`"12345.6700"`), never as
 *   a number. `number` is IEEE-754 and loses centavos. Nothing in the frontend does arithmetic
 *   with them; if a screen needs a total, the backend sends it.
 * - An instant arrives as ISO-8601 UTC and a civil date as `YYYY-MM-DD`. Conversion to local time
 *   happens only when formatting.
 */

/** Decimal string with exactly four decimals. Formatted for display, never computed with. */
export type Money = string
/** Same encoding as {@link Money}, for percentages, multipliers, quantities and worked days. */
export type Decimal4 = string
/** ISO-8601 UTC with milliseconds, e.g. `2026-08-29T15:04:05.123Z`. */
export type Instant = string
/** Calendar day, `YYYY-MM-DD`. */
export type CivilDate = string
/** Canonical lowercase UUID. */
export type Uuid = string
/** 16-character hex optimistic-concurrency token. Passed back untouched. */
export type RowVersion = string

export interface PagedResult<T> {
  items: T[]
  totalCount: number
  page: number
  size: number
  totalPages: number
  hasPrevious: boolean
  hasNext: boolean
}

export interface PageRequest {
  page: number
  size: number
}

export const PAGE_SIZES = [10, 30, 50, 100, 0] as const
export const DEFAULT_PAGE_SIZE = 30

export type SortDir = 'Asc' | 'Desc'

/** A list request: a module-specific filter plus the paging every list shares. */
export interface ListQuery<F> {
  filtro: F
  /** 1-based; there is no page zero. */
  page: number
  /** `0` means no paging. */
  pageSize: number
  sortBy?: string
  sortDir?: SortDir
}

/** One option of a selector, returned by the `*_lookup` commands. */
export interface LookupItem {
  id: Uuid
  label: string
  /** Extra data the selector needs to render the option: a colour, a rate, a state. */
  meta?: Record<string, string>
}

/**
 * A state plus the moves the backend will accept from it.
 *
 * The interface renders the buttons from `transicionesPermitidas` instead of listing the enum:
 * the rules live in one place and a screen cannot offer a transition that will be refused.
 */
export interface EstadoInfo {
  /** Variant name, e.g. `Emitida`. */
  actual: string
  /** Full i18n key, e.g. `State.Factura.Emitida`. */
  clave: string
  transicionesPermitidas: TransicionPermitida[]
  esTerminal: boolean
}

export interface TransicionPermitida {
  destino: string
  clave: string
  /** i18n key of the button label. */
  accion: string
  /** The move loses information, so the interface asks first. */
  requiereConfirmacion: boolean
}

export interface Audit {
  createdAt: Instant
  updatedAt: Instant | null
  rowVersion: RowVersion
  isDeleted: boolean
  deletedAt: Instant | null
}

// ------------------------------------------------------------------ config

export type Environment = 'development' | 'production'
export type ThemePreference = 'light' | 'dark' | 'system'
export type LogLevel = 'trace' | 'debug' | 'info' | 'warn' | 'error'
export type EmailClient = 'systemDefault' | 'gmail' | 'outlook' | 'yahoo'
export type DashboardPeriod = 'mensual' | 'anual' | 'total'

export interface ApplicationConfig {
  name: string
  environment: Environment
  seedEnabled: boolean
  lastPageSize: number
  theme: ThemePreference
  lastRoute: string
  sidebarExpanded: boolean
  dataDir: string | null
}

export interface LocaleConfig {
  language: string
  formatoFecha: string
  formatoFechaHora: string
  primerDiaSemana: number
  simboloMoneda: string
  separadorMiles: string
  separadorDecimal: string
  decimalesMoneda: number
  decimalesPorcentaje: number
  monedaPorDefecto: string
  zonaHoraria: string
}

export interface DiasPorFrecuencia {
  diario: Decimal4
  semanal: Decimal4
  quincenal: Decimal4
  mensual: Decimal4
}

export interface BusinessConfig {
  nombreComercial: string
  lema: string
  contratista: string
  cuit: string
  direccion: string
  telefono: string
  email: string
  logoPath: string | null
  ivaSugerido: Decimal4
  facturaDiasVencimientoDefault: number
  categoriaProfundidadMaxima: number
  diasPorFrecuencia: DiasPorFrecuencia
}

export interface SettlementConfig {
  multiplicadorSabado: Decimal4
  multiplicadorDomingo: Decimal4
  multiplicadorFeriado: Decimal4
  incluirSabado: boolean
  incluirDomingo: boolean
  incluirFeriado: boolean
  periodoPorDefectoDias: number
  sincronizarFeriadosAlIniciar: boolean
  aniosFeriadosASincronizar: number
}

export interface DashboardConfig {
  lastPeriod: DashboardPeriod
  privacyMode: boolean
  casasDolar: string[]
  cotizacionPorDefecto: string
  topClientesCantidad: number
  topCategoriasCantidad: number
  ultimosMovimientosCantidad: number
  obrasRankingCantidad: number
  /** Percentage drop in income that raises the alert. */
  alertaCaidaIngresosPct: Decimal4
}

export interface ExternalApisConfig {
  dollarUrl: string
  holidayUrl: string
  timeoutSeconds: number
  reintentos: number
  dollarAutoUpdate: boolean
  dollarCacheMinutes: number
}

export interface AttachmentsConfig {
  maxSizeMb: number
  maxTotalMb: number
  trashRetentionDays: number
  extensionesPermitidas: string[]
}

export interface BackupConfig {
  enabled: boolean
  directory: string
  retentionDays: number
  minimoAConservar: number
  maxAgeDays: number
}

export interface CommunicationConfig {
  emailCliente: EmailClient
  gmailUrl: string
  outlookUrl: string
  yahooUrl: string
  codigoPais: string
  whatsAppTemplate: string
  whatsAppLiquidacionTemplate: string
  emailLiquidacionAsunto: string
}

export interface LoggingConfig {
  level: LogLevel
  retentionDays: number
  consoleEnabled: boolean
  filter: string
}

export interface ValidationConfig {
  fechaMinima: CivilDate
  fechaFuturaMaxDias: number
}

export interface ReportConfig {
  font: string
  mostrarLogo: boolean
  mostrarFirmas: boolean
  pieDePagina: string
}

export interface AppConfig {
  application: ApplicationConfig
  locale: LocaleConfig
  business: BusinessConfig
  settlement: SettlementConfig
  dashboard: DashboardConfig
  externalApis: ExternalApisConfig
  attachments: AttachmentsConfig
  backup: BackupConfig
  communication: CommunicationConfig
  logging: LoggingConfig
  validation: ValidationConfig
  report: ReportConfig
}
