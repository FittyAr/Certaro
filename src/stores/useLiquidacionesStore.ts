import { defineStore } from 'pinia'
import { ref } from 'vue'

import {
  createLiquidacion,
  createLiquidacionesBatch,
  deleteLiquidacion,
  getLiquidacion,
  LIQUIDACIONES_SORTABLE,
  listLiquidaciones,
  suggestLiquidaciones,
  updateLiquidacion,
  type LiquidacionDetalle,
  type LiquidacionFiltro,
  type LiquidacionInput,
  type LiquidacionListItem,
  type LiquidacionSugerencia,
  type LiquidacionSugerenciaQuery,
  type LiquidacionUpdateInput,
} from '@/api/liquidaciones'
import type { ListQuery, PagedResult, RowVersion, Uuid } from '@/api/types'

export type {
  LiquidacionAdelantoDto,
  LiquidacionAdelantoSugerido,
  LiquidacionDesglose,
  LiquidacionDetalle,
  LiquidacionFiltro,
  LiquidacionInput,
  LiquidacionListItem,
  LiquidacionSugerencia,
  LiquidacionUpdateInput,
  OrigenLiquidacion,
} from '@/api/liquidaciones'
export { LIQUIDACIONES_SORTABLE } from '@/api/liquidaciones'

export const useLiquidacionesStore = defineStore('liquidaciones', () => {
  const current = ref<LiquidacionDetalle | null>(null)
  /** Step two of the wizard: what the backend proposes, before the user edits it. */
  const sugerencias = ref<LiquidacionSugerencia[]>([])

  function fetchPaged(
    query: ListQuery<LiquidacionFiltro>,
  ): Promise<PagedResult<LiquidacionListItem>> {
    return listLiquidaciones(query)
  }

  async function fetchOne(id: Uuid): Promise<LiquidacionDetalle> {
    current.value = await getLiquidacion(id)
    return current.value
  }

  async function suggest(query: LiquidacionSugerenciaQuery): Promise<LiquidacionSugerencia[]> {
    sugerencias.value = await suggestLiquidaciones(query)
    return sugerencias.value
  }

  async function create(dto: LiquidacionInput): Promise<LiquidacionDetalle> {
    current.value = await createLiquidacion(dto)
    return current.value
  }

  async function createBatch(dtos: LiquidacionInput[]): Promise<Uuid[]> {
    const result = await createLiquidacionesBatch(dtos)
    // The suggestions described a state where nothing was settled yet.
    sugerencias.value = []
    return result.creadas
  }

  async function update(
    id: Uuid,
    dto: LiquidacionUpdateInput,
    rowVersion: RowVersion,
  ): Promise<LiquidacionDetalle> {
    const updated = await updateLiquidacion(id, dto, rowVersion)
    if (current.value?.id === id) current.value = updated
    return updated
  }

  async function remove(id: Uuid, rowVersion: RowVersion): Promise<void> {
    await deleteLiquidacion(id, rowVersion)
    if (current.value?.id === id) current.value = null
  }

  return {
    current,
    sugerencias,
    sortable: LIQUIDACIONES_SORTABLE,
    fetchPaged,
    fetchOne,
    suggest,
    create,
    createBatch,
    update,
    remove,
  }
})
