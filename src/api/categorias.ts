import { callCommand } from './client'
import type { Audit, ListQuery, LookupItem, PagedResult, RowVersion, Uuid } from './types'

/** See `docs/11-contratos-tauri.md` §5.12. */

export interface CategoriaFiltro {
  /** Matched against the name and the description, ignoring case. */
  texto?: string
  categoriaPadreId?: Uuid
  /** Root categories only. Distinct from not filtering by parent at all. */
  soloRaiz?: boolean
}

export interface CategoriaInput {
  nombre: string
  descripcion: string | null
  /** `#RRGGBB`; the backend stores it upper case. */
  colorHex: string | null
  icono: string | null
  categoriaPadreId: Uuid | null
}

export interface CategoriaListItem {
  id: Uuid
  nombre: string
  descripcion: string | null
  colorHex: string | null
  icono: string | null
  categoriaPadreId: Uuid | null
  categoriaPadreNombre: string | null
  movimientosCount: number
  hijasCount: number
  /** False when the category has movements or children, so the action can be disabled up front. */
  puedeEliminarse: boolean
  rowVersion: RowVersion
}

export interface CategoriaDetalle {
  id: Uuid
  nombre: string
  descripcion: string | null
  colorHex: string | null
  icono: string | null
  categoriaPadreId: Uuid | null
  movimientosCount: number
  hijasCount: number
  puedeEliminarse: boolean
  audit: Audit
}

/** Columns the backend accepts; anything else is rejected as a validation error. */
export const CATEGORIAS_SORTABLE = [
  'nombre',
  'movimientosCount',
  'hijasCount',
  'createdAt',
] as const

export function listCategorias(
  query: ListQuery<CategoriaFiltro>,
): Promise<PagedResult<CategoriaListItem>> {
  return callCommand('categorias_list', { query })
}

export function getCategoria(id: Uuid): Promise<CategoriaDetalle> {
  return callCommand('categorias_get', { id })
}

export function createCategoria(dto: CategoriaInput): Promise<CategoriaDetalle> {
  return callCommand('categorias_create', { dto })
}

export function updateCategoria(
  id: Uuid,
  dto: CategoriaInput,
  rowVersion: RowVersion,
): Promise<CategoriaDetalle> {
  return callCommand('categorias_update', { id, dto, rowVersion })
}

export function deleteCategoria(id: Uuid, rowVersion: RowVersion): Promise<void> {
  return callCommand('categorias_delete', { id, rowVersion })
}

export function lookupCategorias(texto?: string, limite?: number): Promise<LookupItem[]> {
  return callCommand('categorias_lookup', { texto, limite })
}
