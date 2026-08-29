import { defineStore } from 'pinia'
import { ref } from 'vue'

import {
  createFactura,
  createPago,
  deleteFactura,
  deletePago,
  FACTURAS_SORTABLE,
  getFactura,
  listFacturas,
  listPagos,
  lookupFacturas,
  transitionFactura,
  updateFactura,
  updatePago,
  type EstadoFactura,
  type FacturaDetalle,
  type FacturaFiltro,
  type FacturaInput,
  type FacturaListItem,
  type PagoFacturaInput,
  type PagoFacturaItem,
} from '@/api/facturas'
import type { ListQuery, LookupItem, PagedResult, RowVersion, Uuid } from '@/api/types'

export type {
  EstadoFactura,
  FacturaDetalle,
  FacturaFiltro,
  FacturaInput,
  FacturaListItem,
  PagoFacturaInput,
  PagoFacturaItem,
} from '@/api/facturas'
export { FACTURAS_SORTABLE, MEDIOS_PAGO } from '@/api/facturas'

export const useFacturasStore = defineStore('facturas', () => {
  const current = ref<FacturaDetalle | null>(null)

  /** Every write answers with the whole invoice, so the open detail is refreshed in one place. */
  function adopt(detalle: FacturaDetalle): FacturaDetalle {
    if (current.value?.id === detalle.id) current.value = detalle
    return detalle
  }

  function fetchPaged(query: ListQuery<FacturaFiltro>): Promise<PagedResult<FacturaListItem>> {
    return listFacturas(query)
  }

  async function fetchOne(id: Uuid): Promise<FacturaDetalle> {
    current.value = await getFactura(id)
    return current.value
  }

  function create(dto: FacturaInput): Promise<FacturaDetalle> {
    return createFactura(dto)
  }

  async function update(
    id: Uuid,
    dto: FacturaInput,
    rowVersion: RowVersion,
  ): Promise<FacturaDetalle> {
    return adopt(await updateFactura(id, dto, rowVersion))
  }

  async function transition(
    id: Uuid,
    destino: EstadoFactura,
    rowVersion: RowVersion,
  ): Promise<FacturaDetalle> {
    return adopt(await transitionFactura(id, destino, rowVersion))
  }

  async function remove(id: Uuid, rowVersion: RowVersion): Promise<void> {
    await deleteFactura(id, rowVersion)
    if (current.value?.id === id) current.value = null
  }

  function lookup(
    clienteId?: Uuid,
    soloImpagas = false,
    texto?: string,
    limite?: number,
  ): Promise<LookupItem[]> {
    return lookupFacturas(clienteId, soloImpagas, texto, limite)
  }

  function pagos(facturaId: Uuid): Promise<PagoFacturaItem[]> {
    return listPagos(facturaId)
  }

  async function crearPago(dto: PagoFacturaInput): Promise<FacturaDetalle> {
    return adopt(await createPago(dto))
  }

  async function actualizarPago(
    id: Uuid,
    dto: PagoFacturaInput,
    rowVersion: RowVersion,
  ): Promise<FacturaDetalle> {
    return adopt(await updatePago(id, dto, rowVersion))
  }

  async function borrarPago(id: Uuid, rowVersion: RowVersion): Promise<FacturaDetalle> {
    return adopt(await deletePago(id, rowVersion))
  }

  return {
    current,
    sortable: FACTURAS_SORTABLE,
    fetchPaged,
    fetchOne,
    create,
    update,
    transition,
    remove,
    lookup,
    pagos,
    crearPago,
    actualizarPago,
    borrarPago,
  }
})
