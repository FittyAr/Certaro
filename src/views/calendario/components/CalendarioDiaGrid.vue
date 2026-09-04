<script setup lang="ts">
import type { CalendarioEventoDto } from '@/stores/useCalendarioStore'
import { HORAS_DIA, pad, coincideHora, fechaLocalIsoDe } from '../composables/useCalendarioPeriodo'
import { getBadgeClass } from '../composables/useBadgeClass'

defineProps<{
  tituloPeriodo: string
  eventos: CalendarioEventoDto[]
  fechaSeleccionadaIso: string
}>()

const emit = defineEmits<{
  (e: 'crear', fechaIsoHora: string): void
  (e: 'editar', ev: CalendarioEventoDto): void
}>()
</script>

<template>
  <div class="flex flex-col h-full border border-border rounded-lg bg-surface-card overflow-hidden">
    <div class="border-b border-border bg-muted/30 p-3 text-sm font-semibold flex items-center justify-between">
      <span>Detalle de Agenda: {{ tituloPeriodo }}</span>
      <span class="text-xs text-muted-foreground font-normal">
        {{ eventos.length }} eventos programados
      </span>
    </div>
    <div class="flex-1 overflow-y-auto divide-y divide-border">
      <div
        v-for="hora in HORAS_DIA"
        :key="hora"
        class="flex items-start min-h-14 p-2 hover:bg-muted/10 cursor-pointer"
        @click="emit('crear', `${fechaSeleccionadaIso}T${pad(hora)}:00`)"
      >
        <span class="w-16 text-xs text-muted-foreground font-mono">{{ pad(hora) }}:00</span>
        <div class="flex-1 flex flex-wrap gap-2 pl-4">
          <div
            v-for="ev in eventos.filter(e => coincideHora(e.inicio, hora) && fechaLocalIsoDe(e.inicio) === fechaSeleccionadaIso)"
            :key="ev.id"
            :class="[
              'px-3 py-1.5 rounded-md border text-xs font-medium hover:opacity-80 flex items-center gap-2',
              getBadgeClass(ev.tipo, ev.esVirtual)
            ]"
            @click.stop="emit('editar', ev)"
          >
            <span>{{ ev.titulo }}</span>
            <span v-if="ev.recursos.length > 0" class="text-[10px] opacity-75 font-normal">
              ({{ ev.recursos.map(r => r.nombre).join(', ') }})
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
