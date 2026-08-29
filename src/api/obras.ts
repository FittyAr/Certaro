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

export type EstadoObra = 'Activa' | 'Pausada' | 'Finalizada' | 'Cancelada'

export interface ObraFiltro {
  /** Matched against the name, the number, the address and the locality. */
  texto?: string
  clienteId?: Uuid
  estado?: EstadoObra
  /** Shorthand for the two states that mean "still going on". */
  soloActivas?: boolean
}

/**
 * No state here: it only ever changes through {@link transitionObra}, so the form has a single,
 * guarded way in.
 */
export interface ObraInput {
  numero: number
  nombre: string
  direccion: string | null
  localidad: string | null
  clienteId: Uuid
}

export interface ObraListItem {
  id: Uuid
  numero: number
  nombre: string
  direccion: string | null
  localidad: string | null
  clienteId: Uuid
  clienteNombre: string
  estado: EstadoObra
  trabajosCount: number
  rentabilidad: Money
  puedeEliminarse: boolean
  rowVersion: RowVersion
}

export interface ObraDetalle {
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

export const OBRAS_SORTABLE = [
  'numero',
  'nombre',
  'clienteNombre',
  'estado',
  'trabajosCount',
  'rentabilidad',
  'createdAt',
] as const

export function listObras(query: ListQuery<ObraFiltro>): Promise<PagedResult<ObraListItem>> {
  return callCommand('obras_list', { query })
}

export function getObra(id: Uuid): Promise<ObraDetalle> {
  return callCommand('obras_get', { id })
}

export function createObra(dto: ObraInput): Promise<ObraDetalle> {
  return callCommand('obras_create', { dto })
}

export function updateObra(id: Uuid, dto: ObraInput, rowVersion: RowVersion): Promise<ObraDetalle> {
  return callCommand('obras_update', { id, dto, rowVersion })
}

/** `cascada` answers the confirmation: close the open jobs along with the site. */
export function transitionObra(
  id: Uuid,
  destino: EstadoObra,
  rowVersion: RowVersion,
  cascada = false,
): Promise<ObraDetalle> {
  return callCommand('obras_transition', { id, destino, rowVersion, cascada })
}

export function deleteObra(id: Uuid, rowVersion: RowVersion): Promise<void> {
  return callCommand('obras_delete', { id, rowVersion })
}

export function lookupObras(
  clienteId?: Uuid,
  texto?: string,
  limite?: number,
): Promise<LookupItem[]> {
  return callCommand('obras_lookup', { clienteId, texto, limite })
}

/** The number the create form pre-fills with, so the user never has to guess it. */
export function siguienteNumeroObra(): Promise<number> {
  return callCommand('obras_siguiente_numero', {})
}
