import { defineStore } from 'pinia'
import { ref } from 'vue'

import {
  createOrdenTrabajo,
  deleteOrdenTrabajo,
  getOrdenTrabajo,
  listOrdenesTrabajo,
  lookupOrdenesTrabajo,
  updateOrdenTrabajo,
  type OrdenTrabajoDetalle,
  type OrdenTrabajoInput,
  type OrdenTrabajoListItem,
} from '@/api/ordenesTrabajo'
import type { LookupItem, RowVersion, Uuid } from '@/api/types'

export type {
  OrdenTrabajoDetalle,
  OrdenTrabajoInput,
  OrdenTrabajoItem,
  OrdenTrabajoItemInput,
  OrdenTrabajoListItem,
} from '@/api/ordenesTrabajo'

export const useOrdenesTrabajoStore = defineStore('ordenesTrabajo', () => {
  const current = ref<OrdenTrabajoDetalle | null>(null)

  function fetchDeTrabajo(trabajoId: Uuid): Promise<OrdenTrabajoListItem[]> {
    return listOrdenesTrabajo(trabajoId)
  }

  function fetchList(trabajoId?: Uuid): Promise<OrdenTrabajoListItem[]> {
    return listOrdenesTrabajo(trabajoId)
  }

  async function fetchOne(id: Uuid): Promise<OrdenTrabajoDetalle> {
    current.value = await getOrdenTrabajo(id)
    return current.value
  }

  async function create(dto: OrdenTrabajoInput): Promise<OrdenTrabajoDetalle> {
    current.value = await createOrdenTrabajo(dto)
    return current.value
  }

  async function update(
    id: Uuid,
    dto: OrdenTrabajoInput,
    rowVersion: RowVersion,
  ): Promise<OrdenTrabajoDetalle> {
    const updated = await updateOrdenTrabajo(id, dto, rowVersion)
    if (current.value?.id === id) current.value = updated
    return updated
  }

  async function remove(id: Uuid, rowVersion: RowVersion): Promise<void> {
    await deleteOrdenTrabajo(id, rowVersion)
    if (current.value?.id === id) current.value = null
  }

  function lookup(trabajoId?: Uuid, texto?: string, limite?: number): Promise<LookupItem[]> {
    return lookupOrdenesTrabajo(trabajoId, texto, limite)
  }

  return { current, fetchDeTrabajo, fetchList, fetchOne, create, update, remove, lookup }
})
