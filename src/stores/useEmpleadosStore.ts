import { defineStore } from 'pinia'
import { ref } from 'vue'

import {
  cargosEmpleados,
  createEmpleado,
  deleteEmpleado,
  EMPLEADOS_SORTABLE,
  getEmpleado,
  listEmpleados,
  lookupEmpleados,
  updateEmpleado,
  type EmpleadoDetalle,
  type EmpleadoFiltro,
  type EmpleadoInput,
  type EmpleadoListItem,
} from '@/api/empleados'
import type { ListQuery, LookupItem, PagedResult, RowVersion, Uuid } from '@/api/types'

export type {
  EmpleadoDetalle,
  EmpleadoFiltro,
  EmpleadoInput,
  EmpleadoListItem,
  FrecuenciaPago,
} from '@/api/empleados'
export { EMPLEADOS_SORTABLE } from '@/api/empleados'

export const useEmpleadosStore = defineStore('empleados', () => {
  const current = ref<EmpleadoDetalle | null>(null)
  /** The roles already in use, so the filter offers what exists. Loaded once per session. */
  const cargos = ref<string[]>([])
  const opciones = ref<LookupItem[]>([])

  function fetchPaged(query: ListQuery<EmpleadoFiltro>): Promise<PagedResult<EmpleadoListItem>> {
    return listEmpleados(query)
  }

  async function fetchOne(id: Uuid): Promise<EmpleadoDetalle> {
    current.value = await getEmpleado(id)
    return current.value
  }

  async function fetchCargos(): Promise<string[]> {
    cargos.value = await cargosEmpleados()
    return cargos.value
  }

  async function fetchLookup(soloActivos = true, texto?: string): Promise<LookupItem[]> {
    opciones.value = await lookupEmpleados(soloActivos, texto)
    return opciones.value
  }

  async function create(dto: EmpleadoInput): Promise<EmpleadoDetalle> {
    current.value = await createEmpleado(dto)
    // A new role has to show up in the filter without a reload.
    await fetchCargos()
    return current.value
  }

  async function update(
    id: Uuid,
    dto: EmpleadoInput,
    rowVersion: RowVersion,
  ): Promise<EmpleadoDetalle> {
    const updated = await updateEmpleado(id, dto, rowVersion)
    if (current.value?.id === id) current.value = updated
    await fetchCargos()
    return updated
  }

  async function remove(id: Uuid, rowVersion: RowVersion): Promise<void> {
    await deleteEmpleado(id, rowVersion)
    if (current.value?.id === id) current.value = null
  }

  return {
    current,
    cargos,
    opciones,
    sortable: EMPLEADOS_SORTABLE,
    fetchPaged,
    fetchOne,
    fetchCargos,
    fetchLookup,
    create,
    update,
    remove,
  }
})
