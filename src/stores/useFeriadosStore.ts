import { defineStore } from 'pinia'
import { ref } from 'vue'

import {
  addFeriado,
  deleteFeriado,
  listFeriados,
  syncFeriados,
  type Feriado,
  type FeriadoInput,
  type FeriadoSyncResult,
} from '@/api/feriados'
import type { CivilDate } from '@/api/types'

export type { Feriado, FeriadoInput, FeriadoSyncResult, OrigenFeriado } from '@/api/feriados'

export const useFeriadosStore = defineStore('feriados', () => {
  const anio = ref(new Date().getFullYear())
  const feriados = ref<Feriado[]>([])

  async function fetch(year = anio.value): Promise<Feriado[]> {
    anio.value = year
    feriados.value = await listFeriados(year)
    return feriados.value
  }

  /** Defaults to the current year and the next one, which is what the settings screen offers. */
  async function sync(anios = [anio.value, anio.value + 1]): Promise<FeriadoSyncResult> {
    const result = await syncFeriados(anios)
    await fetch()
    return result
  }

  async function add(dto: FeriadoInput): Promise<Feriado[]> {
    feriados.value = await addFeriado(dto)
    return feriados.value
  }

  async function remove(fecha: CivilDate): Promise<Feriado[]> {
    feriados.value = await deleteFeriado(fecha)
    return feriados.value
  }

  return { anio, feriados, fetch, sync, add, remove }
})
