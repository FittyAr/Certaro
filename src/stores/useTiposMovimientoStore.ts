import { defineStore } from 'pinia'
import { ref } from 'vue'

import {
  createTipoMovimiento,
  deleteTipoMovimiento,
  getTipoMovimiento,
  listTiposMovimiento,
  lookupTiposMovimiento,
  TIPOS_MOVIMIENTO_SORTABLE,
  updateTipoMovimiento,
  type TipoMovimientoDetalle,
  type TipoMovimientoFiltro,
  type TipoMovimientoInput,
  type TipoMovimientoListItem,
} from '@/api/tiposMovimiento'
import type { ListQuery, LookupItem, PagedResult, RowVersion, Uuid } from '@/api/types'

import { useCatalogStore } from './useCatalogStore'

export type {
  TipoMovimientoDetalle,
  TipoMovimientoFiltro,
  TipoMovimientoInput,
  TipoMovimientoListItem,
} from '@/api/tiposMovimiento'
export { TIPOS_MOVIMIENTO_SORTABLE } from '@/api/tiposMovimiento'

export const useTiposMovimientoStore = defineStore('tiposMovimiento', () => {
  const current = ref<TipoMovimientoDetalle | null>(null)
  const catalog = useCatalogStore()

  function fetchPaged(
    query: ListQuery<TipoMovimientoFiltro>,
  ): Promise<PagedResult<TipoMovimientoListItem>> {
    return listTiposMovimiento(query)
  }

  async function fetchOne(id: Uuid): Promise<TipoMovimientoDetalle> {
    current.value = await getTipoMovimiento(id)
    return current.value
  }

  async function create(dto: TipoMovimientoInput): Promise<TipoMovimientoDetalle> {
    const created = await createTipoMovimiento(dto)
    catalog.invalidateTiposMovimiento()
    return created
  }

  async function update(
    id: Uuid,
    dto: TipoMovimientoInput,
    rowVersion: RowVersion,
  ): Promise<TipoMovimientoDetalle> {
    const updated = await updateTipoMovimiento(id, dto, rowVersion)
    catalog.invalidateTiposMovimiento()
    return updated
  }

  async function remove(id: Uuid, rowVersion: RowVersion): Promise<void> {
    await deleteTipoMovimiento(id, rowVersion)
    catalog.invalidateTiposMovimiento()
    if (current.value?.id === id) current.value = null
  }

  function lookup(texto?: string, limite?: number): Promise<LookupItem[]> {
    return lookupTiposMovimiento(texto, limite)
  }

  return {
    current,
    sortable: TIPOS_MOVIMIENTO_SORTABLE,
    fetchPaged,
    fetchOne,
    create,
    update,
    remove,
    lookup,
  }
})
