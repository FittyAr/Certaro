<script setup lang="ts">
import Chart from 'primevue/chart'
import SelectButton from 'primevue/selectbutton'
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import DateText from '@/components/domain/DateText.vue'
import KpiCard from '@/components/domain/KpiCard.vue'
import ListState from '@/components/domain/ListState.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
import PercentBar from '@/components/domain/PercentBar.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import { useApiError, type ApiError } from '@/composables/useApiError'
import { useMoney } from '@/composables/useMoney'
import { useShortcuts } from '@/composables/useShortcuts'
import { useDashboardStore, PERIODOS, type PeriodoDashboard } from '@/stores/useDashboardStore'
import { useUiStore } from '@/stores/useUiStore'

/**
 * The dashboard of `docs/09-modulos-funcionales.md` §3.1. Every figure on this screen is computed
 * by the backend: the only arithmetic here is turning an amount into a number to draw the chart,
 * which doc 06 §9.8 allows and confines to `puntos`.
 *
 * The quotes block is loaded separately and is allowed to come back empty. When the external
 * service is down the screen is complete without it and no error is shown (doc 13 §2.4).
 */

const { t } = useI18n()
const router = useRouter()
const store = useDashboardStore()
const ui = useUiStore()
const { notify } = useApiError()
const { format } = useMoney()

const error = ref<ApiError | null>(null)

const stats = computed(() => store.stats)

const opcionesPeriodo = computed(() =>
  PERIODOS.map((value) => ({ value, label: t(`Dashboard.Period.${value}`) })),
)

async function cargar(): Promise<void> {
  error.value = null
  try {
    await Promise.all([store.fetchStats(), store.fetchAlertas()])
  } catch (e) {
    error.value = notify(e)
  }
  // Kept out of the block above: a quote that cannot be fetched is not an error of this screen.
  try {
    await store.fetchCotizaciones()
  } catch {
    /* doc 13 §2.4: accessory information, silently absent. */
  }
}

function cambiarPeriodo(value: PeriodoDashboard | null): void {
  if (!value || value === store.periodo) return
  store.periodo = value
  void cargar()
}

/** The chart needs numbers, and this is the one place allowed to produce them (doc 06 §9.8). */
const serie = computed(() => {
  const puntos = stats.value?.serieMensual ?? []
  return {
    labels: puntos.map((p) => t(`Dashboard.Mes.M${p.mes}`)),
    ingresos: puntos.map((p) => Number(p.ingresos)),
    gastos: puntos.map((p) => Number(p.gastos)),
  }
})

/**
 * Chart.js paints on a canvas, where a CSS variable does not resolve, so the token is read from
 * the document and handed over already resolved. The colour still comes from the design system.
 */
function token(name: string): string {
  if (typeof document === 'undefined') return 'transparent'
  const value = getComputedStyle(document.documentElement).getPropertyValue(name).trim()
  return value ? `hsl(${value})` : 'transparent'
}

const chartData = computed(() => ({
  labels: serie.value.labels,
  datasets: [
    {
      label: t('Dashboard.Ingresos'),
      data: serie.value.ingresos,
      borderColor: token('--money-positive'),
      backgroundColor: token('--money-positive'),
      tension: 0.3,
    },
    {
      label: t('Dashboard.Gastos'),
      data: serie.value.gastos,
      borderColor: token('--money-negative'),
      backgroundColor: token('--money-negative'),
      tension: 0.3,
    },
  ],
}))

const chartOptions = computed(() => ({
  maintainAspectRatio: false,
  plugins: {
    legend: { labels: { color: token('--muted-foreground') } },
    tooltip: {
      // In privacy mode the tooltip would defeat the whole point of hiding the amounts.
      enabled: !ui.privacyMode,
      callbacks: {
        label: (ctx: { dataset: { label?: string }; parsed: { y: number } }) =>
          `${ctx.dataset.label ?? ''}: ${format(ctx.parsed.y.toFixed(4))}`,
      },
    },
  },
  scales: {
    x: { ticks: { color: token('--muted-foreground') }, grid: { display: false } },
    y: {
      // Privacy mode also blanks the value axis, as doc 09 §3.1 requires.
      ticks: { display: !ui.privacyMode, color: token('--muted-foreground') },
      grid: { color: token('--border') },
    },
  },
}))

/** Size of the database, in megabytes with one decimal. Not money: plain locale-free arithmetic. */
const tamanoMb = computed(() => {
  const bytes = stats.value?.estadoSistema?.tamanoBytes ?? 0
  return (bytes / (1024 * 1024)).toFixed(1)
})

const severidadClase: Record<string, string> = {
  Info: 'border-l-primary',
  Warning: 'border-l-warning',
  Error: 'border-l-destructive',
}

function irA(destino: string): void {
  void router.push(destino)
}

useShortcuts({ 'ctrl+r': () => void cargar() })

onMounted(() => {
  store.restorePeriodo()
  void cargar()
})
</script>

