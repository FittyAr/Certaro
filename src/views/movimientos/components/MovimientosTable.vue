<script setup lang="ts">
import Column from 'primevue/column'
import { useI18n } from 'vue-i18n'
import DataGrid from '@/components/domain/DataGrid.vue'
import DateText from '@/components/domain/DateText.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import type { ServerTable } from '@/composables/useServerTable'
import type {
  MovimientoFiltro,
  MovimientoListItem,
} from '@/stores/useMovimientosStore'

defineProps<{
  table: ServerTable<MovimientoFiltro, MovimientoListItem>
}>()

const emit = defineEmits<{
  (e: 'edit', id: string): void
  (e: 'delete', row: MovimientoListItem): void
}>()

const { t } = useI18n()

function movimientoContextMenu(row: MovimientoListItem) {
  return [
    {
      label: t('General.Edit'),
      icon: 'pi pi-pencil',
      disabled: row.bloqueadoPorLiquidacion,
      command: () => emit('edit', row.id),
    },
    { separator: true },
    {
      label: t('General.Delete'),
      icon: 'pi pi-trash',
      disabled: row.bloqueadoPorLiquidacion,
      command: () => emit('delete', row),
    },
  ]
}
</script>

<template>
  <DataGrid
    :table="table"
    empty-key="Movimientos.Empty"
    class="flex-1"
    :context-menu-items="movimientoContextMenu"
    @row-edit="(row: any) => emit('edit', row.id)"
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
          @click="emit('edit', data.id)"
        >
          <AppIcon name="pencil" :size="14" />
        </Button>
        <Button
          variant="ghost"
          size="sm"
          :disabled="data.bloqueadoPorLiquidacion"
          :aria-label="$t('General.Delete')"
          @click="emit('delete', data)"
        >
          <AppIcon name="trash-2" :size="14" />
        </Button>
      </div>
    </template>
  </DataGrid>
</template>
