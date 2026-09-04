<script setup lang="ts">
import Column from 'primevue/column'
import DataGrid from '@/components/domain/DataGrid.vue'
import DateText from '@/components/domain/DateText.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import StatePill from '@/components/domain/StatePill.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import type { useServerTable } from '@/composables/useServerTable'
import type { FacturaFiltro, FacturaListItem, EstadoFactura } from '@/stores/useFacturasStore'

defineProps<{
  table: ReturnType<typeof useServerTable<FacturaFiltro, FacturaListItem>>
}>()

const emit = defineEmits<{
  (e: 'row-edit', id: string): void
  (e: 'cambiar-estado', row: FacturaListItem, destino: EstadoFactura): void
  (e: 'abrir-pagos', row: FacturaListItem): void
  (e: 'delete', row: FacturaListItem): void
}>()
</script>

<template>
  <DataGrid
    :table="table"
    empty-key="Facturas.Empty"
    class="flex-1"
    @row-edit="(row: any) => emit('row-edit', row.id)"
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
          variant="outline"
          size="sm"
          class="h-7 gap-1 px-2 border-primary/40 text-primary hover:bg-primary/10 text-xs font-medium"
          :title="$t('Actions.Factura.Emitida')"
          @click="emit('cambiar-estado', data, 'Emitida')"
        >
          <AppIcon name="send" :size="12" />
          <span>{{ $t('Actions.Factura.Emitida') || 'Emitir' }}</span>
        </Button>
        <Button
          variant="ghost"
          size="sm"
          :disabled="!data.admitePagos"
          :title="data.admitePagos ? $t('Facturas.Pagos') : (data.estado === 'Borrador' ? 'Debe emitir la factura para registrar pagos' : $t('Facturas.Pagos'))"
          @click="emit('abrir-pagos', data)"
        >
          <AppIcon name="wallet" :size="14" />
        </Button>
        <Button
          variant="ghost"
          size="sm"
          :aria-label="$t('General.Edit')"
          @click="emit('row-edit', data.id)"
        >
          <AppIcon name="pencil" :size="14" />
        </Button>
        <Button
          variant="ghost"
          size="sm"
          :aria-label="$t('General.Delete')"
          @click="emit('delete', data)"
        >
          <AppIcon name="trash-2" :size="14" />
        </Button>
      </div>
    </template>
  </DataGrid>
</template>
