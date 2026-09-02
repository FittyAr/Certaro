import { defineStore } from 'pinia'
import { ref } from 'vue'

import {
  createProyecto,
  deleteProyecto,
  getProyecto,
  listProyectos,
  lookupProyectos,
  PROYECTOS_SORTABLE,
  siguienteNumeroProyecto,
  transitionProyecto,
  updateProyecto,
  type EstadoProyecto,
  type ProyectoDetalle,
  type ProyectoFiltro,
  type ProyectoInput,
  type ProyectoListItem,
} from '@/api/proyectos'
import type { ListQuery, LookupItem, PagedResult, RowVersion, Uuid } from '@/api/types'

export type { EstadoProyecto, ProyectoDetalle, ProyectoFiltro, ProyectoInput, ProyectoListItem } from '@/api/proyectos'
export { PROYECTOS_SORTABLE } from '@/api/proyectos'

export const useProyectosStore = defineStore('proyectos', () => {
  const current = ref<ProyectoDetalle | null>(null)

  function fetchPaged(query: ListQuery<ProyectoFiltro>): Promise<PagedResult<ProyectoListItem>> {
    return listProyectos(query)
  }

  async function fetchOne(id: Uuid): Promise<ProyectoDetalle> {
    current.value = await getProyecto(id)
    return current.value
  }

  function create(dto: ProyectoInput): Promise<ProyectoDetalle> {
    return createProyecto(dto)
  }

  function update(id: Uuid, dto: ProyectoInput, rowVersion: RowVersion): Promise<ProyectoDetalle> {
    return updateProyecto(id, dto, rowVersion)
  }

  async function transition(
    id: Uuid,
    destino: EstadoProyecto,
    rowVersion: RowVersion,
    cascada = false,
  ): Promise<ProyectoDetalle> {
    const updated = await transitionProyecto(id, destino, rowVersion, cascada)
    if (current.value?.id === id) current.value = updated
    return updated
  }

  async function remove(id: Uuid, rowVersion: RowVersion): Promise<void> {
    await deleteProyecto(id, rowVersion)
    if (current.value?.id === id) current.value = null
  }

  function lookup(clienteId?: Uuid, texto?: string, limite?: number): Promise<LookupItem[]> {
    return lookupProyectos(clienteId, texto, limite)
  }

  function siguienteNumero(): Promise<number> {
    return siguienteNumeroProyecto()
  }

  return {
    current,
    sortable: PROYECTOS_SORTABLE,
    fetchPaged,
    fetchOne,
    create,
    update,
    transition,
    remove,
    lookup,
    siguienteNumero,
  }
})
