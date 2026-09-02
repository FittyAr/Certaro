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

/** See `docs/11-contratos-tauri.md` §5.4. */

export type EstadoTrabajo = 'Presupuestado' | 'EnProceso' | 'Pausado' | 'Finalizado' | 'Cancelado'

export interface TrabajoFiltro {
  texto?: string
  proyectoId?: Uuid
  /** Resolved through the site: a job carries no customer of its own. */
  clienteId?: Uuid
  estado?: EstadoTrabajo
  fechaDesde?: CivilDate
  fechaHasta?: CivilDate
}

export interface TrabajoInput {
  proyectoId: Uuid
  descripcion: string
  fechaInicio: CivilDate
  fechaFin: CivilDate | null
  presupuesto: Money
}

export interface TrabajoListItem {
  id: Uuid
  proyectoId: Uuid
  proyectoNumero: number
  proyectoNombre: string
  clienteId: Uuid
  clienteNombre: string
  descripcion: string
  fechaInicio: CivilDate
  fechaFin: CivilDate | null
  presupuesto: Money
  estado: EstadoTrabajo
  rowVersion: RowVersion
}

export interface TrabajoDetalle {
  id: Uuid
  proyectoId: Uuid
  proyectoNumero: number
  proyectoNombre: string
  clienteId: Uuid
  clienteNombre: string
  descripcion: string
  fechaInicio: CivilDate
  fechaFin: CivilDate | null
  presupuesto: Money
  estado: EstadoInfo
  puedeEliminarse: boolean
  audit: Audit
}

export const TRABAJOS_SORTABLE = [
  'fechaInicio',
  'descripcion',
  'proyectoNombre',
  'clienteNombre',
  'presupuesto',
  'estado',
  'createdAt',
] as const

export function listTrabajos(
  query: ListQuery<TrabajoFiltro>,
): Promise<PagedResult<TrabajoListItem>> {
  return callCommand('trabajos_list', { query })
}

export function getTrabajo(id: Uuid): Promise<TrabajoDetalle> {
  return callCommand('trabajos_get', { id })
}

export function createTrabajo(dto: TrabajoInput): Promise<TrabajoDetalle> {
  return callCommand('trabajos_create', { dto })
}

export function updateTrabajo(
  id: Uuid,
  dto: TrabajoInput,
  rowVersion: RowVersion,
): Promise<TrabajoDetalle> {
  return callCommand('trabajos_update', { id, dto, rowVersion })
}

export function transitionTrabajo(
  id: Uuid,
  destino: EstadoTrabajo,
  rowVersion: RowVersion,
  forzar = false,
): Promise<TrabajoDetalle> {
  return callCommand('trabajos_transition', { id, destino, rowVersion, forzar })
}

export function deleteTrabajo(id: Uuid, rowVersion: RowVersion): Promise<void> {
  return callCommand('trabajos_delete', { id, rowVersion })
}

export function lookupTrabajos(
  proyectoId?: Uuid,
  texto?: string,
  limite?: number,
): Promise<LookupItem[]> {
  return callCommand('trabajos_lookup', { proyectoId, texto, limite })
}
