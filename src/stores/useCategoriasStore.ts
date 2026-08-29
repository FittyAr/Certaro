import { defineStore } from 'pinia'
import { ref } from 'vue'

import {
  CATEGORIAS_SORTABLE,
  createCategoria,
  deleteCategoria,
  getCategoria,
  listCategorias,
  lookupCategorias,
  updateCategoria,
  type CategoriaDetalle,
  type CategoriaFiltro,
  type CategoriaInput,
  type CategoriaListItem,
} from '@/api/categorias'
import type { ListQuery, LookupItem, PagedResult, RowVersion, Uuid } from '@/api/types'

import { useCatalogStore } from './useCatalogStore'

export type {
  CategoriaDetalle,
  CategoriaFiltro,
  CategoriaInput,
  CategoriaListItem,
} from '@/api/categorias'
export { CATEGORIAS_SORTABLE } from '@/api/categorias'

export const useCategoriasStore = defineStore('categorias', () => {
  const current = ref<CategoriaDetalle | null>(null)
  const catalog = useCatalogStore()

  function fetchPaged(query: ListQuery<CategoriaFiltro>): Promise<PagedResult<CategoriaListItem>> {
    return listCategorias(query)
  }

  async function fetchOne(id: Uuid): Promise<CategoriaDetalle> {
    current.value = await getCategoria(id)
    return current.value
  }

  async function create(dto: CategoriaInput): Promise<CategoriaDetalle> {
    const created = await createCategoria(dto)
    catalog.invalidateCategorias()
    return created
  }

  async function update(
    id: Uuid,
    dto: CategoriaInput,
    rowVersion: RowVersion,
  ): Promise<CategoriaDetalle> {
    const updated = await updateCategoria(id, dto, rowVersion)
    catalog.invalidateCategorias()
    return updated
  }

  async function remove(id: Uuid, rowVersion: RowVersion): Promise<void> {
    await deleteCategoria(id, rowVersion)
    catalog.invalidateCategorias()
    if (current.value?.id === id) current.value = null
  }

  function lookup(texto?: string, limite?: number): Promise<LookupItem[]> {
    return lookupCategorias(texto, limite)
  }

  return {
    current,
    sortable: CATEGORIAS_SORTABLE,
    fetchPaged,
    fetchOne,
    create,
    update,
    remove,
    lookup,
  }
})
