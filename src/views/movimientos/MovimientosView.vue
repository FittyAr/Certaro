<script setup lang="ts">
import Column from 'primevue/column'
import InputNumber from 'primevue/inputnumber'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'

import CrudDrawer from '@/components/domain/CrudDrawer.vue'
import DataGrid from '@/components/domain/DataGrid.vue'
import DateInput from '@/components/domain/DateInput.vue'
import DateText from '@/components/domain/DateText.vue'
import FieldError from '@/components/domain/FieldError.vue'
import FilterBar from '@/components/domain/FilterBar.vue'
import MoneyInput from '@/components/domain/MoneyInput.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import { useConfirmDelete } from '@/composables/useConfirmDelete'
import { useCrudDrawer } from '@/composables/useCrudDrawer'
import { useServerTable } from '@/composables/useServerTable'
import { useShortcuts } from '@/composables/useShortcuts'
import { useCatalogStore, type LookupItem } from '@/stores/useCatalogStore'
import {
  useMovimientosStore,
  type Moneda,
  type MovimientoFiltro,
  type MovimientoInput,
  type MovimientoListItem,
  type MovimientoResumen,
} from '@/stores/useMovimientosStore'

/**
 * The cash ledger. See `docs/09-modulos-funcionales.md` §3.2.
 *
 * This is the only screen whose filtering and paging happen on the server: the table grows without
 * bound and cannot be shipped whole to the frontend. The totals under the table describe the whole
 * filter, not the visible page.
 */

const { t } = useI18n()
const { confirmDelete } = useConfirmDelete()
const store = useMovimientosStore()
const catalog = useCatalogStore()

const table = useServerTable<MovimientoFiltro, MovimientoListItem, MovimientoResumen>({
  key: 'movimientos',
  initialFilter: { concepto: '' },
  fetch: (query) => store.fetchPaged(query),
  defaultSort: { field: 'fecha', dir: 'Desc' },
})

const tipos = ref<LookupItem[]>([])
const categorias = ref<LookupItem[]>([])

async function cargarSelectores(): Promise<void> {
  ;[tipos.value, categorias.value] = await Promise.all([
    catalog.loadTiposMovimiento(),
    catalog.loadCategorias(),
  ])
}

function vacio(): MovimientoInput & { rowVersion?: string } {
  return {
    fecha: new Date().toISOString(),
    concepto: '',
    monto: '0.0000',
    // RC-03: an ordinary movement is one unit, so the field is prefilled and usually untouched.
    cantidad: '1.0000',
    tipoMovimientoId: '',
    moneda: 'Ars',
    cotizacionAplicada: null,
    tipoConceptoPagoId: null,
    categoriaId: null,
    clienteId: null,
    trabajoId: null,
    empleadoId: null,
    facturaId: null,
  }
}

const drawer = useCrudDrawer<MovimientoInput & { rowVersion?: string }>({
  entityKey: 'Entity.Movimiento',
  empty: vacio,
  load: async (id) => {
    const d = await store.fetchOne(id)
    return {
      fecha: d.fecha,
      concepto: d.concepto,
      monto: d.monto,
      cantidad: d.cantidad,
      tipoMovimientoId: d.tipoMovimientoId,
      moneda: d.moneda,
      cotizacionAplicada: d.cotizacionAplicada,
      tipoConceptoPagoId: d.tipoConceptoPagoId,
      categoriaId: d.categoriaId,
      clienteId: d.clienteId,
      trabajoId: d.trabajoId,
      empleadoId: d.empleadoId,
      facturaId: d.facturaId,
      rowVersion: d.rowVersion,
    }
  },
  create: (dto) => store.create(dto),
  update: (id, dto) => store.update(id, dto, dto.rowVersion ?? ''),
  onSaved: () => table.reload(),
})

const monedaOptions = computed<{ label: string; value: Moneda }[]>(() => [
  { label: t('Movimientos.Moneda.Ars'), value: 'Ars' },
  { label: t('Movimientos.Moneda.Usd'), value: 'Usd' },
])

/** The rate belongs to a foreign-currency amount and is refused on a peso one. */
const pideCotizacion = computed(() => drawer.model.value.moneda === 'Usd')

const resumen = computed(() => table.summary.value)

