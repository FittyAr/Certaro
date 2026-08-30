import { callCommand } from './client'
import type { CivilDate, Decimal4, Uuid } from './types'

/** See `docs/11-contratos-tauri.md` §5.7. */

export type TipoJornada = 'Completa' | 'Media' | 'Falta' | 'FaltaJustificada' | 'Feriado'

/**
 * The click cycle of the grid. `null` is a cell with no record, and it has to be reachable:
 * otherwise a cell clicked by mistake could never be cleared.
 */
export const CICLO_JORNADA: (TipoJornada | null)[] = [
  'Completa',
  'Media',
  'Falta',
  'FaltaJustificada',
  'Feriado',
  null,
]

export function siguienteJornada(actual: TipoJornada | null): TipoJornada | null {
  const index = CICLO_JORNADA.indexOf(actual)
  return CICLO_JORNADA[(index + 1) % CICLO_JORNADA.length] ?? null
}

export interface AsistenciaGrillaQuery {
  desde: CivilDate
  hasta: CivilDate
  /** Empty means every active employee. */
  empleadoIds?: Uuid[]
}

export interface AsistenciaUpsertInput {
  empleadoId: Uuid
  fecha: CivilDate
  /** `null` clears the cell. */
  tipoJornada: TipoJornada | null
  trabajoId: Uuid | null
  observaciones: string | null
}

export interface AsistenciaRangoInput {
  empleadoId: Uuid
  desde: CivilDate
  hasta: CivilDate
  tipoJornada: TipoJornada
  /** Skips Saturdays, Sundays and holidays. */
  soloDiasHabiles: boolean
  trabajoId: Uuid | null
}

export interface AsistenciaDia {
  fecha: CivilDate
  /** `1` is Monday, `7` is Sunday. */
  diaSemana: number
  esFinDeSemana: boolean
  esFeriado: boolean
  feriadoNombre: string | null
}

export interface AsistenciaCelda {
  fecha: CivilDate
  tipoJornada: TipoJornada | null
  trabajoId: Uuid | null
  observaciones: string | null
}

export interface AsistenciaResumen {
  completas: number
  medias: number
  faltas: number
  faltasJustificadas: number
  feriados: number
  /** Sum of the day factors. */
  jornadasEquivalentes: Decimal4
}

export interface AsistenciaFila {
  empleadoId: Uuid
  empleadoNombre: string
  /** Always the same length as `dias`, so the grid renders by index. */
  celdas: AsistenciaCelda[]
  resumen: AsistenciaResumen
}

export interface AsistenciaGrilla {
  desde: CivilDate
  hasta: CivilDate
  dias: AsistenciaDia[]
  filas: AsistenciaFila[]
}

export function grillaAsistencia(query: AsistenciaGrillaQuery): Promise<AsistenciaGrilla> {
  return callCommand('asistencia_grilla', { query })
}

/** Idempotent on `(empleadoId, fecha)`: the last click wins, so there is no `rowVersion`. */
export function upsertAsistencia(dto: AsistenciaUpsertInput): Promise<AsistenciaCelda> {
  return callCommand('asistencia_upsert', { dto })
}

export function upsertRangoAsistencia(dto: AsistenciaRangoInput): Promise<AsistenciaCelda[]> {
  return callCommand('asistencia_upsert_rango', { dto })
}

export function deleteAsistencia(empleadoId: Uuid, fecha: CivilDate): Promise<void> {
  return callCommand('asistencia_delete', { empleadoId, fecha })
}
