<script setup lang="ts">
import Column from 'primevue/column'
import DataTable from 'primevue/datatable'
import DateText from '@/components/domain/DateText.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import StatePill from '@/components/domain/StatePill.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import type { CuentaCorrienteFactura } from '@/stores/useComercialStore'

defineProps<{
  facturas: CuentaCorrienteFactura[]
  claseMora: (dias: number) => string
}>()

const emit = defineEmits<{
  (e: 'ver-factura', factura: CuentaCorrienteFactura): void
  (e: 'abrir-cobro', factura: CuentaCorrienteFactura): void
}>()
</script>

<template>
  <DataTable
    :value="facturas"
    data-key="id"
    size="small"
    scrollable
    scroll-height="flex"
    class="flex-1 text-sm"
    @row-dblclick="emit('ver-factura', $event.data as CuentaCorrienteFactura)"
  >
    <template #empty>
      <p class="p-4 text-center text-sm text-muted-foreground">
        {{ $t('Comercial.CuentaCorriente.SinDeuda') }}
      </p>
    </template>
    <Column field="numero" :header="$t('Facturas.Numero')" sortable />
    <Column field="fecha" :header="$t('Facturas.Fecha')" sortable>
      <template #body="{ data }"><DateText :value="data.fecha" /></template>
    </Column>
    <Column field="fechaVencimiento" :header="$t('Facturas.Vencimiento')" sortable>
      <template #body="{ data }"><DateText :value="data.fechaVencimiento" /></template>
    </Column>
    <Column field="estado" :header="$t('Facturas.Estado')">
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
    <Column field="diasMora" :header="$t('Comercial.CuentaCorriente.DiasMora')" sortable>
      <template #body="{ data }">
        <span class="tabular-nums" :class="claseMora(data.diasMora)">{{ data.diasMora }}</span>
      </template>
    </Column>
    <Column :header="$t('General.Actions')" class="w-24 text-right">
      <template #body="{ data }">
        <div class="flex items-center justify-end gap-1">
          <Button
            v-if="Number(data.saldo) > 0"
            size="sm"
            variant="outline"
            title="Registrar Cobro"
            class="h-7 px-2 text-xs"
            @click="emit('abrir-cobro', data)"
          >
            <AppIcon name="wallet" :size="12" />
            <span class="ml-1">Cobrar</span>
          </Button>
          <Button
            variant="ghost"
            size="sm"
            :title="$t('General.View')"
            class="h-7 w-7 p-0"
            @click="emit('ver-factura', data)"
          >
            <AppIcon name="eye" :size="13" />
          </Button>
        </div>
      </template>
    </Column>
  </DataTable>
</template>
