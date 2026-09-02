import { defineStore } from 'pinia'
import { ref } from 'vue'
import {
  calendarioApi,
  type CalendarioEventoDto,
  type CalendarioGrupoRecursoDto,
  type CalendarioRecursoDto,
  type CrearEventoInput,
  type ActualizarEventoInput,
  type CrearRecursoInput,
  type ActualizarRecursoInput,
  type CrearGrupoRecursoInput,
  type ActualizarGrupoRecursoInput,
  type TipoEvento,
  type TipoRecurso,
} from '@/api/calendario'
import type { RowVersion, Uuid } from '@/api/types'

export type {
  CalendarioEventoDto,
  CalendarioGrupoRecursoDto,
  CalendarioRecursoDto,
  CrearEventoInput,
  ActualizarEventoInput,
  CrearRecursoInput,
  ActualizarRecursoInput,
  CrearGrupoRecursoInput,
  ActualizarGrupoRecursoInput,
  TipoEvento,
  TipoRecurso,
}

export type VistaCalendario = 'mes' | 'semana' | 'dia' | 'recursos'

export const useCalendarioStore = defineStore('calendario', () => {
  const eventos = ref<CalendarioEventoDto[]>([])
  const recursos = ref<CalendarioRecursoDto[]>([])
  const grupos = ref<CalendarioGrupoRecursoDto[]>([])
  const vistaActual = ref<VistaCalendario>('mes')
  const fechaSeleccionada = ref<Date>(new Date())
  const cargando = ref<boolean>(false)
  const error = ref<string | null>(null)

  // Filters
  const filtroTipoRecurso = ref<TipoRecurso | 'Todos'>('Todos')
  const filtroRecursoId = ref<Uuid | 'Todos'>('Todos')
  const mostrarVirtuales = ref<boolean>(true)

  async function cargarGrupos() {
    try {
      grupos.value = await calendarioApi.listGrupos()
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : 'Error al cargar grupos de recursos'
    }
  }

  async function cargarRecursos() {
    try {
      recursos.value = await calendarioApi.listRecursos()
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : 'Error al cargar recursos'
    }
  }

  async function cargarEventos(desdeIso: string, hastaIso: string) {
    cargando.value = true
    error.value = null
    try {
      eventos.value = await calendarioApi.listEventos(desdeIso, hastaIso)
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : 'Error al cargar eventos'
    } finally {
      cargando.value = false
    }
  }

  async function crearEvento(input: CrearEventoInput): Promise<CalendarioEventoDto> {
    const nuevo = await calendarioApi.createEvento(input)
    eventos.value.push(nuevo)
    return nuevo
  }

  async function actualizarEvento(id: Uuid, input: ActualizarEventoInput): Promise<CalendarioEventoDto> {
    const actualizado = await calendarioApi.updateEvento(id, input)
    const idx = eventos.value.findIndex((e) => e.id === id)
    if (idx !== -1) {
      eventos.value[idx] = actualizado
    }
    return actualizado
  }

  async function moverEvento(
    id: Uuid,
    nuevoInicio: string,
    nuevoFin: string,
    rowVersion: RowVersion,
  ): Promise<void> {
    await calendarioApi.moverEvento(id, nuevoInicio, nuevoFin, rowVersion)
    const ev = eventos.value.find((e) => e.id === id)
    if (ev) {
      ev.inicio = nuevoInicio
      ev.fin = nuevoFin
    }
  }

  async function eliminarEvento(id: Uuid, rowVersion: RowVersion): Promise<void> {
    await calendarioApi.deleteEvento(id, rowVersion)
    eventos.value = eventos.value.filter((e) => e.id !== id)
  }

  async function crearRecurso(input: CrearRecursoInput): Promise<CalendarioRecursoDto> {
    const nuevo = await calendarioApi.createRecurso(input)
    recursos.value.push(nuevo)
    return nuevo
  }

  async function actualizarRecurso(id: Uuid, input: ActualizarRecursoInput): Promise<CalendarioRecursoDto> {
    const actualizado = await calendarioApi.updateRecurso(id, input)
    const idx = recursos.value.findIndex((r) => r.id === id)
    if (idx !== -1) {
      recursos.value[idx] = actualizado
    }
    return actualizado
  }

  async function eliminarRecurso(id: Uuid, rowVersion: RowVersion): Promise<void> {
    await calendarioApi.deleteRecurso(id, rowVersion)
    recursos.value = recursos.value.filter((r) => r.id !== id)
  }

  async function sincronizarEmpleados(): Promise<void> {
    await calendarioApi.sincronizarEmpleados()
    await cargarRecursos()
  }

  return {
    eventos,
    recursos,
    grupos,
    vistaActual,
    fechaSeleccionada,
    cargando,
    error,
    filtroTipoRecurso,
    filtroRecursoId,
    mostrarVirtuales,
    cargarGrupos,
    cargarRecursos,
    cargarEventos,
    crearEvento,
    actualizarEvento,
    moverEvento,
    eliminarEvento,
    crearRecurso,
    actualizarRecurso,
    eliminarRecurso,
    sincronizarEmpleados,
  }
})
