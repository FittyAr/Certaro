<script setup lang="ts">
import MultiSelect from 'primevue/multiselect'
import DateInput from '@/components/domain/DateInput.vue'
import type { LookupItem } from '@/stores/useCatalogStore'

defineProps<{
  seleccion: string[]
  periodo: { desde: string; hasta: string }
  empleadosOpciones: LookupItem[]
}>()

const emit = defineEmits<{
  (e: 'update:seleccion', val: string[]): void
  (e: 'update:periodo', val: { desde: string; hasta: string }): void
}>()
</script>

<template>
  <div class="flex flex-col gap-4">
    <label class="flex flex-col gap-1">
      <span class="text-sm">{{ $t('Liquidaciones.Empleados') }}</span>
      <MultiSelect
        :model-value="seleccion"
        :options="empleadosOpciones"
        option-label="label"
        option-value="id"
        :placeholder="$t('Liquidaciones.ElegirEmpleados')"
        filter
        display="chip"
        @update:model-value="emit('update:seleccion', $event)"
      />
    </label>
    <div class="grid grid-cols-2 gap-3">
      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('General.From') }}</span>
        <DateInput
          :model-value="periodo.desde"
          @update:model-value="emit('update:periodo', { ...periodo, desde: $event ?? '' })"
        />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('General.To') }}</span>
        <DateInput
          :model-value="periodo.hasta"
          @update:model-value="emit('update:periodo', { ...periodo, hasta: $event ?? '' })"
        />
      </label>
    </div>
    <p class="text-xs text-muted-foreground">{{ $t('Liquidaciones.PasoUnoAyuda') }}</p>
  </div>
</template>
