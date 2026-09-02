import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import {
  kanbanApi,
  type ActualizarChecklistInput,
  type ActualizarColumnaInput,
  type ActualizarEtiquetaInput,
  type ActualizarTableroInput,
  type ActualizarTarjetaInput,
  type CrearChecklistInput,
  type CrearColumnaInput,
  type CrearEtiquetaInput,
  type CrearTableroInput,
  type CrearTarjetaInput,
  type KanbanChecklistDto,
  type KanbanColumnaDto,
  type KanbanEtiquetaDto,
  type KanbanTableroDetalleDto,
  type KanbanTableroDto,
  type KanbanTarjetaDto,
  type MoverTarjetaInput,
  type ReordenarColumnasInput,
  type ReordenarTarjetasInput,
  type PrioridadTarjeta,
  type TipoPresetTablero,
} from '@/api/kanban'
import type { RowVersion, Uuid } from '@/api/types'

export type {
  ActualizarChecklistInput,
  ActualizarColumnaInput,
  ActualizarEtiquetaInput,
  ActualizarTableroInput,
  ActualizarTarjetaInput,
  CrearChecklistInput,
  CrearColumnaInput,
  CrearEtiquetaInput,
  CrearTableroInput,
  CrearTarjetaInput,
  KanbanChecklistDto,
  KanbanColumnaDto,
  KanbanEtiquetaDto,
  KanbanTableroDetalleDto,
  KanbanTableroDto,
  KanbanTarjetaDto,
  MoverTarjetaInput,
  ReordenarColumnasInput,
  ReordenarTarjetasInput,
  PrioridadTarjeta,
  TipoPresetTablero,
  RowVersion,
  Uuid,
}

export const useKanbanStore = defineStore('kanban', () => {
  const tableros = ref<KanbanTableroDto[]>([])
  const currentTableroId = ref<Uuid | null>(null)
  const detalle = ref<KanbanTableroDetalleDto | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  const activeTableros = computed(() => tableros.value.filter((t) => t.activo))
  const currentTablero = computed(() =>
    tableros.value.find((t) => t.id === currentTableroId.value) ?? null
  )

  async function fetchTableros() {
    loading.value = true
    error.value = null
    try {
      const list = await kanbanApi.listTableros()
      tableros.value = list
      if (!currentTableroId.value && list.length > 0 && list[0]) {
        currentTableroId.value = list[0].id
        await fetchDetalle(list[0].id)
      }
    } catch (e: any) {
      error.value = e?.message ?? 'Error al cargar tableros'
    } finally {
      loading.value = false
    }
  }

  async function fetchDetalle(id: Uuid) {
    loading.value = true
    error.value = null
    try {
      detalle.value = await kanbanApi.getTablero(id)
    } catch (e: any) {
      error.value = e?.message ?? 'Error al cargar detalle del tablero'
    } finally {
      loading.value = false
    }
  }

  async function selectTablero(id: Uuid) {
    currentTableroId.value = id
    await fetchDetalle(id)
  }

  async function createTablero(input: CrearTableroInput) {
    loading.value = true
    error.value = null
    try {
      const created = await kanbanApi.createTablero(input)
      tableros.value.push(created)
      await selectTablero(created.id)
      return created
    } catch (e: any) {
      error.value = e?.message ?? 'Error al crear tablero'
      throw e
    } finally {
      loading.value = false
    }
  }

  async function updateTablero(id: Uuid, input: ActualizarTableroInput) {
    loading.value = true
    error.value = null
    try {
      const updated = await kanbanApi.updateTablero(id, input)
      const idx = tableros.value.findIndex((t) => t.id === id)
      if (idx !== -1) tableros.value[idx] = updated
      if (detalle.value && detalle.value.tablero.id === id) {
        detalle.value.tablero = updated
      }
      return updated
    } catch (e: any) {
      error.value = e?.message ?? 'Error al actualizar tablero'
      throw e
    } finally {
      loading.value = false
    }
  }

  async function deleteTablero(id: Uuid, rowVersion: RowVersion) {
    loading.value = true
    error.value = null
    try {
      await kanbanApi.deleteTablero(id, rowVersion)
      tableros.value = tableros.value.filter((t) => t.id !== id)
      if (currentTableroId.value === id) {
        currentTableroId.value = tableros.value[0]?.id ?? null
        if (currentTableroId.value) {
          await fetchDetalle(currentTableroId.value)
        } else {
          detalle.value = null
        }
      }
    } catch (e: any) {
      error.value = e?.message ?? 'Error al eliminar tablero'
      throw e
    } finally {
      loading.value = false
    }
  }

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
      // Reload on failure to reconcile state
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

  async function syncPreset(tableroId: Uuid) {
    loading.value = true
    try {
      await kanbanApi.sincronizarPreset(tableroId)
      await fetchDetalle(tableroId)
    } catch (e: any) {
      error.value = e?.message ?? 'Error al sincronizar preset'
      throw e
    } finally {
      loading.value = false
    }
  }

  // --- Etiquetas ---

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

  // --- Checklist ---

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
    tableros,
    currentTableroId,
    detalle,
    loading,
    error,
    activeTableros,
    currentTablero,
    fetchTableros,
    fetchDetalle,
    selectTablero,
    createTablero,
    updateTablero,
    deleteTablero,
    createColumna,
    updateColumna,
    reordenarColumnas,
    deleteColumna,
    createTarjeta,
    updateTarjeta,
    moverTarjeta,
    reordenarTarjetas,
    deleteTarjeta,
    syncPreset,
    createEtiqueta,
    listChecklist,
    addChecklistItem,
    updateChecklistItem,
    deleteChecklistItem,
  }
})
