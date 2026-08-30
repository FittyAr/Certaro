import { save } from '@tauri-apps/plugin-dialog'
import { useToast } from 'primevue/usetoast'
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'

import { EXTENSIONES, nombreSugerido, type ExportResult, type FormatoExport } from '@/api/reportes'
import { useApiError } from '@/composables/useApiError'

/**
 * The export flow, once, for every screen that exports. See `docs/12` §1.2 and `docs/16` §5.
 *
 * Ask for the name, open the dialog, generate, report. A cancelled dialog is not an error and
 * leaves no toast: the user changed their mind.
 */

export interface ExportOptions {
  /** Report identifier the backend knows: `movimientos`, `liquidacion`, `certificado`. */
  reporte: string
  formato: FormatoExport
  /** Names the subject of the report — the employee, the site — for the suggested filename. */
  detalle?: string
  /** Does the work once a destination exists. */
  run: (destino: string) => Promise<ExportResult>
}

export function useExport() {
  const toast = useToast()
  const { t } = useI18n()
  const { notify } = useApiError()

  /** True while a document is being generated, to disable the button that started it. */
  const exportando = ref(false)

  async function exportar(opts: ExportOptions): Promise<ExportResult | null> {
    if (exportando.value) return null

    const extension = EXTENSIONES[opts.formato]
    let destino: string | null
    try {
      destino = await save({
        defaultPath: await nombreSugerido(opts.reporte, opts.formato, opts.detalle),
        filters: [{ name: t(`Export.Tipo.${opts.formato}`), extensions: [extension] }],
      })
    } catch (e) {
      notify(e)
      return null
    }
    if (!destino) return null

    exportando.value = true
    try {
      const resultado = await opts.run(destino)
      toast.add({
        severity: 'success',
        summary: t('Export.Listo'),
        // The row count is what tells the user the filter was the one they meant.
        detail: t('Export.ListoDetalle', {
          registros: resultado.registros,
          archivo: nombreDeArchivo(resultado.ruta),
        }),
        life: 5000,
      })
      return resultado
    } catch (e) {
      notify(e)
      return null
    } finally {
      exportando.value = false
    }
  }

  return { exportar, exportando }
}

/** The last segment of a path, with either separator: the toast shows a name, not a path. */
export function nombreDeArchivo(ruta: string): string {
  const partes = ruta.split(/[/\\]/)
  return partes[partes.length - 1] || ruta
}
