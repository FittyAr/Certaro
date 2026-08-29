import { callCommand } from './client'
import type {
  CivilDate,
  Decimal4,
  Instant,
  ListQuery,
  Money,
  PagedResult,
  RowVersion,
  Uuid,
} from './types'

/** See `docs/11-contratos-tauri.md` §5.1. */

export type Moneda = 'Ars' | 'Usd'

export interface MovimientoFiltro {
  concepto?: string
  tipoMovimientoId?: Uuid
  categoriaId?: Uuid
  clienteId?: Uuid
  trabajoId?: Uuid
  empleadoId?: Uuid
  facturaId?: Uuid
  moneda?: Moneda
  fechaDesde?: CivilDate
  fechaHasta?: CivilDate
  /** Compared against the unit amount, not against the total. */
  montoMin?: Money
  montoMax?: Money
}

export interface MovimientoInput {
  fecha: Instant
  concepto: string
  monto: Money
  cantidad: Decimal4
  tipoMovimientoId: Uuid
  moneda: Moneda
  /** Required in USD, refused in ARS. */
  cotizacionAplicada: Money | null
  tipoConceptoPagoId: Uuid | null
  categoriaId: Uuid | null
  clienteId: Uuid | null
  trabajoId: Uuid | null
  empleadoId: Uuid | null
  facturaId: Uuid | null
}

export interface MovimientoListItem {
  id: Uuid
  fecha: Instant
  concepto: string
  monto: Money
  cantidad: Decimal4
  /** Derived by the backend as `monto * cantidad`; never stored, never recomputed here. */
  total: Money
  moneda: Moneda
  cotizacionAplicada: Money | null
  tipoMovimientoId: Uuid
  tipoMovimientoNombre: string
  esIngreso: boolean
  categoriaId: Uuid | null
  categoriaNombre: string | null
  categoriaColor: string | null
  clienteId: Uuid | null
  trabajoId: Uuid | null
  empleadoId: Uuid | null
  facturaId: Uuid | null
  tipoConceptoPagoId: Uuid | null
  /** An advance already consumed by a settlement cannot be edited or deleted. */
  bloqueadoPorLiquidacion: boolean
  rowVersion: RowVersion
}

export interface MovimientoDetalle extends MovimientoListItem {
  createdAt: Instant
  updatedAt: Instant | null
}

/** Totals of the whole filter, not of the visible page. */
export interface MovimientoResumen {
  totalIngresos: Money
  totalGastos: Money
  balance: Money
  cantidad: number
}

export interface MovimientoListResult extends PagedResult<MovimientoListItem> {
  resumen: MovimientoResumen
}

/** Columns the backend accepts; anything else is rejected as a validation error. */
export const MOVIMIENTOS_SORTABLE = [
  'fecha',
  'concepto',
  'monto',
  'total',
  'tipoMovimientoNombre',
  'categoriaNombre',
] as const

export function listMovimientos(query: ListQuery<MovimientoFiltro>): Promise<MovimientoListResult> {
  return callCommand('movimientos_list', { query })
}

export function getMovimiento(id: Uuid): Promise<MovimientoDetalle> {
  return callCommand('movimientos_get', { id })
}

export function resumenMovimientos(filtro: MovimientoFiltro): Promise<MovimientoResumen> {
  return callCommand('movimientos_resumen', { filtro })
}

export function createMovimiento(dto: MovimientoInput): Promise<MovimientoDetalle> {
  return callCommand('movimientos_create', { dto })
}

export function updateMovimiento(
  id: Uuid,
  dto: MovimientoInput,
  rowVersion: RowVersion,
): Promise<MovimientoDetalle> {
  return callCommand('movimientos_update', { id, dto, rowVersion })
}

export function deleteMovimiento(id: Uuid, rowVersion: RowVersion): Promise<void> {
  return callCommand('movimientos_delete', { id, rowVersion })
}
