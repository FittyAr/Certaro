import { callCommand } from './client'
import type {
  Audit,
  CivilDate,
  EstadoInfo,
  ListQuery,
  LookupItem,
  Money,
  PagedResult,
  RowVersion,
  Uuid,
} from './types'

/** See `docs/11-contratos-tauri.md` §5.6. */

export type EstadoFactura =
  'Borrador' | 'Emitida' | 'Pagada' | 'Anulada' | 'Vencida' | 'PagadaParcial'

export interface FacturaFiltro {
  /** Matched against the number and the customer name. */
  texto?: string
  clienteId?: Uuid
  /** Empty means every state; the screen offers a multi-select. */
  estados?: EstadoFactura[]
  fechaDesde?: CivilDate
  fechaHasta?: CivilDate
  soloImpagas?: boolean
  soloVencidas?: boolean
}

/**
 * `total` is sent because the form shows it, but the backend overwrites it with `subtotal + iva`.
 */
export interface FacturaInput {
  numero: string
  fecha: CivilDate
  fechaVencimiento: CivilDate | null
  clienteId: Uuid
  subtotal: Money
  iva: Money
  total: Money
  observaciones: string | null
}

export interface PagoFacturaInput {
  facturaId: Uuid
  fecha: CivilDate
  monto: Money
  medioPago: string
}

export interface PagoFacturaItem {
  id: Uuid
  facturaId: Uuid
  fecha: CivilDate
  monto: Money
  medioPago: string
  rowVersion: RowVersion
}

export interface FacturaListItem {
  id: Uuid
  numero: string
  fecha: CivilDate
  fechaVencimiento: CivilDate | null
  clienteId: Uuid
  clienteNombre: string
  estado: EstadoFactura
  subtotal: Money
  iva: Money
  total: Money
  pagado: Money
  saldo: Money
  diasMora: number
  rowVersion: RowVersion
}

export interface FacturaDetalle {
  id: Uuid
  numero: string
  fecha: CivilDate
  fechaVencimiento: CivilDate | null
  clienteId: Uuid
  clienteNombre: string
  estado: EstadoInfo
  subtotal: Money
  iva: Money
  total: Money
  pagado: Money
  saldo: Money
  diasMora: number
  observaciones: string | null
  pagos: PagoFacturaItem[]
  /** False when the state takes no money, so the payment form is disabled up front. */
  admitePagos: boolean
  puedeEliminarse: boolean
  audit: Audit
}

export const FACTURAS_SORTABLE = [
  'fecha',
  'numero',
  'clienteNombre',
  'estado',
  'total',
  'pagado',
  'saldo',
  'fechaVencimiento',
  'createdAt',
] as const

/** The options the payment form offers. A historical value outside this list is kept as it is. */
export const MEDIOS_PAGO = [
  'Efectivo',
  'Transferencia',
  'Cheque',
  'TarjetaDebito',
  'TarjetaCredito',
  'MercadoPago',
  'Otro',
] as const

export function listFacturas(
  query: ListQuery<FacturaFiltro>,
): Promise<PagedResult<FacturaListItem>> {
  return callCommand('facturas_list', { query })
}

export function getFactura(id: Uuid): Promise<FacturaDetalle> {
  return callCommand('facturas_get', { id })
}

export function createFactura(dto: FacturaInput): Promise<FacturaDetalle> {
  return callCommand('facturas_create', { dto })
}

export function updateFactura(
  id: Uuid,
  dto: FacturaInput,
  rowVersion: RowVersion,
): Promise<FacturaDetalle> {
  return callCommand('facturas_update', { id, dto, rowVersion })
}

export function transitionFactura(
  id: Uuid,
  destino: EstadoFactura,
  rowVersion: RowVersion,
): Promise<FacturaDetalle> {
  return callCommand('facturas_transition', { id, destino, rowVersion })
}

export function deleteFactura(id: Uuid, rowVersion: RowVersion): Promise<void> {
  return callCommand('facturas_delete', { id, rowVersion })
}

export function lookupFacturas(
  clienteId?: Uuid,
  soloImpagas = false,
  texto?: string,
  limite?: number,
): Promise<LookupItem[]> {
  return callCommand('facturas_lookup', { clienteId, soloImpagas, texto, limite })
}

export function listPagos(facturaId: Uuid): Promise<PagoFacturaItem[]> {
  return callCommand('facturas_pagos', { facturaId })
}

// Every payment command answers with the whole invoice: a payment moves the balance and can move
// the state, so returning only the payment would leave the screen showing stale totals.

export function createPago(dto: PagoFacturaInput): Promise<FacturaDetalle> {
  return callCommand('facturas_pago_create', { dto })
}

export function updatePago(
  id: Uuid,
  dto: PagoFacturaInput,
  rowVersion: RowVersion,
): Promise<FacturaDetalle> {
  return callCommand('facturas_pago_update', { id, dto, rowVersion })
}

export function deletePago(id: Uuid, rowVersion: RowVersion): Promise<FacturaDetalle> {
  return callCommand('facturas_pago_delete', { id, rowVersion })
}
