import { callCommand } from './client'
import type { Audit, CivilDate, Decimal4, LookupItem, Money, RowVersion, Uuid } from './types'

/** See `docs/11-contratos-tauri.md` §5.4. */

export interface OrdenTrabajoItemInput {
  /** Absent on a new line. */
  id: Uuid | null
  descripcion: string
  unidad: string
  cantidad: Decimal4
  precioUnitario: Money
  /**
   * Progress of the certificate being prepared. There is no `porcentajeAnterior`: the history is
   * written only by issuing or voiding a certificate.
   */
  porcentajeActual: Decimal4
  ejecutado: boolean
  nota: string | null
}

export interface OrdenTrabajoInput {
  trabajoId: Uuid
  titulo: string
  fecha: CivilDate
  observaciones: string | null
  /** A percentage, not an amount: `8` means 8 %. */
  ajusteUocraPorcentaje: Decimal4
  otrosDescuentos: Money
  items: OrdenTrabajoItemInput[]
}

export interface OrdenTrabajoItem {
  id: Uuid
  descripcion: string
  unidad: string
  cantidad: Decimal4
  precioUnitario: Money
  porcentajeAnterior: Decimal4
  porcentajeActual: Decimal4
  porcentajeAcumulado: Decimal4
  porcentajePendiente: Decimal4
  base: Money
  subtotalActual: Money
  subtotalAcumulado: Money
  ejecutado: boolean
  nota: string | null
  orden: number
  /** Already part of some certificate, so the line cannot be removed. */
  certificado: boolean
}

export interface OrdenTrabajoListItem {
  id: Uuid
  trabajoId: Uuid
  trabajoDescripcion?: string
  proyectoId?: Uuid
  proyectoNumero?: number
  proyectoNombre?: string
  clienteId?: Uuid
  clienteNombre?: string
  titulo: string
  numeroCertificado: string | null
  fecha: CivilDate
  itemsCount: number
  totalPresupuestado: Money
  totalPresupuestadoNeto?: Money
  totalCertificado: Money
  certificadosCount: number
  rowVersion: RowVersion
}

export interface OrdenTrabajoDetalle {
  id: Uuid
  trabajoId: Uuid
  trabajoDescripcion: string
  proyectoId: Uuid
  proyectoNumero: number
  proyectoNombre: string
  clienteId: Uuid
  clienteNombre: string
  titulo: string
  numeroCertificado: string | null
  fecha: CivilDate
  observaciones: string | null
  ajusteUocraPorcentaje: Decimal4
  otrosDescuentos: Money
  items: OrdenTrabajoItem[]
  totalPresupuestado: Money
  ajusteUocraPresupuestado?: Money
  totalPresupuestadoNeto?: Money
  totalCertificado: Money
  ajusteUocra: Money
  totalNeto: Money
  certificadosCount: number
  puedeEliminarse: boolean
  audit: Audit
}

/** Not paged: a job has a handful of sheets, not thousands. If trabajoId is omitted, returns all orders. */
export function listOrdenesTrabajo(trabajoId?: Uuid): Promise<OrdenTrabajoListItem[]> {
  return callCommand('ordenes_trabajo_list', { trabajoId: trabajoId || null })
}

export function getOrdenTrabajo(id: Uuid): Promise<OrdenTrabajoDetalle> {
  return callCommand('ordenes_trabajo_get', { id })
}

export function createOrdenTrabajo(dto: OrdenTrabajoInput): Promise<OrdenTrabajoDetalle> {
  return callCommand('ordenes_trabajo_create', { dto })
}

export function updateOrdenTrabajo(
  id: Uuid,
  dto: OrdenTrabajoInput,
  rowVersion: RowVersion,
): Promise<OrdenTrabajoDetalle> {
  return callCommand('ordenes_trabajo_update', { id, dto, rowVersion })
}

export function deleteOrdenTrabajo(id: Uuid, rowVersion: RowVersion): Promise<void> {
  return callCommand('ordenes_trabajo_delete', { id, rowVersion })
}

export function lookupOrdenesTrabajo(
  trabajoId?: Uuid,
  texto?: string,
  limite?: number,
): Promise<LookupItem[]> {
  return callCommand('ordenes_trabajo_lookup', { trabajoId, texto, limite })
}
