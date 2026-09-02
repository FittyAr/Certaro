import { callCommand } from './client'
import type { MovimientoListItem } from './movimientos'
import type { Decimal4, Instant, Money, Uuid } from './types'

/** See `docs/11-contratos-tauri.md` §5.10. */

/**
 * The period the aggregates are read over. `PascalCase` because that is how the enum crosses the
 * IPC; the lowercase `DashboardPeriod` of the configuration is a different type on purpose.
 */
export type PeriodoDashboard = 'Mensual' | 'Anual' | 'Total'

export const PERIODOS: readonly PeriodoDashboard[] = ['Mensual', 'Anual', 'Total'] as const

/** One month of the calendar year in progress, with both signs so the chart draws two series. */
export interface PuntoSerie {
  /** 1 to 12. The label is built here, where the locale lives. */
  mes: number
  ingresos: Money
  gastos: Money
}

/** A ranking row. Serves both top customers and expenses by category. */
export interface TopCliente {
  /** Absent when the grouping is not navigable. */
  id: Uuid | null
  nombre: string
  total: Money
}

export interface RentabilidadItem {
  id: Uuid
  nombre: string
  /** Site the job belongs to; empty when the row is itself a site. */
  contexto: string
  ingresos: Money
  gastos: Money
  rentabilidad: Money
  /** Zero when there is no income: never null, never a division by zero (doc 06 §7.1). */
  margenPorcentaje: Decimal4
}

export interface EstadoSistema {
  version: string
  baseSaludable: boolean
  /** An i18n key, not a sentence (doc 06 §9.10). */
  estado: string
  migraciones: number
  tamanoBytes: number
}

export interface DashboardStats {
  periodo: PeriodoDashboard
  desde: Instant
  hasta: Instant
  totalIngresos: Money
  totalGastos: Money
  balance: Money
  cantidadMovimientos: number
  rentabilidad: Decimal4
  anteriorIngresos: Money
  anteriorGastos: Money
  /** `null` means there is no basis for comparison, and the screen shows a dash (doc 06 §9.5). */
  variacionIngresos: Decimal4 | null
  variacionGastos: Decimal4 | null
  variacionBalance: Decimal4 | null
  clientesActivos: number
  trabajosPendientes: number
  proyectosPausadas: number
  facturasVencidas: number
  liquidacionesPendientes: number
  serieMensual: PuntoSerie[]
  topClientes: TopCliente[]
  gastosPorCategoria: TopCliente[]
  mejoresProyectos: RentabilidadItem[]
  peoresProyectos: RentabilidadItem[]
  ultimosMovimientos: MovimientoListItem[]
  estadoSistema: EstadoSistema
}

export type TipoAlerta =
  | 'FacturasVencidas'
  | 'BalanceNegativo'
  | 'ProyectosPausados'
  | 'LiquidacionesPendientes'
  | 'CaidaIngresos'

export type SeveridadAlerta = 'Info' | 'Warning' | 'Error'

export interface Alerta {
  tipo: TipoAlerta
  /** i18n key of the message, with the count or the amount as its parameter. */
  clave: string
  cantidad: number
  /** Set instead of `cantidad` when the subject of the alert is an amount. */
  monto: Money | null
  severidad: SeveridadAlerta
  /** Route with its filter already applied, so the card is a link (doc 09 §3.1). */
  destino: string
}

export interface Cotizacion {
  /** `oficial`, `blue`, … always lowercase. */
  casa: string
  nombre: string
  compra: Money
  venta: Money
  fechaActualizacion: Instant
  /** Served from the cache because the request failed: the screen shows the date, not an error. */
  desactualizada: boolean
}

export function dashboardStats(periodo: PeriodoDashboard): Promise<DashboardStats> {
  return callCommand('dashboard_stats', { periodo })
}

export function dashboardAlertas(periodo: PeriodoDashboard): Promise<Alerta[]> {
  return callCommand('dashboard_alertas', { periodo })
}

export function cotizacionesGet(): Promise<Cotizacion[]> {
  return callCommand('cotizaciones_get')
}

export function cotizacionesRefresh(): Promise<Cotizacion[]> {
  return callCommand('cotizaciones_refresh')
}
