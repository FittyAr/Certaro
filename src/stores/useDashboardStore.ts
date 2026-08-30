import { defineStore } from 'pinia'
import { ref } from 'vue'

import {
  cotizacionesGet,
  cotizacionesRefresh,
  dashboardAlertas,
  dashboardStats,
  type Alerta,
  type Cotizacion,
  type DashboardStats,
  type PeriodoDashboard,
} from '@/api/dashboard'

import { useConfigStore } from './useConfigStore'

export type {
  Alerta,
  Cotizacion,
  DashboardStats,
  EstadoSistema,
  PeriodoDashboard,
  PuntoSerie,
  RentabilidadItem,
  SeveridadAlerta,
  TipoAlerta,
  TopCliente,
} from '@/api/dashboard'
export { PERIODOS } from '@/api/dashboard'

/** The lowercase period of the configuration and the one the command takes are two encodings. */
const DESDE_CONFIG: Record<string, PeriodoDashboard> = {
  mensual: 'Mensual',
  anual: 'Anual',
  total: 'Total',
}

/**
 * The dashboard is read-only, so this store does hold its data: unlike a paginated list, the whole
 * screen is one snapshot and every block refreshes together.
 *
 * The quotes are kept apart from the stats. They come from an external service that is allowed to
 * be down (doc 13 §2.4), and a missing quote must not blank out the rest of the screen.
 */
export const useDashboardStore = defineStore('dashboard', () => {
  const periodo = ref<PeriodoDashboard>('Mensual')
  const stats = ref<DashboardStats | null>(null)
  const cotizaciones = ref<Cotizacion[]>([])
  const alertas = ref<Alerta[]>([])
  const loading = ref(false)
  const firstLoad = ref(true)

  /** Adopts the period the user left selected last time (doc 14 §2.5). */
  function restorePeriodo(): void {
    const guardado = useConfigStore().config?.dashboard.lastPeriod
    if (guardado && DESDE_CONFIG[guardado]) periodo.value = DESDE_CONFIG[guardado]
  }

  async function fetchStats(next?: PeriodoDashboard): Promise<DashboardStats> {
    if (next) periodo.value = next
    loading.value = true
    try {
      const cargado = await dashboardStats(periodo.value)
      stats.value = cargado
      return cargado
    } finally {
      loading.value = false
      firstLoad.value = false
    }
  }

  async function fetchAlertas(): Promise<Alerta[]> {
    const cargadas = await dashboardAlertas(periodo.value)
    alertas.value = cargadas
    return cargadas
  }

  /** Never rejects for the caller: an unreachable service leaves the block simply absent. */
  async function fetchCotizaciones(forzar = false): Promise<Cotizacion[]> {
    const cargadas = await (forzar ? cotizacionesRefresh() : cotizacionesGet())
    cotizaciones.value = cargadas
    return cargadas
  }

  return {
    periodo,
    stats,
    cotizaciones,
    alertas,
    loading,
    firstLoad,
    restorePeriodo,
    fetchStats,
    fetchAlertas,
    fetchCotizaciones,
  }
})