<template>
  <section class="flex h-full flex-col gap-6 p-6">
    <PageHeader :title="$t('Menu.Dashboard')">
      <template #actions>
        <SelectButton
          :model-value="store.periodo"
          :options="opcionesPeriodo"
          option-label="label"
          option-value="value"
          :allow-empty="false"
          size="small"
          :aria-label="$t('Dashboard.Periodo')"
          @update:model-value="cambiarPeriodo($event)"
        />
        <Button
          variant="ghost"
          size="icon"
          :aria-label="$t('General.PrivacyMode')"
          @click="ui.togglePrivacy()"
        >
          <AppIcon :name="ui.privacyMode ? 'eye-off' : 'eye'" :size="16" />
        </Button>
        <Button variant="outline" :disabled="store.loading" @click="cargar()">
          <AppIcon name="refresh-cw" :size="16" />
          {{ $t('General.Refresh') }}
        </Button>
      </template>
    </PageHeader>

    <ListState
      :loading="store.loading"
      :first-load="store.firstLoad"
      :error="error"
      :is-empty="!stats"
      :is-filtered="false"
      empty-key="Dashboard.Empty"
      @retry="cargar()"
    >
      <div v-if="stats" class="space-y-6">
        <!-- Quotes. Absent, not failed, when the service is unreachable. -->
        <div v-if="store.cotizaciones?.length" class="flex flex-wrap gap-3">
          <article
            v-for="cotizacion in store.cotizaciones"
            :key="cotizacion.casa"
            class="rounded-md border border-border bg-surface-card px-3 py-2 text-sm"
          >
            <p class="text-xs text-muted-foreground">{{ cotizacion.nombre }}</p>
            <p class="flex items-center gap-3">
              <span>
                {{ $t('Cotizaciones.Compra') }} <MoneyText :value="cotizacion.compra" />
              </span>
              <span>
                {{ $t('Cotizaciones.Venta') }} <MoneyText :value="cotizacion.venta" />
              </span>
            </p>
            <p v-if="cotizacion.desactualizada" class="text-xs text-muted-foreground">
              {{ $t('Cotizaciones.Desactualizada') }}
              <DateText :value="cotizacion.fechaActualizacion" />
            </p>
          </article>
        </div>

        <!-- Alerts. Each one navigates to its module with the filter already applied. -->
        <div v-if="store.alertas?.length" class="grid gap-2 md:grid-cols-2 xl:grid-cols-3">
          <button
            v-for="alerta in store.alertas"
            :key="alerta.tipo"
            type="button"
            class="flex items-center gap-2 rounded-md border border-border border-l-4 bg-surface-card px-3 py-2 text-left text-sm hover:bg-muted"
            :class="severidadClase[alerta.severidad]"
            @click="irA(alerta.destino)"
          >
            <AppIcon name="triangle-alert" :size="16" />
            <!-- The amount goes through MoneyText so privacy mode covers it as well. -->
            <span class="flex-1">
              {{ $t(alerta.clave, { cantidad: alerta.cantidad }) }}
              <MoneyText v-if="alerta.monto" :value="alerta.monto" colored />
            </span>
            <AppIcon name="chevron-right" :size="16" />
          </button>
        </div>

        <div class="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
          <KpiCard
            :label="$t('Dashboard.Ingresos')"
            :value="stats.totalIngresos"
            :variacion="stats.variacionIngresos"
          />
          <KpiCard
            :label="$t('Dashboard.Gastos')"
            :value="stats.totalGastos"
            :variacion="stats.variacionGastos"
          />
          <KpiCard
            :label="$t('Dashboard.Balance')"
            :value="stats.balance"
            :variacion="stats.variacionBalance"
            colored
          />
          <KpiCard
            :label="$t('Dashboard.Movimientos')"
            :count="stats.cantidadMovimientos"
          />
        </div>

        <div class="grid gap-3 sm:grid-cols-2 xl:grid-cols-5">
          <KpiCard :label="$t('Dashboard.ClientesActivos')" :count="stats.clientesActivos" />
          <KpiCard :label="$t('Dashboard.TrabajosPendientes')" :count="stats.trabajosPendientes" />
          <KpiCard :label="$t('Dashboard.ObrasPausadas')" :count="stats.obrasPausadas" />
          <KpiCard :label="$t('Dashboard.FacturasVencidas')" :count="stats.facturasVencidas" />
          <KpiCard
            :label="$t('Dashboard.LiquidacionesPendientes')"
            :count="stats.liquidacionesPendientes"
          />
        </div>

        <section class="rounded-lg border border-border bg-surface-card p-4">
          <h3 class="mb-3 text-sm font-semibold">{{ $t('Dashboard.SerieAnual') }}</h3>
          <Chart type="line" :data="chartData" :options="chartOptions" class="h-64" />
        </section>

        <div class="grid gap-3 lg:grid-cols-2">
          <section class="rounded-lg border border-border bg-surface-card p-4">
            <h3 class="mb-3 text-sm font-semibold">{{ $t('Dashboard.TopClientes') }}</h3>
            <p v-if="!stats.topClientes?.length" class="text-xs text-muted-foreground">
              {{ $t('Dashboard.SinDatos') }}
            </p>
            <ul v-else class="divide-y divide-border text-sm">
              <li
                v-for="(cliente, index) in stats.topClientes"
                :key="cliente.id ?? index"
                class="flex items-center gap-2 py-2"
              >
                <span class="w-5 text-xs text-muted-foreground tabular-nums">{{ index + 1 }}</span>
                <span class="flex-1 truncate">{{ cliente.nombre }}</span>
                <MoneyText :value="cliente.total" />
              </li>
            </ul>
          </section>

          <section class="rounded-lg border border-border bg-surface-card p-4">
            <h3 class="mb-3 text-sm font-semibold">{{ $t('Dashboard.GastosPorCategoria') }}</h3>
            <p v-if="!stats.gastosPorCategoria?.length" class="text-xs text-muted-foreground">
              {{ $t('Dashboard.SinDatos') }}
            </p>
            <ul v-else class="divide-y divide-border text-sm">
              <li
                v-for="(categoria, index) in stats.gastosPorCategoria"
                :key="categoria.id ?? index"
                class="flex items-center gap-2 py-2"
              >
                <span class="flex-1 truncate">{{ categoria.nombre }}</span>
                <MoneyText :value="categoria.total" />
              </li>
            </ul>
          </section>
        </div>

        <div class="grid gap-3 lg:grid-cols-2">
          <section class="rounded-lg border border-border bg-surface-card p-4">
            <h3 class="mb-3 text-sm font-semibold">{{ $t('Dashboard.MejoresObras') }}</h3>
            <p v-if="!stats.mejoresObras?.length" class="text-xs text-muted-foreground">
              {{ $t('Dashboard.SinDatos') }}
            </p>
            <ul v-else class="space-y-2 text-sm">
              <li v-for="obra in stats.mejoresObras" :key="obra.id" class="space-y-1">
                <div class="flex items-center gap-2">
                  <span class="flex-1 truncate">{{ obra.nombre }}</span>
                  <MoneyText :value="obra.rentabilidad" colored />
                </div>
                <PercentBar :value="obra.margenPorcentaje" />
              </li>
            </ul>
          </section>

          <section class="rounded-lg border border-border bg-surface-card p-4">
            <h3 class="mb-3 text-sm font-semibold">{{ $t('Dashboard.PeoresObras') }}</h3>
            <p v-if="!stats.peoresObras?.length" class="text-xs text-muted-foreground">
              {{ $t('Dashboard.SinDatos') }}
            </p>
            <ul v-else class="space-y-2 text-sm">
              <li v-for="obra in stats.peoresObras" :key="obra.id" class="space-y-1">
                <div class="flex items-center gap-2">
                  <span class="flex-1 truncate">{{ obra.nombre }}</span>
                  <MoneyText :value="obra.rentabilidad" colored />
                </div>
                <PercentBar :value="obra.margenPorcentaje" />
              </li>
            </ul>
          </section>
        </div>

        <section class="rounded-lg border border-border bg-surface-card p-4">
          <h3 class="mb-3 text-sm font-semibold">{{ $t('Dashboard.UltimosMovimientos') }}</h3>
          <p v-if="!stats.ultimosMovimientos?.length" class="text-xs text-muted-foreground">
            {{ $t('Dashboard.SinDatos') }}
          </p>
          <ul v-else class="divide-y divide-border text-sm">
            <li
              v-for="movimiento in stats.ultimosMovimientos"
              :key="movimiento.id"
              class="flex items-center gap-3 py-2"
            >
              <DateText :value="movimiento.fecha" class="text-muted-foreground" />
              <span class="flex-1 truncate">{{ movimiento.concepto }}</span>
              <MoneyText :value="movimiento.total" :colored="movimiento.esIngreso" />
            </li>
          </ul>
        </section>

        <section class="rounded-lg border border-border bg-surface-card p-4 text-sm">
          <h3 class="mb-3 text-sm font-semibold">{{ $t('Dashboard.EstadoSistema') }}</h3>
          <dl class="grid grid-cols-2 gap-3 md:grid-cols-4">
            <div>
              <dt class="text-xs text-muted-foreground">{{ $t('Dashboard.Version') }}</dt>
              <dd class="tabular-nums">{{ stats.estadoSistema?.version ?? '-' }}</dd>
            </div>
            <div>
              <dt class="text-xs text-muted-foreground">{{ $t('Dashboard.EstadoBase') }}</dt>
              <dd :class="stats.estadoSistema?.baseSaludable ? 'text-success' : 'text-destructive'">
                {{ $t(stats.estadoSistema?.estado ?? 'Dashboard.Estado.Saludable') }}
              </dd>
            </div>
            <div>
              <dt class="text-xs text-muted-foreground">{{ $t('Dashboard.Migraciones') }}</dt>
              <dd class="tabular-nums">{{ stats.estadoSistema?.migraciones ?? 0 }}</dd>
            </div>
            <div>
              <dt class="text-xs text-muted-foreground">{{ $t('Dashboard.TamanoBase') }}</dt>
              <dd class="tabular-nums">
                {{ $t('Dashboard.Megabytes', { valor: tamanoMb }) }}
              </dd>
            </div>
          </dl>
        </section>
      </div>
    </ListState>
  </section>
</template>
