import { defineStore } from 'pinia'
import { ref } from 'vue'

import {
  CLIENTES_SORTABLE,
  createCliente,
  deleteCliente,
  getCliente,
  listClientes,
  lookupClientes,
  updateCliente,
  type ClienteDetalle,
  type ClienteFiltro,
  type ClienteInput,
  type ClienteListItem,
} from '@/api/clientes'
import type { ListQuery, LookupItem, PagedResult, RowVersion, Uuid } from '@/api/types'

export type {
  ClienteContacto,
  ClienteContactoInput,
  ClienteDetalle,
  ClienteFiltro,
  ClienteInput,
  ClienteListItem,
} from '@/api/clientes'
export { CLIENTES_SORTABLE } from '@/api/clientes'

export const useClientesStore = defineStore('clientes', () => {
  const current = ref<ClienteDetalle | null>(null)

  function fetchPaged(query: ListQuery<ClienteFiltro>): Promise<PagedResult<ClienteListItem>> {
    return listClientes(query)
  }

  async function fetchOne(id: Uuid): Promise<ClienteDetalle> {
    current.value = await getCliente(id)
    return current.value
  }

  function create(dto: ClienteInput): Promise<ClienteDetalle> {
    return createCliente(dto)
  }

  function update(id: Uuid, dto: ClienteInput, rowVersion: RowVersion): Promise<ClienteDetalle> {
    return updateCliente(id, dto, rowVersion)
  }

  async function remove(id: Uuid, rowVersion: RowVersion): Promise<void> {
    await deleteCliente(id, rowVersion)
    if (current.value?.id === id) current.value = null
  }

  function lookup(texto?: string, limite?: number): Promise<LookupItem[]> {
    return lookupClientes(texto, limite)
  }

  return {
    current,
    sortable: CLIENTES_SORTABLE,
    fetchPaged,
    fetchOne,
    create,
    update,
    remove,
    lookup,
  }
})
