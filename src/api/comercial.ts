import { callCommand } from './client'
import type { RentabilidadItem } from './dashboard'
import type { EstadoFactura } from './facturas'
import type { CivilDate, Money, Uuid } from './types'

/** Account statement, ageing of receivables and profitability. See `docs/06` §4.5, §4.6 y §7. */

export interface CuentaCorrienteQuery {
  clienteId: Uuid
  /** Adds the settled invoices, which the screen hides: the statement is about what is owed. */
  incluirPagadas?: boolean
}

export interface CuentaCorrienteFactura {
  id: Uuid
  numero: string
  fecha: CivilDate
  fechaVencimiento: CivilDate | null
  estado: EstadoFactura
  total: Money
  pagado: Money
  saldo: Money
  /** Zero for a settled invoice: a paid row is not in arrears, however late it was. */
  diasMora: number
}

export interface CuentaCorriente {
  clienteId: Uuid
  clienteNombre: string
  totalFacturado: Money
  totalPagado: Money
  saldo: Money
  facturas: CuentaCorrienteFactura[]
}

export interface AntiguedadDeudaQuery {
  /** Defaults to today. */
  fechaCorte?: CivilDate
  clienteId?: Uuid
}

export interface AntiguedadDeudaCliente {
  clienteId: Uuid
  clienteNombre: string
  total: Money
  bucket0a30: Money
  bucket31a60: Money
  bucket61a90: Money
  bucketMas90: Money
}

export interface AntiguedadDeuda {
  fechaCorte: CivilDate
  total: Money
  bucket0a30: Money
  bucket31a60: Money
  bucket61a90: Money
  bucketMas90: Money
  /** Upper bound of each closed bucket, so the columns are labelled from configuration. */
  limites: number[]
  detalle: AntiguedadDeudaCliente[]
}

export function cuentaCorriente(query: CuentaCorrienteQuery): Promise<CuentaCorriente> {
  return callCommand('clientes_cuenta_corriente', { query })
}

export function antiguedadDeuda(query: AntiguedadDeudaQuery): Promise<AntiguedadDeuda> {
  return callCommand('clientes_antiguedad_deuda', { query })
}

export function rentabilidadProyectos(limite?: number): Promise<RentabilidadItem[]> {
  return callCommand('proyectos_rentabilidad', { limite: limite ?? null })
}

export function rentabilidadTrabajos(
  proyectoId?: Uuid,
  limite?: number,
): Promise<RentabilidadItem[]> {
  return callCommand('trabajos_rentabilidad', { proyectoId: proyectoId ?? null, limite: limite ?? null })
}
