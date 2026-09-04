<script setup lang="ts">
import Column from 'primevue/column'
import InputNumber from 'primevue/inputnumber'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute } from 'vue-router'

import CrudDrawer from '@/components/domain/CrudDrawer.vue'
import DataGrid from '@/components/domain/DataGrid.vue'
import DateInput from '@/components/domain/DateInput.vue'
import DateText from '@/components/domain/DateText.vue'
import FieldError from '@/components/domain/FieldError.vue'
import ExportMenu from '@/components/domain/ExportMenu.vue'
import Divider from 'primevue/divider'
import FilterBar from '@/components/domain/FilterBar.vue'
import MoneyInput from '@/components/domain/MoneyInput.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import HelpButton from '@/components/ui/HelpButton.vue'
import { Button } from '@/components/ui/button'
import { useConfirmDelete } from '@/composables/useConfirmDelete'
import { useCrudDrawer } from '@/composables/useCrudDrawer'
import { useServerTable } from '@/composables/useServerTable'
import { useShortcuts } from '@/composables/useShortcuts'
import { useCatalogStore, type LookupItem } from '@/stores/useCatalogStore'
import { useClientesStore } from '@/stores/useClientesStore'
import { useEmpleadosStore } from '@/stores/useEmpleadosStore'
import { useMovimientosStore } from '@/stores/useMovimientosStore'
import { useProyectosStore } from '@/stores/useProyectosStore'
import { useReportesStore } from '@/stores/useReportesStore'
import { useTrabajosStore } from '@/stores/useTrabajosStore'
import type {
  Moneda,
  MovimientoFiltro,
  MovimientoInput,
  MovimientoListItem,
  MovimientoResumen,
} from '@/stores/useMovimientosStore'

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
const trabajos = useTrabajosStore()

const ADELANTO_ID = '00000000-0000-0000-0000-000000000003'

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
const opcionesTrabajo = ref<LookupItem[]>([])
const selectedProyectoId = ref<string | null>(null)

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

async function onClienteChange(): Promise<void> {
  const cId = drawer.model.value.clienteId
  selectedProyectoId.value = null
  drawer.model.value.trabajoId = null
  opcionesTrabajo.value = []
  if (cId) {
    opcionesProyecto.value = await proyectos.lookup(cId)
  } else {
    opcionesProyecto.value = await proyectos.lookup(undefined, undefined, 200)
  }
}

async function onProyectoChange(): Promise<void> {
  drawer.model.value.trabajoId = null
  if (!selectedProyectoId.value) {
    opcionesTrabajo.value = []
    return
  }
  try {
    opcionesTrabajo.value = await trabajos.lookup(selectedProyectoId.value)
    if (opcionesTrabajo.value.length > 0 && opcionesTrabajo.value[0]) {
      drawer.model.value.trabajoId = opcionesTrabajo.value[0].id
    }
    const p = await proyectos.fetchOne(selectedProyectoId.value)
    if (p?.clienteId) {
      drawer.model.value.clienteId = p.clienteId
    }
  } catch {
    opcionesTrabajo.value = []
  }
}

function vacio(): MovimientoInput & { rowVersion?: string } {
  selectedProyectoId.value = null
  opcionesTrabajo.value = []
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
    selectedProyectoId.value = null
    opcionesTrabajo.value = []
    if (d.trabajoId) {
      try {
        const trab = await trabajos.fetchOne(d.trabajoId)
        selectedProyectoId.value = trab.proyectoId
        opcionesTrabajo.value = await trabajos.lookup(trab.proyectoId)
      } catch {
        // Fallback
      }
    }
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
  create: (dto) => {
    if (
      selectedProyectoId.value &&
      !dto.trabajoId &&
      opcionesTrabajo.value.length > 0 &&
      opcionesTrabajo.value[0]
    ) {
      dto.trabajoId = opcionesTrabajo.value[0].id
    }
    return store.create(dto)
  },
  update: (id, dto) => {
    if (
      selectedProyectoId.value &&
      !dto.trabajoId &&
      opcionesTrabajo.value.length > 0 &&
      opcionesTrabajo.value[0]
    ) {
      dto.trabajoId = opcionesTrabajo.value[0].id
    }
    return store.update(id, dto, dto.rowVersion ?? '')
  },
  onSaved: () => table.reload(),
})

