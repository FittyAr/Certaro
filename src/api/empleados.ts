import { callCommand } from './client'
import type {
  Audit,
  CivilDate,
  Decimal4,
  ListQuery,
  LookupItem,
  Money,
  PagedResult,
  RowVersion,
  Uuid,
} from './types'

/** See `docs/11-contratos-tauri.md` §5.7. */

export type FrecuenciaPago = 'Diario' | 'Semanal' | 'Quincenal' | 'Mensual'

export interface EmpleadoFiltro {
  texto?: string
  /** Absent means everyone; the list screen sends `true`. */
  activo?: boolean
  cargo?: string
}

export interface EmpleadoInput {
  nombre: string
  dni: string | null
  cargo: string | null
  sueldoBase: Money
  pagoFrecuencia: FrecuenciaPago
  tarifaDiaria: Money
  multiplicadorSabado: Decimal4
  multiplicadorDomingo: Decimal4
  multiplicadorFeriado: Decimal4
  email: string | null
  telefono: string | null
  fechaIngreso: CivilDate
  fechaEgreso: CivilDate | null
  activo: boolean
}

export interface EmpleadoListItem {
  id: Uuid
  nombre: string
  dni: string | null
  cargo: string | null
  tarifaDiaria: Money
  sueldoBase: Money
  pagoFrecuencia: FrecuenciaPago
  email: string | null
  telefono: string | null
  fechaIngreso: CivilDate
  fechaEgreso: CivilDate | null
  activo: boolean
  rowVersion: RowVersion
}

export interface EmpleadoDetalle {
  id: Uuid
  nombre: string
  dni: string | null
  cargo: string | null
  sueldoBase: Money
  pagoFrecuencia: FrecuenciaPago
  tarifaDiaria: Money
  /** What the rate would be if derived from the salary, offered by the form. */
  tarifaDiariaSugerida: Money
  multiplicadorSabado: Decimal4
  multiplicadorDomingo: Decimal4
  multiplicadorFeriado: Decimal4
  email: string | null
  telefono: string | null
  fechaIngreso: CivilDate
  fechaEgreso: CivilDate | null
  activo: boolean
  /** False when settlements, attendance or movements depend on the employee. */
  puedeEliminarse: boolean
  audit: Audit
}

export const EMPLEADOS_SORTABLE = [
  'nombre',
  'cargo',
  'tarifaDiaria',
  'sueldoBase',
  'fechaIngreso',
] as const

export function listEmpleados(
  query: ListQuery<EmpleadoFiltro>,
): Promise<PagedResult<EmpleadoListItem>> {
  return callCommand('empleados_list', { query })
}

export function getEmpleado(id: Uuid): Promise<EmpleadoDetalle> {
  return callCommand('empleados_get', { id })
}

export function lookupEmpleados(
  soloActivos?: boolean,
  texto?: string,
  limite?: number,
): Promise<LookupItem[]> {
  return callCommand('empleados_lookup', { soloActivos, texto, limite })
}

/** The roles already in use, so the filter offers what exists instead of a free field. */
export function cargosEmpleados(): Promise<string[]> {
  return callCommand('empleados_cargos', {})
}

export function createEmpleado(dto: EmpleadoInput): Promise<EmpleadoDetalle> {
  return callCommand('empleados_create', { dto })
}

export function updateEmpleado(
  id: Uuid,
  dto: EmpleadoInput,
  rowVersion: RowVersion,
): Promise<EmpleadoDetalle> {
  return callCommand('empleados_update', { id, dto, rowVersion })
}

export function deleteEmpleado(id: Uuid, rowVersion: RowVersion): Promise<void> {
  return callCommand('empleados_delete', { id, rowVersion })
}
