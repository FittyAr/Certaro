import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import { useConfigStore } from '@/stores/useConfigStore'
import { useDashboardStore } from '@/stores/useDashboardStore'

const calls: Array<{ command: string; args?: Record<string, unknown> }> = []
let respuesta: unknown = null
let falla = false

vi.mock('@/api/client', () => ({
  callCommand: (command: string, args?: Record<string, unknown>) => {
    calls.push({ command, args })
    return falla ? Promise.reject(new Error('down')) : Promise.resolve(respuesta)
  },
}))

describe('useDashboardStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    calls.length = 0
    respuesta = null
    falla = false
  })

  it('el periodo por defecto es mensual', () => {
    expect(useDashboardStore().periodo).toBe('Mensual')
  })

  it('adopta el periodo que quedo guardado en la configuracion', () => {
    useConfigStore().config = { dashboard: { lastPeriod: 'anual' } } as never
    const store = useDashboardStore()
    store.restorePeriodo()
    expect(store.periodo).toBe('Anual')
  })

  it('un periodo desconocido en la configuracion no rompe el default', () => {
    useConfigStore().config = { dashboard: { lastPeriod: 'trimestral' } } as never
    const store = useDashboardStore()
    store.restorePeriodo()
    expect(store.periodo).toBe('Mensual')
  })

  it('pide las estadisticas con el periodo elegido y lo recuerda', async () => {
    respuesta = { periodo: 'Anual', totalIngresos: '10.0000' }
    const store = useDashboardStore()
    await store.fetchStats('Anual')
    expect(calls[0]).toEqual({ command: 'dashboard_stats', args: { periodo: 'Anual' } })
    expect(store.periodo).toBe('Anual')
    expect(store.stats?.totalIngresos).toBe('10.0000')
    expect(store.firstLoad).toBe(false)
  })

  it('un fallo deja de cargar y propaga el error a la vista', async () => {
    falla = true
    const store = useDashboardStore()
    await expect(store.fetchStats()).rejects.toThrow()
    expect(store.loading).toBe(false)
  })

  it('el refresco explicito usa el comando que ignora la cache', async () => {
    respuesta = []
    const store = useDashboardStore()
    await store.fetchCotizaciones()
    await store.fetchCotizaciones(true)
    expect(calls.map((c) => c.command)).toEqual(['cotizaciones_get', 'cotizaciones_refresh'])
  })

  it('las alertas viajan con el periodo vigente', async () => {
    respuesta = [{ tipo: 'ObrasPausadas', cantidad: 2 }]
    const store = useDashboardStore()
    store.periodo = 'Total'
    await store.fetchAlertas()
    expect(calls[0]).toEqual({ command: 'dashboard_alertas', args: { periodo: 'Total' } })
    expect(store.alertas).toHaveLength(1)
  })
})
