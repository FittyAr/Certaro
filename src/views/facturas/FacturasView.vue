<script setup lang="ts">
import Checkbox from 'primevue/checkbox'
import Column from 'primevue/column'
import DataTable from 'primevue/datatable'
import Dialog from 'primevue/dialog'
import InputText from 'primevue/inputtext'
import MultiSelect from 'primevue/multiselect'
import Select from 'primevue/select'
import Textarea from 'primevue/textarea'
import ToggleSwitch from 'primevue/toggleswitch'
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute } from 'vue-router'

import CrudDrawer from '@/components/domain/CrudDrawer.vue'
import DataGrid from '@/components/domain/DataGrid.vue'
import DateInput from '@/components/domain/DateInput.vue'
import DateText from '@/components/domain/DateText.vue'
import FieldError from '@/components/domain/FieldError.vue'
import Divider from 'primevue/divider'
import FilterBar from '@/components/domain/FilterBar.vue'
import MoneyInput from '@/components/domain/MoneyInput.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
import StatePill from '@/components/domain/StatePill.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import HelpButton from '@/components/ui/HelpButton.vue'
import { Button } from '@/components/ui/button'
import { useApiError } from '@/composables/useApiError'
import { useConfirmDelete } from '@/composables/useConfirmDelete'
import { useCrudDrawer } from '@/composables/useCrudDrawer'
import { useServerTable } from '@/composables/useServerTable'
import { useShortcuts } from '@/composables/useShortcuts'
import type { LookupItem } from '@/stores/useCatalogStore'
import { useClientesStore } from '@/stores/useClientesStore'
import { useMovimientosStore } from '@/stores/useMovimientosStore'
import {
  MEDIOS_PAGO,
  useFacturasStore,
  type EstadoFactura,
  type FacturaDetalle,
  type FacturaFiltro,
  type FacturaInput,
  type FacturaListItem,
  type PagoFacturaInput,
} from '@/stores/useFacturasStore'

/**
 * Invoices and their payments. See `docs/09-modulos-funcionales.md` §3.8.
 *
 * The state is never typed in: it follows the balance. Registering a payment answers with the
 * whole invoice, which is what keeps the totals on screen from drifting from the database.
 */

const { t } = useI18n()
const route = useRoute()
const { confirmDelete } = useConfirmDelete()
const { fieldErrors, notify } = useApiError()
const store = useFacturasStore()
const clientes = useClientesStore()
const movimientosStore = useMovimientosStore()

const registrarEnCaja = ref(true)

const table = useServerTable<FacturaFiltro, FacturaListItem>({
  key: 'facturas',
  initialFilter: { texto: '', estados: [] },
  fetch: (query) => store.fetchPaged(query),
  defaultSort: { field: 'fecha', dir: 'Desc' },
})

const opcionesCliente = ref<LookupItem[]>([])

const estadoOptions = computed<{ label: string; value: EstadoFactura }[]>(() =>
  (['Borrador', 'Emitida', 'PagadaParcial', 'Pagada', 'Vencida', 'Anulada'] as const).map(
    (value) => ({ label: t(`State.Factura.${value}`), value }),
  ),
)

const medioPagoOptions = computed(() =>
  MEDIOS_PAGO.map((value) => ({ label: t(`MedioPago.${value}`), value })),
)

function hoy(): string {
  return new Date().toISOString().slice(0, 10)
}

type Model = FacturaInput & { rowVersion?: string }

const drawer = useCrudDrawer<Model>({
  entityKey: 'Entity.Factura',
  empty: () => ({
    numero: '',
    fecha: hoy(),
    fechaVencimiento: null,
    clienteId: '',
    subtotal: '0.0000',
    iva: '0.0000',
    total: '0.0000',
    observaciones: null,
  }),
  load: async (id) => {
    const d = await store.fetchOne(id)
    return {
      numero: d.numero,
      fecha: d.fecha,
      fechaVencimiento: d.fechaVencimiento,
      clienteId: d.clienteId,
      subtotal: d.subtotal,
      iva: d.iva,
      total: d.total,
      observaciones: d.observaciones,
      rowVersion: d.audit.rowVersion,
    }
  },
  create: (dto) => store.create(dto),
  update: (id, dto) => store.update(id, dto, dto.rowVersion ?? ''),
  onSaved: () => table.reload(),
})

