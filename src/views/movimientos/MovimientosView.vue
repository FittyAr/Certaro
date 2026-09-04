<script setup lang="ts">
import Column from 'primevue/column'
import Divider from 'primevue/divider'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute } from 'vue-router'

import DataGrid from '@/components/domain/DataGrid.vue'
import DateInput from '@/components/domain/DateInput.vue'
import DateText from '@/components/domain/DateText.vue'
import ExportMenu from '@/components/domain/ExportMenu.vue'
import FilterBar from '@/components/domain/FilterBar.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import HelpButton from '@/components/ui/HelpButton.vue'
import { Button } from '@/components/ui/button'
import { useConfirmDelete } from '@/composables/useConfirmDelete'
import { useServerTable } from '@/composables/useServerTable'
import { useShortcuts } from '@/composables/useShortcuts'
import { useCatalogStore, type LookupItem } from '@/stores/useCatalogStore'
import { useClientesStore } from '@/stores/useClientesStore'
import { useEmpleadosStore } from '@/stores/useEmpleadosStore'
import { useMovimientosStore } from '@/stores/useMovimientosStore'
import { useProyectosStore } from '@/stores/useProyectosStore'
import { useReportesStore } from '@/stores/useReportesStore'
import type {
  Moneda,
  MovimientoFiltro,
  MovimientoListItem,
  MovimientoResumen,
} from '@/stores/useMovimientosStore'
import MovimientoDrawer from './components/MovimientoDrawer.vue'

/**
 * The cash ledger. See `docs/09-modulos-funcionales.md` §3.2.
 *
 * This is the only screen whose filtering and paging happen on the server: the table grows without
 * bound and cannot be shipped whole to the frontend. The totals under the table describe the whole
 * filter, not the visible page.
 */

const { t } = useI18n()
const route = useRoute()
const { confirmDelete } = useConfirmDelete()
const store = useMovimientosStore()
const catalog = useCatalogStore()
const reportes = useReportesStore()
const empleados = useEmpleadosStore()
const clientes = useClientesStore()
const proyectos = useProyectosStore()

const drawerRef = ref<InstanceType<typeof MovimientoDrawer> | null>(null)

const table = useServerTable<MovimientoFiltro, MovimientoListItem, MovimientoResumen>({
  key: 'movimientos',
  initialFilter: { concepto: '' },
  fetch: (query) => store.fetchPaged(query),
  defaultSort: { field: 'fecha', dir: 'Desc' },
})

const tipos = ref<LookupItem[]>([])
const categorias = ref<LookupItem[]>([])
const opcionesEmpleado = ref<LookupItem[]>([])
const opcionesCliente = ref<LookupItem[]>([])
const opcionesProyecto = ref<LookupItem[]>([])

async function cargarSelectores(): Promise<void> {
  const [t, c, emp, cli, proy] = await Promise.all([
    catalog.loadTiposMovimiento(),
    catalog.loadCategorias(),
    empleados.fetchLookup(true),
    clientes.lookup(undefined, 200),
    proyectos.lookup(undefined, undefined, 200),
  ])
  tipos.value = t
  categorias.value = c
  opcionesEmpleado.value = emp
  opcionesCliente.value = cli
  opcionesProyecto.value = proy
}

const monedaFilterOptions = computed<{ label: string; value: Moneda | undefined }[]>(() => [
  { label: t('General.All'), value: undefined },
  { label: t('Movimientos.Moneda.Ars'), value: 'Ars' },
  { label: t('Movimientos.Moneda.Usd'), value: 'Usd' },
])

const resumen = computed(() => table.summary.value)

const filtrosActivos = computed(() =>
  Boolean(
    table.filter.value.concepto ||
    table.filter.value.tipoMovimientoId ||
    table.filter.value.categoriaId ||
    table.filter.value.clienteId ||
    table.filter.value.proyectoId ||
    table.filter.value.moneda ||
    table.filter.value.fechaDesde ||
    table.filter.value.fechaHasta,
  ),
)

function onDelete(row: MovimientoListItem): void {
  confirmDelete({
    entityKey: 'Entity.Movimiento',
    label: row.concepto,
    action: () => store.remove(row.id, row.rowVersion),
    onDone: () => table.reload(),
  })
}

