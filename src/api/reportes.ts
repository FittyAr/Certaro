import { callCommand } from './client'
import type { MovimientoFiltro } from './movimientos'
import type { Uuid } from './types'

/** See `docs/11-contratos-tauri.md` §5.11 and `docs/12-reportes-y-exportaciones.md`. */

export type FormatoExport = 'Pdf' | 'Xlsx' | 'Docx' | 'Csv' | 'Json'

/** The formats each report offers, in the order the menu lists them. */
export const FORMATOS_MOVIMIENTOS: readonly FormatoExport[] = [
  'Pdf',
  'Xlsx',
  'Docx',
  'Csv',
  'Json',
] as const

/** Extension per format, for the file dialog filter. */
export const EXTENSIONES: Record<FormatoExport, string> = {
  Pdf: 'pdf',
  Xlsx: 'xlsx',
  Docx: 'docx',
  Csv: 'csv',
  Json: 'json',
}

export interface ExportResult {
  ruta: string
  bytes: number
  /** Rows the document covers. Zero is a valid export: the filter simply matched nothing. */
  registros: number
}

/**
 * Exports the movements of `filtro` — all of them, not the page on screen.
 *
 * `destino` is the absolute path the user picked in the system dialog. The backend validates it
 * before writing.
 */
export function movimientosExport(
  filtro: MovimientoFiltro,
  formato: FormatoExport,
  destino: string,
): Promise<ExportResult> {
  return callCommand('movimientos_export', { filtro, formato, destino })
}

export function liquidacionExport(id: Uuid, destino: string): Promise<ExportResult> {
  return callCommand('liquidacion_export', { id, destino })
}

export function certificadoExport(id: Uuid, destino: string): Promise<ExportResult> {
  return callCommand('certificado_export', { id, destino })
}

/**
 * The name to prefill the dialog with. Asked for before generating, so the user names the file
 * before waiting for it.
 */
export function nombreSugerido(
  reporte: string,
  formato: FormatoExport,
  detalle?: string,
): Promise<string> {
  return callCommand('reportes_nombre_sugerido', {
    reporte,
    formato,
    detalle: detalle ?? null,
  })
}
