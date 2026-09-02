import { callCommand } from './client'
import type { RowVersion, Uuid } from './types'

export type TipoRecurso = 'Empleado' | 'Vehiculo' | 'Herramienta' | 'Proyecto'
export type TipoEvento = 'Trabajo' | 'Reunion' | 'Mantenimiento' | 'Entrega' | 'Otro'

export interface CalendarioGrupoRecursoDto {
  id: Uuid
  nombre: string
  color: string | null
  rowVersion: RowVersion
}

export interface CalendarioRecursoDto {
  id: Uuid
  grupoId: Uuid | null
  grupoNombre: string | null
  nombre: string
  tipo: TipoRecurso
  empleadoId: Uuid | null
  color: string | null
  activo: boolean
  rowVersion: RowVersion
}

export interface CalendarioEventoDto {
  id: Uuid
  titulo: string
  descripcion: string | null
  tipo: TipoEvento
  inicio: string
  fin: string
  todoElDia: boolean
  color: string | null
  trabajoId: Uuid | null
  kanbanTarjetaId: Uuid | null
  recursos: CalendarioRecursoDto[]
  esVirtual: boolean
  rowVersion: RowVersion
}

// --- Inputs ---

export interface CrearGrupoRecursoInput {
  nombre: string
  color?: string | null
}

export interface ActualizarGrupoRecursoInput {
  nombre: string
  color?: string | null
  rowVersion: RowVersion
}

export interface CrearRecursoInput {
  grupoId?: Uuid | null
  nombre: string
  tipo: TipoRecurso
  empleadoId?: Uuid | null
  color?: string | null
}

export interface ActualizarRecursoInput {
  grupoId?: Uuid | null
  nombre: string
  tipo: TipoRecurso
  empleadoId?: Uuid | null
  color?: string | null
  activo: boolean
  rowVersion: RowVersion
}

export interface CrearEventoInput {
  titulo: string
  descripcion?: string | null
  tipo: TipoEvento
  inicio: string
  fin: string
  todoElDia: boolean
  color?: string | null
  trabajoId?: Uuid | null
  kanbanTarjetaId?: Uuid | null
  recursoIds?: Uuid[] | null
}

export interface ActualizarEventoInput {
  titulo: string
  descripcion?: string | null
  tipo: TipoEvento
  inicio: string
  fin: string
  todoElDia: boolean
  color?: string | null
  recursoIds?: Uuid[] | null
  rowVersion: RowVersion
}

export const calendarioApi = {
  listGrupos: () => callCommand<CalendarioGrupoRecursoDto[]>('calendario_list_grupos'),
  createGrupo: (input: CrearGrupoRecursoInput) =>
    callCommand<CalendarioGrupoRecursoDto>('calendario_create_grupo', { input }),
  updateGrupo: (id: Uuid, input: ActualizarGrupoRecursoInput) =>
    callCommand<CalendarioGrupoRecursoDto>('calendario_update_grupo', { id, input }),
  deleteGrupo: (id: Uuid, rowVersion: RowVersion) =>
    callCommand<void>('calendario_delete_grupo', { id, rowVersion }),

  listRecursos: () => callCommand<CalendarioRecursoDto[]>('calendario_list_recursos'),
  createRecurso: (input: CrearRecursoInput) =>
    callCommand<CalendarioRecursoDto>('calendario_create_recurso', { input }),
  updateRecurso: (id: Uuid, input: ActualizarRecursoInput) =>
    callCommand<CalendarioRecursoDto>('calendario_update_recurso', { id, input }),
  deleteRecurso: (id: Uuid, rowVersion: RowVersion) =>
    callCommand<void>('calendario_delete_recurso', { id, rowVersion }),
  sincronizarEmpleados: () =>
    callCommand<void>('calendario_sincronizar_empleados'),

  listEventos: (desde: string, hasta: string) =>
    callCommand<CalendarioEventoDto[]>('calendario_list_eventos', { desde, hasta }),
  createEvento: (input: CrearEventoInput) =>
    callCommand<CalendarioEventoDto>('calendario_create_evento', { input }),
  updateEvento: (id: Uuid, input: ActualizarEventoInput) =>
    callCommand<CalendarioEventoDto>('calendario_update_evento', { id, input }),
  moverEvento: (id: Uuid, nuevoInicio: string, nuevoFin: string, rowVersion: RowVersion) =>
    callCommand<void>('calendario_mover_evento', { id, nuevoInicio, nuevoFin, rowVersion }),
  deleteEvento: (id: Uuid, rowVersion: RowVersion) =>
    callCommand<void>('calendario_delete_evento', { id, rowVersion }),
}
