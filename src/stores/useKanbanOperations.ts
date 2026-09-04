import {
  kanbanApi,
  type ActualizarChecklistInput,
  type ActualizarColumnaInput,
  type CrearChecklistInput,
  type CrearColumnaInput,
  type CrearEtiquetaInput,
  type CrearTarjetaInput,
  type ActualizarTarjetaInput,
  type MoverTarjetaInput,
  type ReordenarColumnasInput,
  type ReordenarTarjetasInput,
  type KanbanTableroDetalleDto,
} from '@/api/kanban'
import type { RowVersion, Uuid } from '@/api/types'
import type { Ref } from 'vue'

export function useKanbanOperations(
  detalle: Ref<KanbanTableroDetalleDto | null>,
  currentTableroId: Ref<Uuid | null>,
  error: Ref<string | null>,
  fetchDetalle: (id: Uuid) => Promise<void>,
) {
  async function createColumna(input: CrearColumnaInput) {
    try {
      const created = await kanbanApi.createColumna(input)
      if (detalle.value && detalle.value.tablero.id === input.tableroId) {
        detalle.value.columnas.push(created)
      }
      return created
    } catch (e: any) {
      error.value = e?.message ?? 'Error al crear columna'
      throw e
    }
  }

  async function updateColumna(id: Uuid, input: ActualizarColumnaInput) {
    try {
      const updated = await kanbanApi.updateColumna(id, input)
      if (detalle.value) {
        const idx = detalle.value.columnas.findIndex((c) => c.id === id)
        if (idx !== -1) detalle.value.columnas[idx] = updated
      }
      return updated
    } catch (e: any) {
      error.value = e?.message ?? 'Error al actualizar columna'
      throw e
    }
  }

  async function deleteColumna(id: Uuid, rowVersion: RowVersion) {
    try {
      await kanbanApi.deleteColumna(id, rowVersion)
      if (detalle.value) {
        detalle.value.columnas = detalle.value.columnas.filter((c) => c.id !== id)
        detalle.value.tarjetas = detalle.value.tarjetas.filter((t) => t.columnaId !== id)
      }
    } catch (e: any) {
      error.value = e?.message ?? 'Error al eliminar columna'
      throw e
    }
  }

  async function createTarjeta(input: CrearTarjetaInput) {
    try {
      const created = await kanbanApi.createTarjeta(input)
      if (detalle.value) {
        detalle.value.tarjetas.push(created)
      }
      return created
    } catch (e: any) {
      error.value = e?.message ?? 'Error al crear tarjeta'
      throw e
    }
  }

  async function updateTarjeta(id: Uuid, input: ActualizarTarjetaInput) {
    try {
      const updated = await kanbanApi.updateTarjeta(id, input)
      if (detalle.value) {
        const idx = detalle.value.tarjetas.findIndex((t) => t.id === id)
        if (idx !== -1) detalle.value.tarjetas[idx] = updated
      }
      return updated
    } catch (e: any) {
      error.value = e?.message ?? 'Error al actualizar tarjeta'
      throw e
    }
  }

  async function moverTarjeta(input: MoverTarjetaInput) {
    // Optimistic local update
    if (detalle.value) {
      const target = detalle.value.tarjetas.find((t) => t.id === input.tarjetaId)
      if (target) {
        target.columnaId = input.nuevaColumnaId
        target.orden = input.nuevoOrden
      }
    }
    try {
      await kanbanApi.moverTarjeta(input)
    } catch (e: any) {
      error.value = e?.message ?? 'Error al mover tarjeta'
      if (currentTableroId.value) {
        await fetchDetalle(currentTableroId.value)
      }
      throw e
    }
  }

  async function reordenarColumnas(input: ReordenarColumnasInput) {
    if (detalle.value) {
      input.columnaIds.forEach((id, idx) => {
        const col = detalle.value?.columnas.find((c) => c.id === id)
        if (col) col.orden = idx
      })
    }
    try {
      await kanbanApi.reordenarColumnas(input)
    } catch (e: any) {
      error.value = e?.message ?? 'Error al reordenar columnas'
      if (currentTableroId.value) {
        await fetchDetalle(currentTableroId.value)
      }
      throw e
    }
  }

  async function reordenarTarjetas(input: ReordenarTarjetasInput) {
    if (detalle.value) {
      const card = detalle.value.tarjetas.find((t) => t.id === input.tarjetaId)
      if (card) {
        card.columnaId = input.destinoColumnaId
        card.orden = input.nuevoOrden
      }
      input.tarjetaIdsEnDestino.forEach((id, idx) => {
        const c = detalle.value?.tarjetas.find((t) => t.id === id)
        if (c) c.orden = idx
      })
    }
    try {
      await kanbanApi.reordenarTarjetas(input)
    } catch (e: any) {
      error.value = e?.message ?? 'Error al reordenar tarjetas'
      if (currentTableroId.value) {
        await fetchDetalle(currentTableroId.value)
      }
      throw e
    }
  }

  async function deleteTarjeta(id: Uuid, rowVersion: RowVersion) {
    try {
      await kanbanApi.deleteTarjeta(id, rowVersion)
      if (detalle.value) {
        detalle.value.tarjetas = detalle.value.tarjetas.filter((t) => t.id !== id)
      }
    } catch (e: any) {
      error.value = e?.message ?? 'Error al eliminar tarjeta'
      throw e
    }
  }

  async function createEtiqueta(input: CrearEtiquetaInput) {
    try {
      const created = await kanbanApi.createEtiqueta(input)
      if (detalle.value) {
        detalle.value.etiquetas.push(created)
      }
      return created
    } catch (e: any) {
      error.value = e?.message ?? 'Error al crear etiqueta'
      throw e
    }
  }

  async function listChecklist(tarjetaId: Uuid) {
    return await kanbanApi.listChecklist(tarjetaId)
  }

  async function addChecklistItem(input: CrearChecklistInput) {
    const item = await kanbanApi.addChecklistItem(input)
    if (detalle.value) {
      const tarjeta = detalle.value.tarjetas.find((t) => t.id === input.tarjetaId)
      if (tarjeta) tarjeta.totalChecklist += 1
    }
    return item
  }

  async function updateChecklistItem(id: Uuid, input: ActualizarChecklistInput) {
    const item = await kanbanApi.updateChecklistItem(id, input)
    return item
  }

  async function deleteChecklistItem(id: Uuid, tarjetaId: Uuid, wasCompleted: boolean) {
    await kanbanApi.deleteChecklistItem(id)
    if (detalle.value) {
      const tarjeta = detalle.value.tarjetas.find((t) => t.id === tarjetaId)
      if (tarjeta) {
        tarjeta.totalChecklist = Math.max(0, tarjeta.totalChecklist - 1)
        if (wasCompleted) {
          tarjeta.completadasChecklist = Math.max(0, tarjeta.completadasChecklist - 1)
        }
      }
    }
  }

  return {
    createColumna,
    updateColumna,
    deleteColumna,
    createTarjeta,
    updateTarjeta,
    moverTarjeta,
    reordenarColumnas,
    reordenarTarjetas,
    deleteTarjeta,
    createEtiqueta,
    listChecklist,
    addChecklistItem,
    updateChecklistItem,
    deleteChecklistItem,
  }
}
