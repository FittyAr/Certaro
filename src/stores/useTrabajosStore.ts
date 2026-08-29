import { defineStore } from 'pinia'
import { ref } from 'vue'

import {
  createTrabajo,
  deleteTrabajo,
  getTrabajo,
  listTrabajos,
  lookupTrabajos,
  TRABAJOS_SORTABLE,
  transitionTrabajo,
  updateTrabajo,
  type EstadoTrabajo,
  type TrabajoDetalle,
  type TrabajoFiltro,
  type TrabajoInput,
  type TrabajoListItem,
} from '@/api/trabajos'
import type { ListQuery, LookupItem, PagedResult, RowVersion, Uuid } from '@/api/types'

export type {
  EstadoTrabajo,
  TrabajoDetalle,
  TrabajoFiltro,
  TrabajoInput,
  TrabajoListItem,
} from '@/api/trabajos'
export { TRABAJOS_SORTABLE } from '@/api/trabajos'

export const useTrabajosStore = defineStore('trabajos', () => {
  const current = ref<TrabajoDetalle | null>(null)

  function fetchPaged(query: ListQuery<TrabajoFiltro>): Promise<PagedResult<TrabajoListItem>> {
    return listTrabajos(query)
  }

  async function fetchOne(id: Uuid): Promise<TrabajoDetalle> {
    current.value = await getTrabajo(id)
    return current.value
  }

  function create(dto: TrabajoInput): Promise<TrabajoDetalle> {
    return createTrabajo(dto)
  }

  function update(id: Uuid, dto: TrabajoInput, rowVersion: RowVersion): Promise<TrabajoDetalle> {
    return updateTrabajo(id, dto, rowVersion)
  }

  async function transition(
    id: Uuid,
    destino: EstadoTrabajo,
    rowVersion: RowVersion,
    forzar = false,
  ): Promise<TrabajoDetalle> {
    const updated = await transitionTrabajo(id, destino, rowVersion, forzar)
    if (current.value?.id === id) current.value = updated
    return updated
  }

  async function remove(id: Uuid, rowVersion: RowVersion): Promise<void> {
    await deleteTrabajo(id, rowVersion)
    if (current.value?.id === id) current.value = null
  }

  function lookup(obraId?: Uuid, texto?: string, limite?: number): Promise<LookupItem[]> {
    return lookupTrabajos(obraId, texto, limite)
  }

  return {
    current,
    sortable: TRABAJOS_SORTABLE,
    fetchPaged,
    fetchOne,
    create,
    update,
    transition,
    remove,
    lookup,
  }
})