const filtrosActivos = computed(() =>
  Boolean(
    table.filter.value.concepto ||
    table.filter.value.tipoMovimientoId ||
    table.filter.value.categoriaId ||
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

useShortcuts({ 'ctrl+n': () => drawer.openCreate() })

onMounted(() => {
  table.start()
  void cargarSelectores()
})
</script>

<template>
  <section class="flex h-full flex-col gap-4 p-6">
    <PageHeader :title="$t('Menu.Movimientos')" :subtitle="$t('Movimientos.Subtitle')">
      <template #actions>
        <Button @click="drawer.openCreate()">
          <AppIcon name="plus" :size="16" />
          {{ $t('General.New') }}
        </Button>
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
        <span class="text-xs text-muted-foreground">{{ $t('Movimientos.Desde') }}</span>
        <DateInput v-model="table.filter.value.fechaDesde" />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('Movimientos.Hasta') }}</span>
        <DateInput v-model="table.filter.value.fechaHasta" />
      </label>
    </FilterBar>

    <DataGrid
      :table="table"
      empty-key="Movimientos.Empty"
      class="flex-1"
      @row-edit="(row) => drawer.openEdit(row.id)"
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
          <MoneyText :value="data.esIngreso ? data.total : `-${data.total}`" colored />
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
            @click="drawer.openEdit(data.id)"
          >
            <AppIcon name="pencil" :size="14" />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            :disabled="data.bloqueadoPorLiquidacion"
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

    <CrudDrawer :drawer="drawer" title-key="Entity.Movimiento">
      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Movimientos.Fecha') }}</span>
        <DateInput
          v-model="drawer.model.value.fecha"
          instant
          show-time
          :invalid="Boolean(drawer.fieldErrors.value.fecha)"
        />
        <FieldError id="mov-fecha-error" :message="drawer.fieldErrors.value.fecha" />
      </label>

      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Movimientos.Concepto') }}</span>
        <InputText
          v-model="drawer.model.value.concepto"
          :invalid="Boolean(drawer.fieldErrors.value.concepto)"
          aria-describedby="mov-concepto-error"
        />
        <FieldError id="mov-concepto-error" :message="drawer.fieldErrors.value.concepto" />
      </label>

      <div class="grid grid-cols-2 gap-3">
        <label class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('Movimientos.Monto') }}</span>
          <MoneyInput
            v-model="drawer.model.value.monto"
            :min="0"
            :invalid="Boolean(drawer.fieldErrors.value.monto)"
          />
          <FieldError id="mov-monto-error" :message="drawer.fieldErrors.value.monto" />
        </label>

        <label class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('Movimientos.Cantidad') }}</span>
          <InputNumber
            :model-value="Number(drawer.model.value.cantidad)"
            :min="0"
            :min-fraction-digits="2"
            :max-fraction-digits="4"
            :invalid="Boolean(drawer.fieldErrors.value.cantidad)"
            fluid
            input-class="tabular-nums text-right"
            @update:model-value="(value) => (drawer.model.value.cantidad = (value ?? 0).toFixed(4))"
          />
          <FieldError id="mov-cantidad-error" :message="drawer.fieldErrors.value.cantidad" />
        </label>
      </div>

      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Movimientos.Tipo') }}</span>
        <Select
          v-model="drawer.model.value.tipoMovimientoId"
          :options="tipos"
          option-label="label"
          option-value="id"
          :invalid="Boolean(drawer.fieldErrors.value.tipoMovimientoId)"
        />
        <FieldError id="mov-tipo-error" :message="drawer.fieldErrors.value.tipoMovimientoId" />
      </label>

      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Movimientos.Categoria') }}</span>
        <Select
          v-model="drawer.model.value.categoriaId"
          :options="categorias"
          option-label="label"
          option-value="id"
          filter
          :invalid="Boolean(drawer.fieldErrors.value.categoriaId)"
        />
        <FieldError id="mov-categoria-error" :message="drawer.fieldErrors.value.categoriaId" />
      </label>

      <div class="grid grid-cols-2 gap-3">
        <label class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('Movimientos.Moneda.Label') }}</span>
          <Select
            v-model="drawer.model.value.moneda"
            :options="monedaOptions"
            option-label="label"
            option-value="value"
            @change="!pideCotizacion && (drawer.model.value.cotizacionAplicada = null)"
          />
        </label>

        <label v-if="pideCotizacion" class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('Movimientos.Cotizacion') }}</span>
          <MoneyInput
            :model-value="drawer.model.value.cotizacionAplicada ?? '0.0000'"
            :min="0"
            :invalid="Boolean(drawer.fieldErrors.value.cotizacionAplicada)"
            @update:model-value="(value) => (drawer.model.value.cotizacionAplicada = value)"
          />
          <FieldError
            id="mov-cotizacion-error"
            :message="drawer.fieldErrors.value.cotizacionAplicada"
          />
        </label>
      </div>
    </CrudDrawer>
  </section>
</template>
