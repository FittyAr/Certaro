<script setup lang="ts">
import Column from 'primevue/column'
import DataTable from 'primevue/datatable'
import DateText from '@/components/domain/DateText.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import type { MovimientoListItem } from '@/stores/useMovimientosStore'

defineProps<{
  items: MovimientoListItem[]
  totalIngresos: string
  totalGastos: string
  balanceNeto: string
}>()

const emit = defineEmits<{
  (e: 'registrar-movimiento'): void
}>()
</script>

<template>
  <div class="space-y-4">
    <!-- Financial KPI Cards -->
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

    <div class="flex items-center justify-between">
      <span class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
        Movimientos Imputados
      </span>
      <Button size="sm" @click="emit('registrar-movimiento')">
        <AppIcon name="plus" :size="14" />
        {{ $t('Movimientos.RegistrarGasto') }}
      </Button>
    </div>

    <div v-if="items.length === 0" class="rounded-lg border border-dashed border-border p-8 text-center text-sm text-muted-foreground">
      {{ $t('Movimientos.Empty') }}
    </div>

    <DataTable
      v-else
      :value="items"
      data-key="id"
      size="small"
      class="text-sm"
      paginator
      :rows="20"
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
          />
        </template>
      </Column>
    </DataTable>
  </div>
</template>