const esAdelanto = computed(() => drawer.model.value.tipoMovimientoId === ADELANTO_ID)

const monedaOptions = computed<{ label: string; value: Moneda }[]>(() => [
  { label: t('Movimientos.Moneda.Ars'), value: 'Ars' },
  { label: t('Movimientos.Moneda.Usd'), value: 'Usd' },
])

const monedaFilterOptions = computed<{ label: string; value: Moneda | undefined }[]>(() => [
  { label: t('General.All'), value: undefined },
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
      command: () => drawer.openEdit(row.id),
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

useShortcuts({ 'ctrl+n': () => drawer.openCreate() })

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
    drawer.openCreate()
    selectedProyectoId.value = String(route.query.proyectoId)
    await onProyectoChange()
    if (route.query.clienteId) {
      drawer.model.value.clienteId = String(route.query.clienteId)
    }
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
        <Button @click="drawer.openCreate()">
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
      @row-edit="(row: any) => drawer.openEdit(row.id)"
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
              class="rounded border border-amber-500/30 bg-amber-500/10 px-1 py-0.5 text-[10px] font-bold text-amber-600 dark:text-amber-400"
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
            @click="drawer.openEdit(data.id)"
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

      <!-- Empleado selector (Required if Adelanto, selectable anytime) -->
      <label v-if="esAdelanto || drawer.model.value.empleadoId || drawer.open.value" class="flex flex-col gap-1">
        <span class="text-sm">
          {{ $t('Movimientos.Empleado') }}
          <span v-if="esAdelanto" class="text-destructive">*</span>
        </span>
        <Select
          v-model="drawer.model.value.empleadoId"
          :options="opcionesEmpleado"
          option-label="label"
          option-value="id"
          filter
          show-clear
          :placeholder="esAdelanto ? $t('Movimientos.EmpleadoRequeridoPlaceholder') : $t('General.None')"
          :invalid="Boolean(drawer.fieldErrors.value.empleadoId)"
        />
        <FieldError id="mov-empleado-error" :message="drawer.fieldErrors.value.empleadoId" />
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

      <!-- Imputación opcional a Cliente / Proyecto / Trabajo -->
      <div class="space-y-3 rounded-md border border-border/70 bg-muted/20 p-3">
        <span class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
          {{ $t('Movimientos.ImputacionOpcional') }}
        </span>

        <label class="flex flex-col gap-1">
          <span class="text-xs text-muted-foreground">{{ $t('Movimientos.Cliente') }}</span>
          <Select
            v-model="drawer.model.value.clienteId"
            :options="opcionesCliente"
            option-label="label"
            option-value="id"
            filter
            show-clear
            :placeholder="$t('General.None')"
            @change="onClienteChange()"
          />
        </label>

        <div class="grid grid-cols-2 gap-3">
          <label class="flex flex-col gap-1">
            <span class="text-xs text-muted-foreground">{{ $t('Movimientos.Proyecto') }}</span>
            <Select
              v-model="selectedProyectoId"
              :options="opcionesProyecto"
              option-label="label"
              option-value="id"
              filter
              show-clear
              :placeholder="$t('General.None')"
              @change="onProyectoChange()"
            />
          </label>

          <label class="flex flex-col gap-1">
            <span class="text-xs text-muted-foreground">{{ $t('Movimientos.Trabajo') }}</span>
            <Select
              v-model="drawer.model.value.trabajoId"
              :options="opcionesTrabajo"
              option-label="label"
              option-value="id"
              filter
              show-clear
              :placeholder="$t('General.None')"
              :disabled="!selectedProyectoId && opcionesTrabajo.length === 0"
            />
          </label>
        </div>
        <p
          v-if="selectedProyectoId && opcionesTrabajo.length === 0"
          class="rounded-md border border-amber-500/30 bg-amber-500/10 p-2 text-xs text-amber-700 dark:text-amber-400"
        >
          {{ $t('Movimientos.ProyectoSinTrabajosAviso') || 'Este proyecto no tiene trabajos creados aún. Recuerda crear al menos un trabajo en el proyecto para que los gastos se imputen a la caja y rentabilidad de la obra.' }}
        </p>
      </div>

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
