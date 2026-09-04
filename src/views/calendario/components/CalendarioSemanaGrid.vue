<script setup lang="ts">
import type { CalendarioEventoDto } from '@/stores/useCalendarioStore'
import { HORAS_DIA, pad, coincideHora } from '../composables/useCalendarioPeriodo'
import { getBadgeClass } from '../composables/useBadgeClass'

export interface DiaSemana {
  fecha: Date
  fechaIso: string
  diaNombre?: string
  numeroDia: number
  esHoy: boolean
  eventos: CalendarioEventoDto[]
}

defineProps<{
  dias: DiaSemana[]
}>()

const emit = defineEmits<{
  (e: 'crear', fechaIsoHora: string): void
  (e: 'editar', ev: CalendarioEventoDto): void
}>()
</script>

<template>
  <div class="flex flex-col h-full border border-border rounded-lg bg-surface-card overflow-hidden">
    <!-- Week header -->
    <div class="grid grid-cols-8 border-b border-border bg-muted/30 text-xs font-semibold py-2">
      <div class="text-center text-muted-foreground">Hora</div>
      <div
        v-for="dia in dias"
        :key="dia.fechaIso"
        :class="['text-center', dia.esHoy ? 'text-primary font-bold' : '']"
      >
        {{ dia.diaNombre }} {{ dia.numeroDia }}
      </div>
    </div>

    <!-- Hourly rows -->
    <div class="flex-1 overflow-y-auto">
      <div
        v-for="hora in HORAS_DIA"
        :key="hora"
        class="grid grid-cols-8 border-b border-border min-h-12 text-xs"
      >
        <div class="border-r border-border p-1 text-center text-muted-foreground text-[11px] font-mono">
          {{ pad(hora) }}:00
        </div>
        <div
          v-for="dia in dias"
          :key="dia.fechaIso"
          class="border-r border-border p-1 flex flex-col gap-1 hover:bg-muted/10 cursor-pointer"
          @click="emit('crear', `${dia.fechaIso}T${pad(hora)}:00`)"
        >
          <div
            v-for="ev in dia.eventos.filter(e => coincideHora(e.inicio, hora))"
            :key="ev.id"
            :class="[
              'text-[11px] px-1.5 py-0.5 rounded-md border font-medium truncate hover:opacity-80',
              getBadgeClass(ev.tipo, ev.esVirtual)
            ]"
            @click.stop="emit('editar', ev)"
          >
            {{ ev.titulo }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