/**
 * The total is `subtotal + iva` and the backend recomputes it anyway; showing the sum here keeps
 * the form honest instead of letting the user type a third, inconsistent number.
 */
function recalcularTotal(): void {
  const subtotal = Number(drawer.model.value.subtotal)
  const iva = Number(drawer.model.value.iva)
  drawer.model.value.total = (subtotal + iva).toFixed(4)
}

// ------------------------------------------------------------------- payments

const pagosVisible = ref(false)
const factura = ref<FacturaDetalle | null>(null)
const nuevoPago = ref<PagoFacturaInput>({
  facturaId: '',
  fecha: hoy(),
  monto: '0.0000',
  medioPago: 'Efectivo',
})
const pagoErrores = ref<Record<string, string>>({})

async function abrirPagos(row: FacturaListItem): Promise<void> {
  try {
    factura.value = await store.fetchOne(row.id)
    // The default is what is still owed: the usual payment cancels the balance in full.
    nuevoPago.value = {
      facturaId: row.id,
      fecha: hoy(),
      monto: factura.value.saldo,
      medioPago: 'Efectivo',
    }
    pagoErrores.value = {}
    pagosVisible.value = true
  } catch (e) {
    notify(e)
  }
}

async function registrarPago(): Promise<void> {
  pagoErrores.value = {}
  try {
    const pagoMonto = nuevoPago.value.monto
    const pagoMedio = nuevoPago.value.medioPago
    const pagoFecha = nuevoPago.value.fecha
    factura.value = await store.crearPago(nuevoPago.value)

    if (registrarEnCaja.value && factura.value) {
      try {
        await movimientosStore.create({
          fecha: new Date(pagoFecha).toISOString(),
          concepto: `Cobranza Factura ${factura.value.numero} (${pagoMedio})`,
          monto: pagoMonto,
          cantidad: '1.0000',
          tipoMovimientoId: '00000000-0000-0000-0000-000000000001',
          moneda: 'Ars',
          cotizacionAplicada: null,
          tipoConceptoPagoId: null,
          categoriaId: null,
          clienteId: factura.value.clienteId,
          trabajoId: null,
          empleadoId: null,
          facturaId: factura.value.id,
        })
      } catch (err) {
        console.warn('No se pudo registrar automáticamente el ingreso en caja:', err)
      }
    }

    nuevoPago.value = {
      facturaId: factura.value.id,
      fecha: hoy(),
      monto: factura.value.saldo,
      medioPago: 'Efectivo',
    }
    table.reload()
  } catch (e) {
    const error = notify(e)
    if (error.code === 'VALIDATION') pagoErrores.value = fieldErrors(error)
  }
}

async function borrarPago(id: string, rowVersion: string): Promise<void> {
  try {
    factura.value = await store.borrarPago(id, rowVersion)
    table.reload()
  } catch (e) {
    notify(e)
  }
}

// --------------------------------------------------------------------- states

async function cambiarEstado(row: FacturaListItem, destino: EstadoFactura): Promise<void> {
  try {
    await store.transition(row.id, destino, row.rowVersion)
    table.reload()
  } catch (e) {
    notify(e)
  }
}

const filtrosActivos = computed(() =>
  Boolean(
    table.filter.value.texto ||
    table.filter.value.clienteId ||
    table.filter.value.estados?.length ||
    table.filter.value.fechaDesde ||
    table.filter.value.fechaHasta ||
    table.filter.value.soloImpagas ||
    table.filter.value.soloVencidas,
  ),
)

