import { defineStore } from 'pinia'
import { ref } from 'vue'

import {
  createObra,
  deleteObra,
  getObra,
  listObras,
  lookupObras,
  OBRAS_SORTABLE,
  siguienteNumeroObra,
  transitionObra,
  updateObra,
  type EstadoObra,
  type ObraDetalle,
  type ObraFiltro,
  type ObraInput,
  type ObraListItem,
} from '@/api/obras'
import type { ListQuery, LookupItem, PagedResult, RowVersion, Uuid } from '@/api/types'

export type { EstadoObra, ObraDetalle, ObraFiltro, ObraInput, ObraListItem } from '@/api/obras'
export { OBRAS_SORTABLE } from '@/api/obras'

export const useObrasStore = defineStore('obras', () => {
  const current = ref<ObraDetalle | null>(null)

  function fetchPaged(query: ListQuery<ObraFiltro>): Promise<PagedResult<ObraListItem>> {
    return listObras(query)
  }

  async function fetchOne(id: Uuid): Promise<ObraDetalle> {
    current.value = await getObra(id)
    return current.value
  }

  function create(dto: ObraInput): Promise<ObraDetalle> {
    return createObra(dto)
  }

  function update(id: Uuid, dto: ObraInput, rowVersion: RowVersion): Promise<ObraDetalle> {
    return updateObra(id, dto, rowVersion)
  }

  async function transition(
    id: Uuid,
    destino: EstadoObra,
    rowVersion: RowVersion,
    cascada = false,
  ): Promise<ObraDetalle> {
    const updated = await transitionObra(id, destino, rowVersion, cascada)
    if (current.value?.id === id) current.value = updated
    return updated
  }

  async function remove(id: Uuid, rowVersion: RowVersion): Promise<void> {
    await deleteObra(id, rowVersion)
    if (current.value?.id === id) current.value = null
  }

  function lookup(clienteId?: Uuid, texto?: string, limite?: number): Promise<LookupItem[]> {
    return lookupObras(clienteId, texto, limite)
  }

  function siguienteNumero(): Promise<number> {
    return siguienteNumeroObra()
  }

  return {
    current,
    sortable: OBRAS_SORTABLE,
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
