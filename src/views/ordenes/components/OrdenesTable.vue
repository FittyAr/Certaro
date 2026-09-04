<script setup lang="ts">
import Column from 'primevue/column'
import DataTable from 'primevue/datatable'
import DateText from '@/components/domain/DateText.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import type { OrdenTrabajoListItem } from '@/stores/useOrdenesTrabajoStore'

defineProps<{
  rows: OrdenTrabajoListItem[]
}>()

const emit = defineEmits<{
  (e: 'detalle', row: OrdenTrabajoListItem): void
  (e: 'editar', id: string): void
  (e: 'borrar', row: OrdenTrabajoListItem): void
}>()
</script>

<template>
  <DataTable
    :value="rows"
    data-key="id"
    size="small"
    class="text-sm"
    @row-dblclick="emit('detalle', $event.data as OrdenTrabajoListItem)"
  >
    <Column field="fecha" :header="$t('Ordenes.Fecha')">
      <template #body="{ data }"><DateText :value="data.fecha" /></template>
    </Column>
    <Column field="titulo" :header="$t('Ordenes.Titulo')" />
    <Column field="itemsCount" :header="$t('Ordenes.Items')" />
    <Column field="totalPresupuestado" :header="$t('Ordenes.TotalPresupuestado')">
      <template #body="{ data }"><MoneyText :value="data.totalPresupuestado" /></template>
    </Column>
    <Column field="certificadosCount" :header="$t('Ordenes.Certificados')" />
    <Column :header="$t('General.Actions')" :style="{ width: '8rem' }">
      <template #body="{ data }">
        <div class="flex gap-1">
          <Button
            variant="ghost"
            size="sm"
            :title="$t('Ordenes.VerDetalle')"
            @click="emit('detalle', data)"
          >
            <AppIcon name="eye" :size="14" />
          </Button>
          <Button variant="ghost" size="sm" @click="emit('editar', data.id)">
            <AppIcon name="pencil" :size="14" />
          </Button>
          <Button
            v-if="data.certificadosCount === 0"
            variant="ghost"
            size="sm"
            @click="emit('borrar', data)"
          >
            <AppIcon name="trash-2" :size="14" />
          </Button>
        </div>
      </template>
    </Column>
  </DataTable>
</template>