function onDelete(row: FacturaListItem): void {
  confirmDelete({
    entityKey: 'Entity.Factura',
    label: row.numero,
    action: () => store.remove(row.id, row.rowVersion),
    onDone: () => table.reload(),
  })
}

useShortcuts({ 'ctrl+n': () => drawer.openCreate() })

onMounted(async () => {
  table.start()
  try {
    opcionesCliente.value = await clientes.lookup(undefined, 200)
  } catch (e) {
    notify(e)
  }
  if (route.query.certificadoId) {
    drawer.openCreate()
    if (route.query.clienteId) drawer.model.value.clienteId = String(route.query.clienteId)
    if (route.query.subtotal) drawer.model.value.subtotal = String(route.query.subtotal)
    if (route.query.iva) drawer.model.value.iva = String(route.query.iva)
    if (route.query.total) drawer.model.value.total = String(route.query.total)
    if (route.query.observaciones) drawer.model.value.observaciones = String(route.query.observaciones)
  }
})
</script>

<template>
  <section class="flex h-full flex-col gap-4 p-6">
    <PageHeader :title="$t('Menu.Facturas')" :subtitle="$t('Facturas.Subtitle')">
      <template #actions>
        <Button @click="drawer.openCreate()">
          <AppIcon name="plus" :size="16" />
          {{ $t('General.New') }}
        </Button>
        <HelpButton topic-id="facturas-overview" title="Ayuda sobre Facturas y Cobranzas" />
      </template>
    </PageHeader>

    <FilterBar :active="filtrosActivos" @clear="table.resetFilter()">
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('General.Search') }}</span>
        <InputText v-model="table.filter.value.texto" :placeholder="$t('Facturas.BuscarHint')" />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('Facturas.Cliente') }}</span>
        <Select
          v-model="table.filter.value.clienteId"
          :options="opcionesCliente"
          option-label="label"
          option-value="id"
          filter
          show-clear
          :placeholder="$t('General.All')"
        />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('Facturas.Estado') }}</span>
        <MultiSelect
          v-model="table.filter.value.estados"
          :options="estadoOptions"
          option-label="label"
          option-value="value"
          :placeholder="$t('General.All')"
        />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('Facturas.Desde') }}</span>
        <DateInput v-model="table.filter.value.fechaDesde" />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('Facturas.Hasta') }}</span>
        <DateInput v-model="table.filter.value.fechaHasta" />
      </label>
      <label class="flex items-center gap-2 self-end pb-2 cursor-pointer select-none">
        <ToggleSwitch v-model="table.filter.value.soloImpagas" />
        <span class="text-xs font-medium text-foreground/90">{{ $t('Facturas.SoloImpagas') }}</span>
      </label>
      <label class="flex items-center gap-2 self-end pb-2 cursor-pointer select-none">
        <ToggleSwitch v-model="table.filter.value.soloVencidas" />
        <span class="text-xs font-medium text-foreground/90">{{ $t('Facturas.SoloVencidas') }}</span>
      </label>
    </FilterBar>

    <Divider />

    <DataGrid
      :table="table"
      empty-key="Facturas.Empty"
      class="flex-1"
      @row-edit="(row: any) => drawer.openEdit(row.id)"
    >
      <Column field="fecha" :header="$t('Facturas.Fecha')" sortable>
        <template #body="{ data }"><DateText :value="data.fecha" /></template>
      </Column>
      <Column field="numero" :header="$t('Facturas.Numero')" sortable />
      <Column field="clienteNombre" :header="$t('Facturas.Cliente')" sortable />
      <Column field="estado" :header="$t('Facturas.Estado')" sortable>
        <template #body="{ data }"><StatePill entity="Factura" :value="data.estado" /></template>
      </Column>
      <Column field="total" :header="$t('Facturas.Total')" sortable>
        <template #body="{ data }"><MoneyText :value="data.total" /></template>
      </Column>
      <Column field="pagado" :header="$t('Facturas.Pagado')" sortable>
        <template #body="{ data }"><MoneyText :value="data.pagado" /></template>
      </Column>
      <Column field="saldo" :header="$t('Facturas.Saldo')" sortable>
        <template #body="{ data }"><MoneyText :value="data.saldo" colored /></template>
      </Column>
      <Column field="diasMora" :header="$t('Facturas.Mora')">
        <template #body="{ data }">
          <span v-if="data.diasMora > 0" class="tabular-nums text-destructive">
            {{ $t('Facturas.DiasMora', { count: data.diasMora }) }}
          </span>
          <span v-else>—</span>
        </template>
      </Column>

      <template #actions="{ data }">
        <div class="flex gap-1">
          <Button
            v-if="data.estado === 'Borrador'"
            variant="ghost"
            size="sm"
            :title="$t('Actions.Factura.Emitida')"
            @click="cambiarEstado(data, 'Emitida')"
          >
            <AppIcon name="send" :size="14" />
          </Button>
          <Button variant="ghost" size="sm" :title="$t('Facturas.Pagos')" @click="abrirPagos(data)">
            <AppIcon name="wallet" :size="14" />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            :aria-label="$t('General.Edit')"
            @click="drawer.openEdit(data.id)"
          >
            <AppIcon name="pencil" :size="14" />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            :aria-label="$t('General.Delete')"
            @click="onDelete(data)"
          >
            <AppIcon name="trash-2" :size="14" />
          </Button>
        </div>
      </template>
    </DataGrid>

    <CrudDrawer :drawer="drawer" title-key="Entity.Factura">
      <div class="grid grid-cols-2 gap-3">
        <label class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('Facturas.Numero') }}</span>
          <InputText
            v-model="drawer.model.value.numero"
            :invalid="Boolean(drawer.fieldErrors.value.numero)"
            aria-describedby="fac-numero-error"
          />
          <FieldError id="fac-numero-error" :message="drawer.fieldErrors.value.numero" />
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('Facturas.Fecha') }}</span>
          <DateInput
            v-model="drawer.model.value.fecha"
            :invalid="Boolean(drawer.fieldErrors.value.fecha)"
          />
          <FieldError id="fac-fecha-error" :message="drawer.fieldErrors.value.fecha" />
        </label>
      </div>

      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Facturas.Cliente') }}</span>
        <Select
          v-model="drawer.model.value.clienteId"
          :options="opcionesCliente"
          option-label="label"
          option-value="id"
          filter
          :invalid="Boolean(drawer.fieldErrors.value.clienteId)"
        />
        <FieldError id="fac-cliente-error" :message="drawer.fieldErrors.value.clienteId" />
      </label>

      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Facturas.Vencimiento') }}</span>
        <DateInput
          v-model="drawer.model.value.fechaVencimiento"
          :invalid="Boolean(drawer.fieldErrors.value.fechaVencimiento)"
        />
        <FieldError id="fac-venc-error" :message="drawer.fieldErrors.value.fechaVencimiento" />
      </label>

      <div class="grid grid-cols-3 gap-3">
        <label class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('Facturas.Subtotal') }}</span>
          <MoneyInput
            v-model="drawer.model.value.subtotal"
            :min="0"
            :invalid="Boolean(drawer.fieldErrors.value.subtotal)"
            @update:model-value="recalcularTotal()"
          />
          <FieldError id="fac-subtotal-error" :message="drawer.fieldErrors.value.subtotal" />
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('Facturas.Iva') }}</span>
          <MoneyInput
            v-model="drawer.model.value.iva"
            :min="0"
            :invalid="Boolean(drawer.fieldErrors.value.iva)"
            @update:model-value="recalcularTotal()"
          />
          <FieldError id="fac-iva-error" :message="drawer.fieldErrors.value.iva" />
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('Facturas.Total') }}</span>
          <MoneyInput v-model="drawer.model.value.total" disabled />
        </label>
      </div>

      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Facturas.Observaciones') }}</span>
        <Textarea v-model="drawer.model.value.observaciones" rows="3" auto-resize />
      </label>
    </CrudDrawer>

    <Dialog
      v-model:visible="pagosVisible"
      modal
      :header="$t('Facturas.Pagos')"
      class="w-[42rem]"
      :dismissable-mask="true"
    >
      <div v-if="factura" class="flex flex-col gap-4">
        <div class="flex flex-wrap items-center gap-6 text-sm">
          <span class="font-medium">{{ factura.numero }}</span>
          <span>{{ factura.clienteNombre }}</span>
          <StatePill entity="Factura" :value="factura.estado.actual" />
          <span class="flex items-center gap-2">
            <span class="text-muted-foreground">{{ $t('Facturas.Total') }}</span>
            <MoneyText :value="factura.total" />
          </span>
          <span class="flex items-center gap-2">
            <span class="text-muted-foreground">{{ $t('Facturas.Saldo') }}</span>
            <MoneyText :value="factura.saldo" colored />
          </span>
        </div>

        <DataTable :value="factura.pagos" size="small" class="text-sm">
          <Column field="fecha" :header="$t('Facturas.Fecha')">
            <template #body="{ data }"><DateText :value="data.fecha" /></template>
          </Column>
          <Column field="medioPago" :header="$t('Facturas.MedioPago')" />
          <Column field="monto" :header="$t('Facturas.Monto')">
            <template #body="{ data }"><MoneyText :value="data.monto" /></template>
          </Column>
          <Column>
            <template #body="{ data }">
              <Button variant="ghost" size="sm" @click="borrarPago(data.id, data.rowVersion)">
                <AppIcon name="trash-2" :size="14" />
              </Button>
            </template>
          </Column>
          <template #empty>
            <span class="text-muted-foreground">{{ $t('Facturas.SinPagos') }}</span>
          </template>
        </DataTable>

        <!-- Disabled rather than hidden: a draft or an annulled invoice shows why it takes none. -->
        <div
          v-if="factura.admitePagos"
          class="grid grid-cols-4 items-end gap-3 border-t border-border pt-3"
        >
          <label class="flex flex-col gap-1">
            <span class="text-xs text-muted-foreground">{{ $t('Facturas.Fecha') }}</span>
            <DateInput v-model="nuevoPago.fecha" :invalid="Boolean(pagoErrores.fecha)" />
            <FieldError id="pago-fecha-error" :message="pagoErrores.fecha" />
          </label>
          <label class="flex flex-col gap-1">
            <span class="text-xs text-muted-foreground">{{ $t('Facturas.Monto') }}</span>
            <MoneyInput v-model="nuevoPago.monto" :min="0" :invalid="Boolean(pagoErrores.monto)" />
            <FieldError id="pago-monto-error" :message="pagoErrores.monto" />
          </label>
          <label class="flex flex-col gap-1">
            <span class="text-xs text-muted-foreground">{{ $t('Facturas.MedioPago') }}</span>
            <Select
              v-model="nuevoPago.medioPago"
              :options="medioPagoOptions"
              option-label="label"
              option-value="value"
              editable
            />
          </label>
          <div class="col-span-4 flex items-center justify-between pt-2">
            <label class="flex items-center gap-2 text-xs text-muted-foreground cursor-pointer select-none">
              <Checkbox v-model="registrarEnCaja" :binary="true" />
              <span>{{ $t('Facturas.RegistrarEnCaja') }}</span>
            </label>
            <Button @click="registrarPago()">
              <AppIcon name="plus" :size="16" />
              {{ $t('Facturas.RegistrarPago') }}
            </Button>
          </div>
        </div>
        <p v-else class="text-xs text-muted-foreground">{{ $t('Facturas.NoAdmitePagos') }}</p>
      </div>
    </Dialog>
  </section>
</template>
