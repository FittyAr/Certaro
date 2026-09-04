<script setup lang="ts">
import Select from 'primevue/select'
import type { LookupItem } from '@/stores/useCatalogStore'

defineProps<{
  clienteId: string | null
  selectedProyectoId: string | null
  trabajoId: string | null
  opcionesCliente: LookupItem[]
  opcionesProyecto: LookupItem[]
  opcionesTrabajo: LookupItem[]
}>()

const emit = defineEmits<{
  (e: 'update:clienteId', value: string | null): void
  (e: 'update:selectedProyectoId', value: string | null): void
  (e: 'update:trabajoId', value: string | null): void
  (e: 'clienteChange'): void
  (e: 'proyectoChange'): void
}>()
</script>

<template>
  <div class="space-y-3 rounded-md border border-border/70 bg-muted/20 p-3">
    <span class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
      {{ $t('Movimientos.ImputacionOpcional') }}
    </span>

    <label class="flex flex-col gap-1">
      <span class="text-xs text-muted-foreground">{{ $t('Movimientos.Cliente') }}</span>
      <Select
        :model-value="clienteId"
        :options="opcionesCliente"
        option-label="label"
        option-value="id"
        filter
        show-clear
        :placeholder="$t('General.None')"
        @update:model-value="(val) => emit('update:clienteId', val)"
        @change="emit('clienteChange')"
      />
    </label>

    <div class="grid grid-cols-2 gap-3">
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('Movimientos.Proyecto') }}</span>
        <Select
          :model-value="selectedProyectoId"
          :options="opcionesProyecto"
          option-label="label"
          option-value="id"
          filter
          show-clear
          :placeholder="$t('General.None')"
          @update:model-value="(val) => emit('update:selectedProyectoId', val)"
          @change="emit('proyectoChange')"
        />
      </label>

      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('Movimientos.Trabajo') }}</span>
        <Select
          :model-value="trabajoId"
          :options="opcionesTrabajo"
          option-label="label"
          option-value="id"
          filter
          show-clear
          :placeholder="$t('General.None')"
          :disabled="!selectedProyectoId && opcionesTrabajo.length === 0"
          @update:model-value="(val) => emit('update:trabajoId', val)"
        />
      </label>
    </div>
    <p
      v-if="selectedProyectoId && opcionesTrabajo.length === 0"
      class="rounded-md border border-warning/30 bg-warning/10 p-2 text-xs text-warning"
    >
      {{ $t('Movimientos.ProyectoSinTrabajosAviso') || 'Este proyecto no tiene trabajos creados aún. Recuerda crear al menos un trabajo en el proyecto para que los gastos se imputen a la caja y rentabilidad de la obra.' }}
    </p>
  </div>
</template>
