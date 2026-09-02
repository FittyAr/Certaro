import { callCommand } from './client'
import type { RowVersion, Uuid } from './types'

export type PrioridadTarjeta = 'Baja' | 'Normal' | 'Alta' | 'Urgente'
export type TipoPresetTablero = 'trabajos' | 'ordenes'

export interface KanbanTableroDto {
  id: Uuid
  nombre: string
  descripcion: string | null
  color: string | null
  esPreset: boolean
  tipoPreset: TipoPresetTablero | null
  activo: boolean
  rowVersion: RowVersion
}

export interface KanbanColumnaDto {
  id: Uuid
  tableroId: Uuid
  nombre: string
  color: string | null
  orden: number
  limiteWip: number | null
  estadoMapeado: number | null
  rowVersion: RowVersion
}

export interface KanbanEtiquetaDto {
  id: Uuid
  nombre: string
  color: string
  rowVersion: RowVersion
}

export interface KanbanChecklistDto {
  id: Uuid
  tarjetaId: Uuid
  titulo: string
  completada: boolean
  orden: number
  rowVersion: RowVersion
}

export interface KanbanTarjetaDto {
  id: Uuid
  columnaId: Uuid
  titulo: string
  descripcion: string | null
  prioridad: PrioridadTarjeta
  fechaVencimiento: string | null
  orden: number
  trabajoId: Uuid | null
  ordenTrabajoId: Uuid | null
  archivada: boolean
  rowVersion: RowVersion
  etiquetas: KanbanEtiquetaDto[]
  totalChecklist: number
  completadasChecklist: number
}

export interface KanbanTableroDetalleDto {
  tablero: KanbanTableroDto
  columnas: KanbanColumnaDto[]
  tarjetas: KanbanTarjetaDto[]
  etiquetas: KanbanEtiquetaDto[]
}

// --- Inputs ---

export interface CrearTableroInput {
  nombre: string
  descripcion?: string | null
  color?: string | null
}

export interface ActualizarTableroInput {
  nombre: string
  descripcion?: string | null
  color?: string | null
  activo: boolean
  rowVersion: RowVersion
}

export interface CrearColumnaInput {
  tableroId: Uuid
  nombre: string
  color?: string | null
  limiteWip?: number | null
}

export interface ActualizarColumnaInput {
  nombre: string
  color?: string | null
  orden: number
  limiteWip?: number | null
  rowVersion: RowVersion
}

export interface CrearTarjetaInput {
  columnaId: Uuid
  titulo: string
  descripcion?: string | null
  prioridad: PrioridadTarjeta
  fechaVencimiento?: string | null
  trabajoId?: Uuid | null
  ordenTrabajoId?: Uuid | null
  etiquetaIds?: Uuid[] | null
}

export interface ActualizarTarjetaInput {
  titulo: string
  descripcion?: string | null
  prioridad: PrioridadTarjeta
  fechaVencimiento?: string | null
  etiquetaIds?: Uuid[] | null
  rowVersion: RowVersion
}

export interface MoverTarjetaInput {
  tarjetaId: Uuid
  nuevaColumnaId: Uuid
  nuevoOrden: number
  rowVersion: RowVersion
}

export interface CrearEtiquetaInput {
  nombre: string
  color: string
}

export interface ActualizarEtiquetaInput {
  nombre: string
  color: string
  rowVersion: RowVersion
}

export interface CrearChecklistInput {
  tarjetaId: Uuid
  titulo: string
}

export interface ActualizarChecklistInput {
  titulo: string
  completada: boolean
  orden: number
  rowVersion: RowVersion
}

export const kanbanApi = {
  listTableros: () => callCommand<KanbanTableroDto[]>('kanban_list_tableros'),
  getTablero: (id: Uuid) => callCommand<KanbanTableroDetalleDto>('kanban_get_tablero', { id }),
  createTablero: (input: CrearTableroInput) =>
    callCommand<KanbanTableroDto>('kanban_create_tablero', { input }),
  updateTablero: (id: Uuid, input: ActualizarTableroInput) =>
    callCommand<KanbanTableroDto>('kanban_update_tablero', { id, input }),
  deleteTablero: (id: Uuid, rowVersion: RowVersion) =>
    callCommand<void>('kanban_delete_tablero', { id, rowVersion }),

  createColumna: (input: CrearColumnaInput) =>
    callCommand<KanbanColumnaDto>('kanban_create_columna', { input }),
  updateColumna: (id: Uuid, input: ActualizarColumnaInput) =>
    callCommand<KanbanColumnaDto>('kanban_update_columna', { id, input }),
  deleteColumna: (id: Uuid, rowVersion: RowVersion) =>
    callCommand<void>('kanban_delete_columna', { id, rowVersion }),

  createTarjeta: (input: CrearTarjetaInput) =>
    callCommand<KanbanTarjetaDto>('kanban_create_tarjeta', { input }),
  updateTarjeta: (id: Uuid, input: ActualizarTarjetaInput) =>
    callCommand<KanbanTarjetaDto>('kanban_update_tarjeta', { id, input }),
  moverTarjeta: (input: MoverTarjetaInput) =>
    callCommand<void>('kanban_mover_tarjeta', { input }),
  deleteTarjeta: (id: Uuid, rowVersion: RowVersion) =>
    callCommand<void>('kanban_delete_tarjeta', { id, rowVersion }),

  sincronizarPreset: (tableroId: Uuid) =>
    callCommand<void>('kanban_sincronizar_preset', { tableroId }),

  listEtiquetas: () => callCommand<KanbanEtiquetaDto[]>('kanban_list_etiquetas'),
  createEtiqueta: (input: CrearEtiquetaInput) =>
    callCommand<KanbanEtiquetaDto>('kanban_create_etiqueta', { input }),
  updateEtiqueta: (id: Uuid, input: ActualizarEtiquetaInput) =>
    callCommand<KanbanEtiquetaDto>('kanban_update_etiqueta', { id, input }),
  deleteEtiqueta: (id: Uuid, rowVersion: RowVersion) =>
    callCommand<void>('kanban_delete_etiqueta', { id, rowVersion }),

  listChecklist: (tarjetaId: Uuid) =>
    callCommand<KanbanChecklistDto[]>('kanban_list_checklist', { tarjetaId }),
  addChecklistItem: (input: CrearChecklistInput) =>
    callCommand<KanbanChecklistDto>('kanban_add_checklist_item', { input }),
  updateChecklistItem: (id: Uuid, input: ActualizarChecklistInput) =>
    callCommand<KanbanChecklistDto>('kanban_update_checklist_item', { id, input }),
  deleteChecklistItem: (id: Uuid) =>
    callCommand<void>('kanban_delete_checklist_item', { id }),
}
