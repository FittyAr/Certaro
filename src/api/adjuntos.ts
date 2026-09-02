import { callCommand } from './client'
import type { Uuid } from './types'

/** See `docs/11-contratos-tauri.md` §5.12 and `docs/13-servicios-externos-y-archivos.md` §1. */

export type EntidadAdjunto =
  | 'Movimiento'
  | 'Factura'
  | 'Certificado'
  | 'Liquidacion'
  | 'Cliente'
  | 'Proyecto'
  | 'Trabajo'

export interface AdjuntoItem {
  id: Uuid
  entidadTipo: EntidadAdjunto
  entidadId: Uuid
  nombreArchivo: string
  mime: string
  /** Bytes. The component turns this into KB or MB. */
  tamano: number
  adjuntadoEn: string
}

export interface AdjuntoInput {
  entidadTipo: EntidadAdjunto
  entidadId: Uuid
  rutaOrigen: string
}

export function listAdjuntos(
  entidadTipo: EntidadAdjunto,
  entidadId: Uuid,
): Promise<AdjuntoItem[]> {
  return callCommand('adjuntos_list', { entidadTipo, entidadId })
}

export function addAdjunto(input: AdjuntoInput): Promise<AdjuntoItem> {
  return callCommand('adjuntos_add', { input })
}

export function deleteAdjunto(id: Uuid): Promise<void> {
  return callCommand('adjuntos_delete', { id })
}

export function openAdjunto(id: Uuid): Promise<void> {
  return callCommand('adjuntos_open', { id })
}

export function revealAdjunto(id: Uuid): Promise<void> {
  return callCommand('adjuntos_reveal', { id })
}