function movimientoContextMenu(row: MovimientoListItem) {
  return [
    {
      label: t('General.Edit'),
      icon: 'pi pi-pencil',
      disabled: row.bloqueadoPorLiquidacion,
      command: () => drawerRef.value?.openEdit(row.id),
    },
    { separator: true },
    {
      label: t('General.Delete'),
      icon: 'pi pi-trash',
      disabled: row.bloqueadoPorLiquidacion,
      command: () => onDelete(row),
    },
  ]
}

useShortcuts({ 'ctrl+n': () => drawerRef.value?.openCreate() })

onMounted(async () => {
  if (route.query.filtroProyectoId) {
    table.filter.value.proyectoId = String(route.query.filtroProyectoId)
  }
  if (route.query.filtroClienteId) {
    table.filter.value.clienteId = String(route.query.filtroClienteId)
  }
  table.start()
  await cargarSelectores()
  if (route.query.proyectoId) {
    await drawerRef.value?.openCreate({
      proyectoId: String(route.query.proyectoId),
      clienteId: route.query.clienteId ? String(route.query.clienteId) : undefined,
    })
  }
})
</script>

<template>
  <section class="flex h-full flex-col gap-4 p-6">
    <PageHeader :title="$t('Menu.Movimientos')" :subtitle="$t('Movimientos.Subtitle')">
      <template #actions>
        <!-- Exports the filter, not the page: the count in the tooltip says so. -->
        <ExportMenu
          reporte="movimientos"
          :cantidad="resumen?.cantidad"
          :run="(formato, destino) => reportes.exportMovimientos(table.filter.value, formato, destino)"
        />
        <Button @click="drawerRef?.openCreate()">
          <AppIcon name="plus" :size="16" />
          {{ $t('General.New') }}
        </Button>
        <HelpButton topic-id="movimientos-overview" title="Ayuda sobre el Libro de Movimientos y Caja" />
      </template>
    </PageHeader>

    <FilterBar :active="filtrosActivos" @clear="table.resetFilter()">
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('Movimientos.Concepto') }}</span>
        <InputText v-model="table.filter.value.concepto" :placeholder="$t('General.Search')" />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('Movimientos.Tipo') }}</span>
        <Select
          v-model="table.filter.value.tipoMovimientoId"
          :options="tipos"
          option-label="label"
          option-value="id"
          show-clear
          :placeholder="$t('General.All')"
        />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('Movimientos.Categoria') }}</span>
        <Select
          v-model="table.filter.value.categoriaId"
          :options="categorias"
          option-label="label"
          option-value="id"
          show-clear
          filter
          :placeholder="$t('General.All')"
        />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('Movimientos.Cliente') }}</span>
        <Select
          v-model="table.filter.value.clienteId"
          :options="opcionesCliente"
          option-label="label"
          option-value="id"
          show-clear
          filter
          :placeholder="$t('General.All')"
        />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('Movimientos.Proyecto') }}</span>
        <Select
          v-model="table.filter.value.proyectoId"
          :options="opcionesProyecto"
          option-label="label"
          option-value="id"
          show-clear
          filter
          :placeholder="$t('General.All')"
        />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('Movimientos.Moneda.Label') }}</span>
        <Select
          v-model="table.filter.value.moneda"
          :options="monedaFilterOptions"
          option-label="label"
          option-value="value"
          show-clear
          :placeholder="$t('General.All')"
        />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('Movimientos.Desde') }}</span>
        <DateInput v-model="table.filter.value.fechaDesde" />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('Movimientos.Hasta') }}</span>
        <DateInput v-model="table.filter.value.fechaHasta" />
      </label>
    </FilterBar>

    <Divider />

    <DataGrid
      :table="table"
      empty-key="Movimientos.Empty"
      class="flex-1"
      :context-menu-items="movimientoContextMenu"
      @row-edit="(row: any) => drawerRef?.openEdit(row.id)"
    >
      <Column field="fecha" :header="$t('Movimientos.Fecha')" sortable>
        <template #body="{ data }"><DateText :value="data.fecha" instant /></template>
      </Column>
      <Column field="concepto" :header="$t('Movimientos.Concepto')" sortable />
      <Column field="tipoMovimientoNombre" :header="$t('Movimientos.Tipo')" sortable />
      <Column field="categoriaNombre" :header="$t('Movimientos.Categoria')" sortable>
        <template #body="{ data }">
          <span class="flex items-center gap-2">
            <span
              v-if="data.categoriaColor"
              class="inline-block size-3 rounded-full border border-border"
              :style="{ backgroundColor: data.categoriaColor }"
            />
            {{ data.categoriaNombre ?? '—' }}
          </span>
        </template>
      </Column>
      <Column :header="$t('Movimientos.ImputacionOpcional')">
        <template #body="{ data }">
          <div v-if="data.proyectoNombre || data.clienteNombre" class="text-xs leading-tight">
            <div v-if="data.proyectoNombre" class="font-medium text-foreground flex items-center gap-1">
              <AppIcon name="briefcase" :size="12" class="text-muted-foreground" />
              <span>{{ data.proyectoNombre }}</span>
            </div>
            <div v-if="data.clienteNombre" class="text-muted-foreground">
              {{ data.clienteNombre }}
            </div>
          </div>
          <span v-else class="text-xs text-muted-foreground">—</span>
        </template>
      </Column>
      <Column field="monto" :header="$t('Movimientos.Monto')" sortable>
        <template #body="{ data }"><MoneyText :value="data.monto" /></template>
      </Column>
      <Column field="cantidad" :header="$t('Movimientos.Cantidad')">
        <template #body="{ data }">
          <span class="tabular-nums">{{ data.cantidad }}</span>
        </template>
      </Column>
      <Column field="total" :header="$t('Movimientos.Total')" sortable>
        <template #body="{ data }">
          <!-- Coloured by the sign of the type, which is the only place the sign lives. -->
          <div class="flex items-center gap-1.5">
            <MoneyText :value="data.esIngreso ? data.total : `-${data.total}`" colored />
            <span
              v-if="data.moneda === 'Usd'"
              class="rounded border border-warning/30 bg-warning/10 px-1 py-0.5 text-[10px] font-bold text-warning"
            >
              USD
            </span>
          </div>
        </template>
      </Column>

      <template #actions="{ data }">
        <div class="flex gap-1">
          <Button
            variant="ghost"
            size="sm"
            :disabled="data.bloqueadoPorLiquidacion"
            :title="
              data.bloqueadoPorLiquidacion ? $t('Movimientos.BloqueadoLiquidacion') : undefined
            "
            :aria-label="$t('General.Edit')"
            @click="drawerRef?.openEdit(data.id)"
          >
            <AppIcon name="pencil" :size="14" />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            :disabled="data.bloqueadoPorLiquidacion"
            :aria-label="$t('General.Delete')"
            @click="onDelete(data)"
          >
            <AppIcon name="trash-2" :size="14" />
          </Button>
        </div>
      </template>
    </DataGrid>

    <!-- The summary covers every matching row, so it does not change when paging. -->
    <div
      v-if="resumen"
      class="flex flex-wrap items-center gap-6 rounded-md border border-border bg-surface-raised px-4 py-3 text-sm"
    >
      <span class="text-muted-foreground">
        {{ $t('Movimientos.ResumenCantidad', { count: resumen.cantidad }) }}
      </span>
      <span class="flex items-center gap-2">
        <span class="text-muted-foreground">{{ $t('Movimientos.Ingresos') }}</span>
        <MoneyText :value="resumen.totalIngresos" colored />
      </span>
      <span class="flex items-center gap-2">
        <span class="text-muted-foreground">{{ $t('Movimientos.Gastos') }}</span>
        <MoneyText :value="`-${resumen.totalGastos}`" colored />
      </span>
      <span class="flex items-center gap-2 font-medium">
        <span class="text-muted-foreground">{{ $t('Movimientos.Balance') }}</span>
        <MoneyText :value="resumen.balance" colored />
      </span>
    </div>

    <MovimientoDrawer
      ref="drawerRef"
      :tipos="tipos"
      :categorias="categorias"
      :opciones-empleado="opcionesEmpleado"
      :opciones-cliente="opcionesCliente"
      :opciones-proyecto="opcionesProyecto"
      @saved="table.reload()"
    />
  </section>
</template>
