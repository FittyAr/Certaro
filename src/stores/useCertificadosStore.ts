import { defineStore } from 'pinia'
import { ref } from 'vue'

import {
  CERTIFICADOS_SORTABLE,
  createCertificado,
  deleteCertificado,
  getCertificado,
  listCertificados,
  prepararCertificado,
  updateObservacionesCertificado,
  type CertificadoBorrador,
  type CertificadoDetalle,
  type CertificadoFiltro,
  type CertificadoInput,
  type CertificadoListItem,
} from '@/api/certificados'
import type { ListQuery, PagedResult, RowVersion, Uuid } from '@/api/types'

export type {
  CertificadoBorrador,
  CertificadoBorradorItem,
  CertificadoDetalle,
  CertificadoFiltro,
  CertificadoInput,
  CertificadoItem,
  CertificadoListItem,
} from '@/api/certificados'
export { CERTIFICADOS_SORTABLE } from '@/api/certificados'

export const useCertificadosStore = defineStore('certificados', () => {
  const current = ref<CertificadoDetalle | null>(null)
  /** The prefilled form of the certificate being issued. */
  const borrador = ref<CertificadoBorrador | null>(null)

  function fetchPaged(
    query: ListQuery<CertificadoFiltro>,
  ): Promise<PagedResult<CertificadoListItem>> {
    return listCertificados(query)
  }

  async function fetchOne(id: Uuid): Promise<CertificadoDetalle> {
    current.value = await getCertificado(id)
    return current.value
  }

  async function preparar(ordenTrabajoId: Uuid): Promise<CertificadoBorrador> {
    borrador.value = await prepararCertificado(ordenTrabajoId)
    return borrador.value
  }

  async function create(dto: CertificadoInput): Promise<CertificadoDetalle> {
    current.value = await createCertificado(dto)
    // The draft described the state before issuing, so it no longer describes anything.
    borrador.value = null
    return current.value
  }

  async function updateObservaciones(
    id: Uuid,
    observaciones: string | null,
    rowVersion: RowVersion,
  ): Promise<CertificadoDetalle> {
    const updated = await updateObservacionesCertificado(id, observaciones, rowVersion)
    if (current.value?.id === id) current.value = updated
    return updated
  }

  async function remove(id: Uuid, rowVersion: RowVersion): Promise<void> {
    await deleteCertificado(id, rowVersion)
    if (current.value?.id === id) current.value = null
  }

  return {
    current,
    borrador,
    sortable: CERTIFICADOS_SORTABLE,
    fetchPaged,
    fetchOne,
    preparar,
    create,
    updateObservaciones,
    remove,
  }
})
