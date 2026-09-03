<script setup lang="ts">
import { useI18n } from 'vue-i18n'

const props = defineProps<{
  searchText: string
  selectedPrioridad: string
  selectedProyectoId: string
  proyectosOptions: { label: string; value: string }[]
}>()

const emit = defineEmits<{
  (e: 'update:searchText', value: string): void
  (e: 'update:selectedPrioridad', value: string): void
  (e: 'update:selectedProyectoId', value: string): void
}>()

const { t } = useI18n()
</script>

<template>
  <div class="flex flex-wrap items-center gap-3 bg-surface-card border border-border p-2.5 rounded-lg select-none">
    <div class="relative flex-1 min-w-50">
      <input
        :value="props.searchText"
        type="text"
        :placeholder="t('Kanban.SearchCards')"
        class="w-full px-3 py-1.5 text-xs rounded-md bg-background border border-border text-foreground placeholder:text-muted-foreground focus:outline-hidden focus:ring-1 focus:ring-primary"
        @input="emit('update:searchText', ($event.target as HTMLInputElement).value)"
      />
    </div>

    <!-- Priority filter -->
    <div class="flex items-center gap-1.5 text-xs text-muted-foreground">
      <span>{{ t('Kanban.Priority') }}:</span>
      <select
        :value="props.selectedPrioridad"
        class="px-2.5 py-1 text-xs rounded-md bg-background border border-border text-foreground focus:outline-hidden"
        @change="emit('update:selectedPrioridad', ($event.target as HTMLSelectElement).value)"
      >
        <option value="all">{{ t('Kanban.All') }}</option>
        <option value="Baja">{{ t('Kanban.PriorityLow') }}</option>
        <option value="Normal">{{ t('Kanban.PriorityNormal') }}</option>
        <option value="Alta">{{ t('Kanban.PriorityHigh') }}</option>
        <option value="Urgente">{{ t('Kanban.PriorityUrgent') }}</option>
      </select>
    </div>

    <!-- Project filter -->
    <div v-if="props.proyectosOptions.length > 1" class="flex items-center gap-1.5 text-xs text-muted-foreground">
      <span>Proyecto:</span>
      <select
        :value="props.selectedProyectoId"
        class="px-2.5 py-1 text-xs rounded-md bg-background border border-border text-foreground focus:outline-hidden max-w-52 truncate"
        @change="emit('update:selectedProyectoId', ($event.target as HTMLSelectElement).value)"
      >
        <option v-for="opt in props.proyectosOptions" :key="opt.value" :value="opt.value">
          {{ opt.label }}
        </option>
      </select>
    </div>
  </div>
</template>
