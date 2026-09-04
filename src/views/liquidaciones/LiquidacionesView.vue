<script setup lang="ts">
import Checkbox from 'primevue/checkbox'
import Column from 'primevue/column'
import Select from 'primevue/select'
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'

import DataGrid from '@/components/domain/DataGrid.vue'
import DateInput from '@/components/domain/DateInput.vue'
import DateText from '@/components/domain/DateText.vue'
import FilterBar from '@/components/domain/FilterBar.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import HelpButton from '@/components/ui/HelpButton.vue'
import { Button } from '@/components/ui/button'
import { useConfirmDelete } from '@/composables/useConfirmDelete'
import { useExport } from '@/composables/useExport'
import { useServerTable } from '@/composables/useServerTable'
import { useShortcuts } from '@/composables/useShortcuts'
import { useEmpleadosStore } from '@/stores/useEmpleadosStore'
import { useReportesStore } from '@/stores/useReportesStore'
import {
  useLiquidacionesStore,
  type LiquidacionFiltro,
  type LiquidacionListItem,
} from '@/stores/useLiquidacionesStore'
import LiquidacionWizardModal from './components/LiquidacionWizardModal.vue'

/**
 * Settlements list and wizard launcher. See `docs/09-modulos-funcionales.md` §3.11.
 */

const router = useRouter()
const { confirmDelete } = useConfirmDelete()
const store = useLiquidacionesStore()
const empleados = useEmpleadosStore()
const reportes = useReportesStore()
const { exportar } = useExport()

const wizardOpen = ref(false)

const table = useServerTable<LiquidacionFiltro, LiquidacionListItem>({
  key: 'liquidaciones',
  initialFilter: {},
  fetch: (query) => store.fetchPaged(query),
  defaultSort: { field: 'fechaInicio', dir: 'Desc' },
})

const filtrosActivos = computed(
  () =>
    Boolean(table.filter.value.empleadoId) ||
    Boolean(table.filter.value.fechaDesde) ||
    Boolean(table.filter.value.fechaHasta) ||
    table.filter.value.soloSinPdf === true,
)

function onDelete(row: LiquidacionListItem): void {
  confirmDelete({
    entityKey: 'Entity.Liquidacion',
    label: row.empleadoNombre,
    action: () => store.remove(row.id, row.rowVersion),
    onDone: () => table.reload(),
  })
}

function abrirDetalle(row: LiquidacionListItem): void {
  void router.push({ name: 'liquidacion-detalle', params: { liquidacionId: row.id } })
}

function exportarPdf(row: LiquidacionListItem): void {
  void exportar({
    reporte: 'liquidacion',
    formato: 'Pdf',
    detalle: row.empleadoNombre,
    run: (destino) => reportes.exportLiquidacion(row.id, destino),
  })
}

useShortcuts({ 'ctrl+n': () => (wizardOpen.value = true) })

onMounted(() => {
  table.start()
  void empleados.fetchLookup(true)
})
</script>

<template>
  <section class="flex h-full flex-col gap-4 p-6">
    <PageHeader :title="$t('Menu.Liquidaciones')" :subtitle="$t('Liquidaciones.Subtitle')">
      <template #actions>
        <Button @click="wizardOpen = true">
          <AppIcon name="plus" :size="16" />
          {{ $t('Liquidaciones.Nueva') }}
        </Button>
        <HelpButton topic-id="liquidaciones-overview" title="Ayuda sobre Liquidaciones y Sueldos" />
      </template>
    </PageHeader>

    <FilterBar :active="filtrosActivos" @clear="table.resetFilter()">
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('Empleados.Nombre') }}</span>
        <Select
          v-model="table.filter.value.empleadoId"
          :options="empleados.opciones"
          option-label="label"
          option-value="id"
          :placeholder="$t('General.All')"
          show-clear
        />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('General.From') }}</span>
        <DateInput v-model="table.filter.value.fechaDesde" />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('General.To') }}</span>
        <DateInput v-model="table.filter.value.fechaHasta" />
      </label>
      <label class="flex items-center gap-2 self-end pb-2">
        <Checkbox v-model="table.filter.value.soloSinPdf" binary />
        <span class="text-xs text-muted-foreground">{{ $t('Liquidaciones.SoloSinPdf') }}</span>
      </label>
    </FilterBar>

    <DataGrid
      :table="table"
      empty-key="Liquidaciones.Empty"
      class="flex-1"
      @row-edit="(row: any) => abrirDetalle(row)"
    >
      <Column field="empleadoNombre" :header="$t('Empleados.Nombre')" sortable />
      <Column field="fechaInicio" :header="$t('Liquidaciones.Periodo')" sortable>
        <template #body="{ data }">
          <DateText :value="data.fechaInicio" /> – <DateText :value="data.fechaFin" />
        </template>
      </Column>
      <Column field="diasTrabajados" :header="$t('Liquidaciones.Dias')" sortable>
        <template #body="{ data }">
          <span class="tabular-nums">{{ data.diasTrabajados }}</span>
        </template>
      </Column>
      <Column field="totalBruto" :header="$t('Liquidaciones.TotalBruto')" sortable>
        <template #body="{ data }"><MoneyText :value="data.totalBruto" /></template>
      </Column>
      <Column field="totalAdelantos" :header="$t('Liquidaciones.TotalAdelantos')">
        <template #body="{ data }"><MoneyText :value="data.totalAdelantos" /></template>
      </Column>
      <Column field="totalNeto" :header="$t('Liquidaciones.TotalNeto')" sortable>
        <template #body="{ data }"><MoneyText :value="data.totalNeto" /></template>
      </Column>

      <template #actions="{ data }">
        <div class="flex gap-1">
          <Button variant="ghost" size="sm" @click="abrirDetalle(data)">
            <AppIcon name="eye" :size="14" />
          </Button>
          <Button variant="ghost" size="sm" :title="$t('Export.Tipo.Pdf')" @click="exportarPdf(data)">
            <AppIcon name="download" :size="14" />
          </Button>
          <Button variant="ghost" size="sm" @click="onDelete(data)">
            <AppIcon name="trash-2" :size="14" />
          </Button>
        </div>
      </template>
    </DataGrid>

    <LiquidacionWizardModal
      v-model:visible="wizardOpen"
      @saved="table.reload()"
    />
  </section>
</template>
