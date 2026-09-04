import { defineStore } from 'pinia'
import { ref } from 'vue'

import {
  deleteAsistencia,
  grillaAsistencia,
  siguienteJornada,
  upsertAsistencia,
  upsertRangoAsistencia,
  type AsistenciaGrilla,
  type AsistenciaGrillaQuery,
  type AsistenciaRangoInput,
  type TipoJornada,
} from '@/api/asistencia'
import type { CivilDate, Uuid } from '@/api/types'

export type {
  AsistenciaCelda,
  AsistenciaDia,
  AsistenciaFila,
  AsistenciaGrilla,
  AsistenciaResumen,
  TipoJornada,
} from '@/api/asistencia'
export { CICLO_JORNADA, siguienteJornada } from '@/api/asistencia'

export const useAsistenciaStore = defineStore('asistencia', () => {
  const grilla = ref<AsistenciaGrilla | null>(null)

  async function fetchGrilla(query: AsistenciaGrillaQuery): Promise<AsistenciaGrilla> {
    grilla.value = await grillaAsistencia(query)
    return grilla.value
  }

  /**
   * One step of the click cycle. The cell is replaced in place with what the backend confirmed
   * rather than with the guess, so a rejected write cannot leave the grid lying.
   */
  async function ciclar(empleadoId: Uuid, fecha: CivilDate): Promise<void> {
    const fila = grilla.value?.filas.find((f) => f.empleadoId === empleadoId)
    const indice = grilla.value?.dias.findIndex((d) => d.fecha === fecha) ?? -1
    if (!fila || indice < 0) return

    const actual = fila.celdas[indice]?.tipoJornada ?? null
    const celda = await upsertAsistencia({
      empleadoId,
      fecha,
      tipoJornada: siguienteJornada(actual),
      trabajoId: fila.celdas[indice]?.trabajoId ?? null,
      observaciones: fila.celdas[indice]?.observaciones ?? null,
    })
    fila.celdas[indice] = celda
  }

  async function marcar(
    empleadoId: Uuid,
    fecha: CivilDate,
    tipoJornada: TipoJornada | null,
    observaciones: string | null = null,
  ): Promise<void> {
    const fila = grilla.value?.filas.find((f) => f.empleadoId === empleadoId)
    const indice = grilla.value?.dias.findIndex((d) => d.fecha === fecha) ?? -1
    const celda = await upsertAsistencia({
      empleadoId,
      fecha,
      tipoJornada,
      trabajoId: null,
      observaciones,
    })
    if (fila && indice >= 0) fila.celdas[indice] = celda
  }

  async function cargarRango(dto: AsistenciaRangoInput): Promise<void> {
    await upsertRangoAsistencia(dto)
    // The summaries per row are computed by the backend, so the grid is reloaded rather than
    // patched cell by cell.
    if (grilla.value) {
      await fetchGrilla({ desde: grilla.value.desde, hasta: grilla.value.hasta })
    }
  }

  async function limpiar(empleadoId: Uuid, fecha: CivilDate): Promise<void> {
    await deleteAsistencia(empleadoId, fecha)
    const fila = grilla.value?.filas.find((f) => f.empleadoId === empleadoId)
    const indice = grilla.value?.dias.findIndex((d) => d.fecha === fecha) ?? -1
    if (fila && indice >= 0) {
      fila.celdas[indice] = { fecha, tipoJornada: null, trabajoId: null, observaciones: null }
    }
  }

  return { grilla, fetchGrilla, ciclar, marcar, cargarRango, limpiar }
})
