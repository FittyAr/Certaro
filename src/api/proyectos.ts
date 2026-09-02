import { callCommand } from './client'
import type {
  Audit,
  EstadoInfo,
  ListQuery,
  LookupItem,
  Money,
  PagedResult,
  RowVersion,
  Uuid,
} from './types'

/** See `docs/11-contratos-tauri.md` §5.3. */

export type EstadoProyecto = 'Activa' | 'Pausada' | 'Finalizada' | 'Cancelada'

export interface ProyectoFiltro {
  /** Matched against the name, the number, the address and the locality. */
  texto?: string
  clienteId?: Uuid
  estado?: EstadoProyecto
  /** Shorthand for the two states that mean "still going on". */
  soloActivas?: boolean
}

/**
 * No state here: it only ever changes through {@link transitionProyecto}, so the form has a single,
 * guarded way in.
 */
export interface ProyectoInput {
  numero: number
  nombre: string
  direccion: string | null
  localidad: string | null
  clienteId: Uuid
}

export interface ProyectoListItem {
  id: Uuid
  numero: number
  nombre: string
  direccion: string | null
  localidad: string | null
  clienteId: Uuid
  clienteNombre: string
  estado: EstadoProyecto
  trabajosCount: number
  rentabilidad: Money
  puedeEliminarse: boolean
  rowVersion: RowVersion
}

export interface ProyectoDetalle {
  id: Uuid
  numero: number
  nombre: string
  direccion: string | null
  localidad: string | null
  clienteId: Uuid
  clienteNombre: string
  estado: EstadoInfo
  trabajosCount: number
  rentabilidad: Money
  puedeEliminarse: boolean
  audit: Audit
}

export const PROYECTOS_SORTABLE = [
  'numero',
  'nombre',
  'clienteNombre',
  'estado',
  'trabajosCount',
  'rentabilidad',
  'createdAt',
] as const

export function listProyectos(query: ListQuery<ProyectoFiltro>): Promise<PagedResult<ProyectoListItem>> {
  return callCommand('proyectos_list', { query })
}

export function getProyecto(id: Uuid): Promise<ProyectoDetalle> {
  return callCommand('proyectos_get', { id })
}

export function createProyecto(dto: ProyectoInput): Promise<ProyectoDetalle> {
  return callCommand('proyectos_create', { dto })
}

export function updateProyecto(id: Uuid, dto: ProyectoInput, rowVersion: RowVersion): Promise<ProyectoDetalle> {
  return callCommand('proyectos_update', { id, dto, rowVersion })
}

/** `cascada` answers the confirmation: close the open jobs along with the site. */
export function transitionProyecto(
  id: Uuid,
  destino: EstadoProyecto,
  rowVersion: RowVersion,
  cascada = false,
): Promise<ProyectoDetalle> {
  return callCommand('proyectos_transition', { id, destino, rowVersion, cascada })
}

export function deleteProyecto(id: Uuid, rowVersion: RowVersion): Promise<void> {
  return callCommand('proyectos_delete', { id, rowVersion })
}

export function lookupProyectos(
  clienteId?: Uuid,
  texto?: string,
  limite?: number,
): Promise<LookupItem[]> {
  return callCommand('proyectos_lookup', { clienteId, texto, limite })
}

/** The number the create form pre-fills with, so the user never has to guess it. */
export function siguienteNumeroProyecto(): Promise<number> {
  return callCommand('proyectos_siguiente_numero', {})
}
