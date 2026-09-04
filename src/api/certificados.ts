import { callCommand } from './client'
import type {
  Audit,
  CivilDate,
  Decimal4,
  ListQuery,
  Money,
  PagedResult,
  RowVersion,
  Uuid,
} from './types'

/** See `docs/11-contratos-tauri.md` §5.5. */

export interface CertificadoFiltro {
  ordenTrabajoId?: Uuid
  proyectoId?: Uuid
  trabajoId?: Uuid
  clienteId?: Uuid
  fechaDesde?: CivilDate
  fechaHasta?: CivilDate
}

export interface CertificadoInputItem {
  ordenTrabajoItemId: Uuid
  porcentajeActual: Decimal4
}

export interface CertificadoInput {
  ordenTrabajoId: Uuid
  fecha: CivilDate
  observaciones: string | null
  items: CertificadoInputItem[]
}

export interface CertificadoBorradorItem {
  ordenTrabajoItemId: Uuid
  descripcion: string
  unidad: string
  cantidad: Decimal4
  precioUnitario: Money
  /** Sum of the percentages of the previous certificates of this line. */
  porcentajeAcumuladoAnterior: Decimal4
  /** `100 - porcentajeAcumuladoAnterior`: the ceiling of what can be certified now. */
  porcentajeDisponible: Decimal4
  porcentajeActual: Decimal4
  base: Money
  subtotalAcumuladoAnterior: Money
}

export interface CertificadoBorrador {
  ordenTrabajoId: Uuid
  ordenTitulo: string
  numeroSugerido: number
  trabajoDescripcion: string
  proyectoNombre: string
  clienteNombre: string
  ajusteUocraPorcentaje: Decimal4
  otrosDescuentos: Money
  items: CertificadoBorradorItem[]
}

export interface CertificadoItem {
  id: Uuid
  ordenTrabajoItemId: Uuid
  descripcion: string
  unidad: string
  cantidad: Decimal4
  precioUnitario: Money
  porcentajeAnterior: Decimal4
  porcentajeActual: Decimal4
  porcentajeAcumulado: Decimal4
  subtotalActual: Money
  subtotalAcumulado: Money
}

export interface CertificadoListItem {
  id: Uuid
  numero: number
  fecha: CivilDate
  ordenTrabajoId: Uuid
  ordenTitulo: string
  trabajoId: Uuid
  trabajoDescripcion: string
  proyectoId: Uuid
  proyectoNumero: number
  proyectoNombre: string
  clienteId: Uuid
  clienteNombre: string
  totalCertificado: Money
  totalNeto: Money
  /** Only the last certificate of an order can be voided. */
  esUltimo: boolean
  rowVersion: RowVersion
}

export interface CertificadoDetalle {
  id: Uuid
  numero: number
  fecha: CivilDate
  observaciones: string | null
  ordenTrabajoId: Uuid
  ordenTitulo: string
  trabajoId: Uuid
  trabajoDescripcion: string
  proyectoId: Uuid
  proyectoNumero: number
  proyectoNombre: string
  clienteId: Uuid
  clienteNombre: string
  totalCertificado: Money
  ajusteUocra: Money
  otrosDescuentos: Money
  totalNeto: Money
  items: CertificadoItem[]
  esUltimo: boolean
  audit: Audit
}

export const CERTIFICADOS_SORTABLE = ['numero', 'fecha', 'totalNeto', 'createdAt'] as const

export function listCertificados(
  query: ListQuery<CertificadoFiltro>,
): Promise<PagedResult<CertificadoListItem>> {
  return callCommand('certificados_list', { query })
}

export function getCertificado(id: Uuid): Promise<CertificadoDetalle> {
  return callCommand('certificados_get', { id })
}

export function prepararCertificado(ordenTrabajoId: Uuid): Promise<CertificadoBorrador> {
  return callCommand('certificados_preparar', { ordenTrabajoId })
}

export function createCertificado(dto: CertificadoInput): Promise<CertificadoDetalle> {
  return callCommand('certificados_create', { dto })
}

/** The only editable field of an issued certificate. */
export function updateObservacionesCertificado(
  id: Uuid,
  observaciones: string | null,
  rowVersion: RowVersion,
): Promise<CertificadoDetalle> {
  return callCommand('certificados_update_observaciones', { id, observaciones, rowVersion })
}

export function deleteCertificado(id: Uuid, rowVersion: RowVersion): Promise<void> {
  return callCommand('certificados_delete', { id, rowVersion })
}
