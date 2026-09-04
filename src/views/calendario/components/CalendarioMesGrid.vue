<script setup lang="ts">
import type { CalendarioEventoDto } from '@/stores/useCalendarioStore'
import { DIAS_SEMANA } from '../composables/useCalendarioPeriodo'
import { getBadgeClass } from '../composables/useBadgeClass'

export interface DiaMes {
  fecha: Date
  fechaIso: string
  numeroDia: number
  esMesActual: boolean
  esHoy: boolean
  eventos: CalendarioEventoDto[]
}

defineProps<{
  dias: DiaMes[]
}>()

const emit = defineEmits<{
  (e: 'crear', fechaIso: string): void
  (e: 'editar', ev: CalendarioEventoDto): void
}>()
</script>

<template>
  <div class="flex flex-col h-full border border-border rounded-lg bg-surface-card overflow-hidden">
    <!-- Weekday headers -->
    <div class="grid grid-cols-7 border-b border-border bg-muted/30 text-center text-xs font-semibold py-2">
      <div v-for="dia in DIAS_SEMANA" :key="dia">{{ dia }}</div>
    </div>

    <!-- Month Day Cells -->
    <div class="grid grid-cols-7 flex-1 auto-rows-fr">
      <div
        v-for="dia in dias"
        :key="dia.fechaIso"
        :class="[
          'border-b border-r border-border p-1.5 flex flex-col min-h-24 transition-colors cursor-pointer hover:bg-muted/20',
          !dia.esMesActual ? 'opacity-40 bg-muted/10' : '',
          dia.esHoy ? 'bg-primary/5' : ''
        ]"
        @click="emit('crear', dia.fechaIso)"
      >
        <div class="flex items-center justify-between mb-1">
          <span
            :class="[
              'text-xs font-medium px-1.5 py-0.5 rounded-full',
              dia.esHoy ? 'bg-primary text-primary-foreground font-bold' : 'text-muted-foreground'
            ]"
          >
            {{ dia.numeroDia }}
          </span>
          <span v-if="dia.eventos.length > 3" class="text-[10px] text-muted-foreground font-medium">
            +{{ dia.eventos.length - 3 }}
          </span>
        </div>

        <!-- Event Pills (max 3 displayed) -->
        <div class="flex flex-col gap-1 overflow-hidden">
          <div
            v-for="ev in dia.eventos.slice(0, 3)"
            :key="ev.id"
            :class="[
              'text-[11px] px-1.5 py-0.5 rounded-md border truncate font-medium flex items-center justify-between cursor-pointer hover:opacity-80',
              getBadgeClass(ev.tipo, ev.esVirtual)
            ]"
            @click.stop="emit('editar', ev)"
          >
            <span class="truncate">{{ ev.titulo }}</span>
            <span v-if="ev.esVirtual" class="text-[9px] uppercase tracking-wider font-semibold opacity-70">
              Virtual
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
