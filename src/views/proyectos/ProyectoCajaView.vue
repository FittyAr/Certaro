<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import DataTable from 'primevue/datatable'
import Column from 'primevue/column'
import DateText from '@/components/domain/DateText.vue'
import ListState from '@/components/domain/ListState.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import HelpButton from '@/components/ui/HelpButton.vue'
import { useApiError, type ApiError } from '@/composables/useApiError'
import { useMovimientosStore, type MovimientoListItem } from '@/stores/useMovimientosStore'
import { useProyectosStore, type ProyectoDetalle } from '@/stores/useProyectosStore'

const route = useRoute()
const router = useRouter()
const { notify } = useApiError()
const store = useMovimientosStore()
const proyectos = useProyectosStore()

const proyectoId = computed(() => String(route.params.proyectoId ?? ''))
const proyecto = ref<ProyectoDetalle | null>(null)
const items = ref<MovimientoListItem[]>([])
const loading = ref(false)
const firstLoad = ref(true)
const error = ref<ApiError | null>(null)

const resumen = ref<{ totalIngresos: string; totalGastos: string; balance: string } | null>(null)

async function cargar(): Promise<void> {
  loading.value = true
  error.value = null
  try {
    const [res, proy] = await Promise.all([
      store.fetchPaged({
        page: 1,
        pageSize: 500,
        filtro: { proyectoId: proyectoId.value },
        sortDir: 'Desc',
      }),
      proyectos.fetchOne(proyectoId.value).catch(() => null),
    ])
    items.value = res.items
    proyecto.value = proy
    if (res.resumen) {
      resumen.value = {
        totalIngresos: res.resumen.totalIngresos,
        totalGastos: res.resumen.totalGastos,
        balance: res.resumen.balance,
      }
    }
  } catch (e) {
    error.value = notify(e)
  } finally {
    loading.value = false
    firstLoad.value = false
  }
}

const totalIngresos = computed(() =>
  resumen.value
    ? resumen.value.totalIngresos
    : items.value
        .filter((m) => m.esIngreso)
        .reduce((acc, m) => acc + Number(m.total), 0)
        .toFixed(4),
)

const totalGastos = computed(() =>
  resumen.value
    ? resumen.value.totalGastos
    : items.value
        .filter((m) => !m.esIngreso)
        .reduce((acc, m) => acc + Number(m.total), 0)
        .toFixed(4),
)

const balanceNeto = computed(() =>
  resumen.value
    ? resumen.value.balance
    : (Number(totalIngresos.value) - Number(totalGastos.value)).toFixed(4),
)

function irARegistrarMovimiento(): void {
  void router.push({
    path: '/movimientos',
    query: {
      proyectoId: proyectoId.value,
      clienteId: proyecto.value?.clienteId ?? undefined,
      tipoMovimientoId: '00000000-0000-0000-0000-000000000002',
    },
  })
}

onMounted(cargar)
</script>

<template>
  <section class="flex h-full flex-col gap-4 p-6">
    <PageHeader
      :title="$t('Proyectos.Caja.Title')"
      :subtitle="proyecto ? `${proyecto.nombre} · #${proyecto.numero}` : $t('Movimientos.Subtitle')"
    >
      <template #actions>
        <Button size="sm" @click="irARegistrarMovimiento()">
          <AppIcon name="plus" :size="16" />
          {{ $t('Movimientos.RegistrarGasto') }}
        </Button>
        <HelpButton topic-id="proyectos-caja" title="Ayuda sobre Caja de Proyecto" />
      </template>
    </PageHeader>

    <!-- Resumen financiero del proyecto -->
    <div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
      <div class="rounded-lg border border-border bg-card p-4 shadow-xs">
        <span class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
          {{ $t('Movimientos.Ingresos') }}
        </span>
        <div class="mt-1 text-xl font-bold">
          <MoneyText :value="totalIngresos" colored />
        </div>
      </div>
      <div class="rounded-lg border border-border bg-card p-4 shadow-xs">
        <span class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
          {{ $t('Movimientos.Gastos') }}
        </span>
        <div class="mt-1 text-xl font-bold">
          <MoneyText :value="Number(totalGastos) > 0 ? `-${totalGastos}` : '0.0000'" colored />
        </div>
      </div>
      <div class="rounded-lg border border-border bg-card p-4 shadow-xs">
        <span class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
          {{ $t('Movimientos.Balance') }}
        </span>
        <div class="mt-1 text-xl font-bold">
          <MoneyText :value="balanceNeto" colored show-sign />
        </div>
      </div>
    </div>

    <ListState
      :loading="loading"
      :first-load="firstLoad"
      :error="error"
      :is-empty="items.length === 0"
      :is-filtered="false"
      empty-key="Movimientos.Empty"
      class="flex-1"
      @retry="cargar()"
    >
      <DataTable
        :value="items"
        data-key="id"
        size="small"
        class="text-sm border border-border rounded-md overflow-hidden"
        paginator
        :rows="20"
        :rows-per-page-options="[10, 20, 50, 100]"
      >
        <Column field="fecha" :header="$t('Movimientos.Fecha')">
          <template #body="{ data }">
            <DateText :value="data.fecha" />
          </template>
        </Column>
        <Column field="concepto" :header="$t('Movimientos.Concepto')" />
        <Column field="tipoMovimientoNombre" :header="$t('Movimientos.Tipo')">
          <template #body="{ data }">
            <span
              class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium"
              :class="
                data.esIngreso
                  ? 'bg-success/10 text-success'
                  : 'bg-destructive/10 text-destructive'
              "
            >
              {{ data.tipoMovimientoNombre }}
            </span>
          </template>
        </Column>
        <Column field="total" :header="$t('Movimientos.Total')">
          <template #body="{ data }">
            <MoneyText
              :value="data.esIngreso ? data.total : (Number(data.total) > 0 ? `-${data.total}` : '0.0000')"
              colored
              show-sign
            />
          </template>
        </Column>
      </DataTable>
    </ListState>
  </section>
</template>
