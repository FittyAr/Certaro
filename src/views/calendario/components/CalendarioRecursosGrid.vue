<script setup lang="ts">
import type { CalendarioEventoDto, CalendarioRecursoDto } from '@/stores/useCalendarioStore'
import { formatearHoraLocal, fechaLocalIsoDe, formatearFechaIso } from '../composables/useCalendarioPeriodo'
import { getBadgeClass } from '../composables/useBadgeClass'

const props = defineProps<{
  tituloPeriodo: string
  recursos: CalendarioRecursoDto[]
  eventos: CalendarioEventoDto[]
  fechaSeleccionada: Date
  puedeGestionarRecursos: boolean
}>()

const emit = defineEmits<{
  (e: 'editar', ev: CalendarioEventoDto): void
  (e: 'sincronizar'): void
}>()

function eventosDeRecurso(recursoId: string) {
  const diaIso = formatearFechaIso(props.fechaSeleccionada)
  return props.eventos.filter((e) => {
    const eInicio = fechaLocalIsoDe(e.inicio)
    const eFin = fechaLocalIsoDe(e.fin)
    const coincideDia = diaIso >= eInicio && diaIso <= eFin
    const tieneRecurso = e.recursos.some((r) => r.id === recursoId)
    return coincideDia && tieneRecurso
  })
}
</script>

<template>
  <div class="flex flex-col h-full border border-border rounded-lg bg-surface-card overflow-hidden">
    <div class="border-b border-border bg-muted/30 p-3 flex items-center justify-between">
      <span class="text-sm font-semibold">Vista de Recursos: {{ tituloPeriodo }}</span>
      <div class="flex items-center gap-2">
        <span class="text-xs text-muted-foreground">
          {{ recursos.length }} recursos activos
        </span>
        <button
          v-if="puedeGestionarRecursos"
          type="button"
          class="px-2.5 py-1 text-xs border border-border rounded-md hover:bg-muted"
          @click="emit('sincronizar')"
        >
          Sincronizar Empleados
        </button>
      </div>
    </div>

    <div class="flex-1 overflow-x-auto flex">
      <div
        v-for="recurso in recursos"
        :key="recurso.id"
        class="flex-1 min-w-56 border-r border-border flex flex-col"
      >
        <!-- Column header -->
        <div class="p-2 border-b border-border bg-muted/20 text-center">
          <div class="text-xs font-bold truncate">{{ recurso.nombre }}</div>
          <div class="text-[10px] text-muted-foreground uppercase tracking-wider">
            {{ recurso.tipo }}
          </div>
        </div>

        <!-- Event list for resource -->
        <div class="flex-1 p-2 flex flex-col gap-2 overflow-y-auto">
          <div
            v-for="ev in eventosDeRecurso(recurso.id)"
            :key="ev.id"
            :class="[
              'p-2 rounded-md border text-xs cursor-pointer hover:opacity-80 transition-opacity',
              getBadgeClass(ev.tipo, ev.esVirtual)
            ]"
            @click="emit('editar', ev)"
          >
            <div class="font-semibold truncate">{{ ev.titulo }}</div>
            <div class="text-[10px] opacity-75 mt-0.5">
              {{ formatearHoraLocal(ev.inicio) }} - {{ formatearHoraLocal(ev.fin) }}
            </div>
            <div v-if="ev.descripcion" class="text-[10px] line-clamp-2 opacity-85 mt-1">
              {{ ev.descripcion }}
            </div>
          </div>
          <div
            v-if="eventosDeRecurso(recurso.id).length === 0"
            class="text-[11px] text-muted-foreground text-center py-8 italic"
          >
            Sin asignaciones hoy
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
