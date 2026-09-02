import { callCommand } from './client'
import type { Audit, ListQuery, LookupItem, Money, PagedResult, RowVersion, Uuid } from './types'

/** See `docs/11-contratos-tauri.md` §5.2. */

export interface ClienteFiltro {
  /** Matched against the name, the CUIT and the email, ignoring case. */
  texto?: string
  condicionIva?: string
  soloConDeuda?: boolean
}

export interface ClienteContactoInput {
  /** Absent on a row the user just added; present on one being edited. */
  id?: Uuid
  etiqueta: string
  email: string
  nombre: string | null
  telefono: string | null
  esPrincipal: boolean
}

/**
 * Contacts travel with the customer: they are one aggregate saved in one transaction, so there is
 * no separate contact command that could get out of step with this one.
 */
export interface ClienteInput {
  nombre: string
  cuit: string | null
  direccion: string | null
  telefono: string | null
  email: string | null
  condicionIva: string | null
  contactos: ClienteContactoInput[]
}

export interface ClienteContacto {
  id: Uuid
  etiqueta: string
  email: string
  nombre: string | null
  telefono: string | null
  esPrincipal: boolean
}

export interface ClienteListItem {
  id: Uuid
  nombre: string
  cuit: string | null
  telefono: string | null
  email: string | null
  condicionIva: string | null
  proyectosCount: number
  facturasCount: number
  deuda: Money
  puedeEliminarse: boolean
  rowVersion: RowVersion
}

export interface ClienteDetalle {
  id: Uuid
  nombre: string
  cuit: string | null
  direccion: string | null
  telefono: string | null
  email: string | null
  condicionIva: string | null
  contactos: ClienteContacto[]
  proyectosCount: number
  facturasCount: number
  puedeEliminarse: boolean
  audit: Audit
}

export const CLIENTES_SORTABLE = [
  'nombre',
  'cuit',
  'deuda',
  'proyectosCount',
  'facturasCount',
  'createdAt',
] as const

export function listClientes(
  query: ListQuery<ClienteFiltro>,
): Promise<PagedResult<ClienteListItem>> {
  return callCommand('clientes_list', { query })
}

export function getCliente(id: Uuid): Promise<ClienteDetalle> {
  return callCommand('clientes_get', { id })
}

export function createCliente(dto: ClienteInput): Promise<ClienteDetalle> {
  return callCommand('clientes_create', { dto })
}

export function updateCliente(
  id: Uuid,
  dto: ClienteInput,
  rowVersion: RowVersion,
): Promise<ClienteDetalle> {
  return callCommand('clientes_update', { id, dto, rowVersion })
}

export function deleteCliente(id: Uuid, rowVersion: RowVersion): Promise<void> {
  return callCommand('clientes_delete', { id, rowVersion })
}

export function lookupClientes(texto?: string, limite?: number): Promise<LookupItem[]> {
  return callCommand('clientes_lookup', { texto, limite })
}
