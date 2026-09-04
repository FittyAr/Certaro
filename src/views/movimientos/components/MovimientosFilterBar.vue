<script setup lang="ts">
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import DateInput from '@/components/domain/DateInput.vue'
import FilterBar from '@/components/domain/FilterBar.vue'
import type { ServerTable } from '@/composables/useServerTable'
import type { LookupItem } from '@/stores/useCatalogStore'
import type {
  Moneda,
  MovimientoFiltro,
  MovimientoListItem,
} from '@/stores/useMovimientosStore'

const props = defineProps<{
  table: ServerTable<MovimientoFiltro, MovimientoListItem>
  tipos: LookupItem[]
  categorias: LookupItem[]
  opcionesCliente: LookupItem[]
  opcionesProyecto: LookupItem[]
}>()

const { t } = useI18n()

const monedaFilterOptions = computed<{ label: string; value: Moneda | undefined }[]>(() => [
  { label: t('General.All'), value: undefined },
  { label: t('Movimientos.Moneda.Ars'), value: 'Ars' },
  { label: t('Movimientos.Moneda.Usd'), value: 'Usd' },
])

const filtrosActivos = computed(() =>
  Boolean(
    props.table.filter.value.concepto ||
    props.table.filter.value.tipoMovimientoId ||
    props.table.filter.value.categoriaId ||
    props.table.filter.value.clienteId ||
    props.table.filter.value.proyectoId ||
    props.table.filter.value.moneda ||
    props.table.filter.value.fechaDesde ||
    props.table.filter.value.fechaHasta,
  ),
)
</script>

<template>
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
</template>
