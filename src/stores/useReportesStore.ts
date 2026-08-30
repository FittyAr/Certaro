import { defineStore } from 'pinia'

import {
  certificadoExport,
  liquidacionExport,
  movimientosExport,
  type ExportResult,
  type FormatoExport,
} from '@/api/reportes'
import type { Uuid } from '@/api/types'
import type { MovimientoFiltro } from '@/stores/useMovimientosStore'

export type { ExportResult, FormatoExport } from '@/api/reportes'
export { EXTENSIONES, FORMATOS_MOVIMIENTOS } from '@/api/reportes'

/**
 * Exports. See `docs/12-reportes-y-exportaciones.md`.
 *
 * Stateless on purpose: a report is generated and written, there is nothing to keep. The store
 * exists so a view never talks to `api/` directly, and so the count of generated reports has a
 * single place to live if it is ever needed.
 */
export const useReportesStore = defineStore('reportes', () => {
  function exportMovimientos(
    filtro: MovimientoFiltro,
    formato: FormatoExport,
    destino: string,
  ): Promise<ExportResult> {
    return movimientosExport(filtro, formato, destino)
  }

  function exportLiquidacion(id: Uuid, destino: string): Promise<ExportResult> {
    return liquidacionExport(id, destino)
  }

  function exportCertificado(id: Uuid, destino: string): Promise<ExportResult> {
    return certificadoExport(id, destino)
  }

  return { exportMovimientos, exportLiquidacion, exportCertificado }
})
