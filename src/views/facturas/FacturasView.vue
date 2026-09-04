<script setup lang="ts">
import Divider from 'primevue/divider'
import InputText from 'primevue/inputtext'
import MultiSelect from 'primevue/multiselect'
import Select from 'primevue/select'
import ToggleSwitch from 'primevue/toggleswitch'
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute } from 'vue-router'

import DateInput from '@/components/domain/DateInput.vue'
import FilterBar from '@/components/domain/FilterBar.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
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
import {
  useFacturasStore,
  type EstadoFactura,
  type FacturaFiltro,
  type FacturaInput,
  type FacturaListItem,
} from '@/stores/useFacturasStore'
import FacturaPagosModal from './components/FacturaPagosModal.vue'
import FacturaFormDrawer from './components/FacturaFormDrawer.vue'
import FacturasTable from './components/FacturasTable.vue'

const { t } = useI18n()
const route = useRoute()
const { confirmDelete } = useConfirmDelete()
const { notify } = useApiError()
const store = useFacturasStore()
const clientes = useClientesStore()

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
  create: async (dto) => {
    const created = await store.create(dto)
    if (route.query.proyectoId || route.query.trabajoId) {
      try {
        localStorage.setItem(
          `certaro:factura-obra:${created.id}`,
          JSON.stringify({
            proyectoId: route.query.proyectoId ? String(route.query.proyectoId) : null,
            trabajoId: route.query.trabajoId ? String(route.query.trabajoId) : null,
          }),
        )
      } catch {
        // ignore
      }
    }
    return created
  },
  update: (id, dto) => store.update(id, dto, dto.rowVersion ?? ''),
  onSaved: () => table.reload(),
})

// ------------------------------------------------------------------- payments
const pagosVisible = ref(false)
const selectedFacturaId = ref<string | null>(null)

function abrirPagos(row: { id: string }): void {
  selectedFacturaId.value = row.id
  pagosVisible.value = true
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
  if (route.query.id) {
    const targetId = String(route.query.id)
    store
      .fetchOne(targetId)
      .then((f) => {
        void abrirPagos(f)
      })
      .catch(() => {
        table.filter.value.texto = targetId
        void table.reload()
      })
  } else if (route.query.certificadoId) {
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

    <FacturasTable
      :table="table"
      @row-edit="(id) => drawer.openEdit(id)"
      @cambiar-estado="cambiarEstado"
      @abrir-pagos="abrirPagos"
      @delete="onDelete"
    />

    <FacturaFormDrawer
      :drawer="drawer"
      :opciones-cliente="opcionesCliente"
    />

    <!-- Modal de pagos separado -->
    <FacturaPagosModal
      v-model:visible="pagosVisible"
      :factura-id="selectedFacturaId"
      @pago-modificado="table.reload()"
    />
  </section>
</template>
