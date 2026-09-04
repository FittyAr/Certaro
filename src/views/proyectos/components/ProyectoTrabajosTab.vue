<script setup lang="ts">
import Column from 'primevue/column'
import DataTable from 'primevue/datatable'
import MoneyText from '@/components/domain/MoneyText.vue'
import StatePill from '@/components/domain/StatePill.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import type { TrabajoListItem } from '@/stores/useTrabajosStore'

defineProps<{
  items: TrabajoListItem[]
}>()

const emit = defineEmits<{
  (e: 'nuevo'): void
  (e: 'ver-ordenes', trabajoId: string): void
  (e: 'ver-detalle', trabajoId: string): void
}>()
</script>

<template>
  <div class="space-y-3">
    <div class="flex items-center justify-between">
      <span class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
        Trabajos y Cómputo de la Obra
      </span>
      <Button size="sm" @click="emit('nuevo')">
        <AppIcon name="plus" :size="14" />
        {{ $t('General.New') }} {{ $t('Entity.Trabajo') }}
      </Button>
    </div>

    <div v-if="items.length === 0" class="rounded-lg border border-dashed border-border p-8 text-center text-sm text-muted-foreground">
      {{ $t('Trabajos.Empty') }}
    </div>

    <DataTable
      v-else
      :value="items"
      data-key="id"
      size="small"
      class="text-sm"
    >
      <Column field="descripcion" :header="$t('Trabajos.Descripcion')">
        <template #body="{ data }">
          <span class="font-medium text-foreground">{{ data.descripcion }}</span>
        </template>
      </Column>
      <Column field="estado" :header="$t('Trabajos.Estado')">
        <template #body="{ data }">
          <StatePill entity="Trabajo" :value="data.estado" />
        </template>
      </Column>
      <Column field="presupuesto" :header="$t('Trabajos.Presupuesto')">
        <template #body="{ data }">
          <MoneyText :value="data.presupuesto" />
        </template>
      </Column>
      <Column :header="$t('General.Actions')" class="w-40 text-right">
        <template #body="{ data }">
          <div class="flex items-center justify-end gap-1">
            <Button
              variant="ghost"
              size="sm"
              :title="$t('Ordenes.Title')"
              @click="emit('ver-ordenes', data.id)"
            >
              <AppIcon name="file-text" :size="14" />
              <span class="ml-1 text-xs">{{ $t('Ordenes.Title') }}</span>
            </Button>
            <Button
              variant="ghost"
              size="sm"
              :title="$t('General.View')"
              @click="emit('ver-detalle', data.id)"
            >
              <AppIcon name="eye" :size="14" />
            </Button>
          </div>
        </template>
      </Column>
    </DataTable>
  </div>
</template>
