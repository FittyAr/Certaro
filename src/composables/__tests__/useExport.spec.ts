import { effectScope } from 'vue'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import type * as reportesApi from '@/api/reportes'
import { nombreDeArchivo, useExport } from '@/composables/useExport'

/** Behaviour required by `docs/12-reportes-y-exportaciones.md` §1.2. */

const save = vi.fn()
const add = vi.fn()
const notify = vi.fn()
const nombreSugerido = vi.fn()

vi.mock('@tauri-apps/plugin-dialog', () => ({ save: (...args: unknown[]) => save(...args) }))
vi.mock('primevue/usetoast', () => ({ useToast: () => ({ add }) }))
vi.mock('vue-i18n', () => ({ useI18n: () => ({ t: (key: string) => key }) }))
vi.mock('@/composables/useApiError', () => ({ useApiError: () => ({ notify }) }))
vi.mock('@/api/reportes', async (original) => ({
  ...(await original<typeof reportesApi>()),
  nombreSugerido: (...args: unknown[]) => nombreSugerido(...args),
}))

function exportador() {
  const scope = effectScope()
  return scope.run(() => useExport())!
}

function resultado(registros = 3) {
  return { ruta: 'D:\\tmp\\Movimientos.pdf', bytes: 100, registros }
}

describe('useExport', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    nombreSugerido.mockResolvedValue('Movimientos_20260830_101112.pdf')
  })

  it('propone el nombre que da el backend y exporta al destino elegido', async () => {
    save.mockResolvedValue('D:\\tmp\\Movimientos.pdf')
    const run = vi.fn().mockResolvedValue(resultado())
    const { exportar } = exportador()

    await exportar({ reporte: 'movimientos', formato: 'Pdf', run })

    expect(save).toHaveBeenCalledWith(
      expect.objectContaining({ defaultPath: 'Movimientos_20260830_101112.pdf' }),
    )
    expect(run).toHaveBeenCalledWith('D:\\tmp\\Movimientos.pdf')
  })

  it('un dialogo cancelado no exporta ni avisa', async () => {
    save.mockResolvedValue(null)
    const run = vi.fn()
    const { exportar } = exportador()

    expect(await exportar({ reporte: 'movimientos', formato: 'Pdf', run })).toBeNull()
    expect(run).not.toHaveBeenCalled()
    expect(add).not.toHaveBeenCalled()
    expect(notify).not.toHaveBeenCalled()
  })

  it('el aviso de exito dice cuantos registros salieron', async () => {
    save.mockResolvedValue('D:\\tmp\\Movimientos.pdf')
    const { exportar } = exportador()

    await exportar({
      reporte: 'movimientos',
      formato: 'Pdf',
      run: () => Promise.resolve(resultado(42)),
    })

    expect(add).toHaveBeenCalledWith(
      expect.objectContaining({ severity: 'success', summary: 'Export.Listo' }),
    )
  })

  it('un fallo del backend se notifica y libera el boton', async () => {
    save.mockResolvedValue('D:\\tmp\\Movimientos.pdf')
    const { exportar, exportando } = exportador()

    expect(
      await exportar({
        reporte: 'movimientos',
        formato: 'Pdf',
        run: () => Promise.reject(new Error('boom')),
      }),
    ).toBeNull()
    expect(notify).toHaveBeenCalled()
    expect(exportando.value).toBe(false)
  })

  it('el filtro de formato usa la extension del formato pedido', async () => {
    save.mockResolvedValue('D:\\tmp\\Movimientos.xlsx')
    const { exportar } = exportador()

    await exportar({
      reporte: 'movimientos',
      formato: 'Xlsx',
      run: () => Promise.resolve(resultado()),
    })

    expect(save).toHaveBeenCalledWith(
      expect.objectContaining({ filters: [{ name: 'Export.Tipo.Xlsx', extensions: ['xlsx'] }] }),
    )
  })

  it('el aviso muestra el nombre del archivo, no la ruta entera', () => {
    expect(nombreDeArchivo('D:\\carpeta\\Movimientos.pdf')).toBe('Movimientos.pdf')
    expect(nombreDeArchivo('/home/eo/Movimientos.pdf')).toBe('Movimientos.pdf')
    expect(nombreDeArchivo('Movimientos.pdf')).toBe('Movimientos.pdf')
  })
})
