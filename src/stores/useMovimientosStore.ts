import { defineStore } from 'pinia'
import { ref } from 'vue'

import {
  createMovimiento,
  deleteMovimiento,
  getMovimiento,
  listMovimientos,
  MOVIMIENTOS_SORTABLE,
  resumenMovimientos,
  updateMovimiento,
  type MovimientoDetalle,
  type MovimientoFiltro,
  type MovimientoInput,
  type MovimientoListResult,
  type MovimientoResumen,
} from '@/api/movimientos'
import type { ListQuery, RowVersion, Uuid } from '@/api/types'

export type {
  Moneda,
  MovimientoDetalle,
  MovimientoFiltro,
  MovimientoInput,
  MovimientoListItem,
  MovimientoListResult,
  MovimientoResumen,
} from '@/api/movimientos'
export { MOVIMIENTOS_SORTABLE } from '@/api/movimientos'

export const useMovimientosStore = defineStore('movimientos', () => {
  const current = ref<MovimientoDetalle | null>(null)
  /**
   * The totals of the last listed filter. They belong to the store and not to the table because
   * they describe the whole filter, not the visible page.
   */
  const resumen = ref<MovimientoResumen | null>(null)

  async function fetchPaged(query: ListQuery<MovimientoFiltro>): Promise<MovimientoListResult> {
    const result = await listMovimientos(query)
    resumen.value = result.resumen
    return result
  }

  async function fetchOne(id: Uuid): Promise<MovimientoDetalle> {
    current.value = await getMovimiento(id)
    return current.value
  }

  async function fetchResumen(filtro: MovimientoFiltro): Promise<MovimientoResumen> {
    resumen.value = await resumenMovimientos(filtro)
    return resumen.value
  }

  function create(dto: MovimientoInput): Promise<MovimientoDetalle> {
    return createMovimiento(dto)
  }

  function update(
    id: Uuid,
    dto: MovimientoInput,
    rowVersion: RowVersion,
  ): Promise<MovimientoDetalle> {
    return updateMovimiento(id, dto, rowVersion)
  }

  async function remove(id: Uuid, rowVersion: RowVersion): Promise<void> {
    await deleteMovimiento(id, rowVersion)
    if (current.value?.id === id) current.value = null
  }

  return {
    current,
    resumen,
    sortable: MOVIMIENTOS_SORTABLE,
    fetchPaged,
    fetchOne,
    fetchResumen,
    create,
    update,
    remove,
  }
})
