import { callCommand } from './client'
import type { CivilDate } from './types'

/** See `docs/11-contratos-tauri.md` §5.13. */

export type OrigenFeriado = 'Api' | 'Manual'

export interface Feriado {
  fecha: CivilDate
  nombre: string
  tipo: string | null
  /** A sync never overwrites a `Manual` row: what the user loaded by hand wins. */
  origen: OrigenFeriado
}

export interface FeriadoInput {
  fecha: CivilDate
  nombre: string
}

export interface FeriadoSyncResult {
  agregados: number
  total: number
  /** Years the provider could not be reached for; the calendar stays as it was. */
  aniosConError: number
}

export function listFeriados(anio: number): Promise<Feriado[]> {
  return callCommand('feriados_list', { anio })
}

export function syncFeriados(anios: number[]): Promise<FeriadoSyncResult> {
  return callCommand('feriados_sync', { anios })
}

/** Every write returns the year's calendar, so the caller needs no second request. */
export function addFeriado(dto: FeriadoInput): Promise<Feriado[]> {
  return callCommand('feriados_add', { dto })
}

export function deleteFeriado(fecha: CivilDate): Promise<Feriado[]> {
  return callCommand('feriados_delete', { fecha })
}
