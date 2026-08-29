import { callCommand } from './client'
import type { Audit, ListQuery, LookupItem, PagedResult, RowVersion, Uuid } from './types'

/** See `docs/11-contratos-tauri.md` §5.11. */

export interface TipoMovimientoFiltro {
  /** Matched against the name and the description, ignoring case. */
  texto?: string
  esIngreso?: boolean
  esSistema?: boolean
}

export interface TipoMovimientoInput {
  nombre: string
  descripcion: string | null
  esIngreso: boolean
}

export interface TipoMovimientoListItem {
  id: Uuid
  nombre: string
  descripcion: string | null
  esIngreso: boolean
  esSistema: boolean
  movimientosCount: number
  /** False for a seeded row or one already in use, so the action can be disabled up front. */
  puedeEliminarse: boolean
  rowVersion: RowVersion
}

export interface TipoMovimientoDetalle {
  id: Uuid
  nombre: string
  descripcion: string | null
  esIngreso: boolean
  esSistema: boolean
  movimientosCount: number
  puedeEliminarse: boolean
  audit: Audit
}

/** Columns the backend accepts; anything else is rejected as a validation error. */
export const TIPOS_MOVIMIENTO_SORTABLE = [
  'nombre',
  'esIngreso',
  'movimientosCount',
  'createdAt',
] as const

export function listTiposMovimiento(
  query: ListQuery<TipoMovimientoFiltro>,
): Promise<PagedResult<TipoMovimientoListItem>> {
  return callCommand('tipos_movimiento_list', { query })
}

export function getTipoMovimiento(id: Uuid): Promise<TipoMovimientoDetalle> {
  return callCommand('tipos_movimiento_get', { id })
}

export function createTipoMovimiento(dto: TipoMovimientoInput): Promise<TipoMovimientoDetalle> {
  return callCommand('tipos_movimiento_create', { dto })
}

export function updateTipoMovimiento(
  id: Uuid,
  dto: TipoMovimientoInput,
  rowVersion: RowVersion,
): Promise<TipoMovimientoDetalle> {
  return callCommand('tipos_movimiento_update', { id, dto, rowVersion })
}

export function deleteTipoMovimiento(id: Uuid, rowVersion: RowVersion): Promise<void> {
  return callCommand('tipos_movimiento_delete', { id, rowVersion })
}

export function lookupTiposMovimiento(texto?: string, limite?: number): Promise<LookupItem[]> {
  return callCommand('tipos_movimiento_lookup', { texto, limite })
}
