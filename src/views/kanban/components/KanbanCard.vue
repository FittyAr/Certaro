<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import type { KanbanTarjetaDto, PrioridadTarjeta } from '@/stores/useKanbanStore'

const props = defineProps<{
  card: KanbanTarjetaDto
  isDragging?: boolean
  isDragHover?: boolean
}>()

const emit = defineEmits<{
  (e: 'pointerdown', event: PointerEvent): void
  (e: 'contextmenu', event: MouseEvent): void
  (e: 'edit'): void
  (e: 'delete'): void
  (e: 'openChecklist'): void
}>()

const { t } = useI18n()

function getPriorityClass(p: PrioridadTarjeta) {
  switch (p) {
    case 'Urgente':
      return 'bg-destructive/10 text-destructive border-destructive/30'
    case 'Alta':
      return 'bg-warning/10 text-warning border-warning/30'
    case 'Baja':
      return 'bg-muted text-muted-foreground border-border'
    default:
      return 'bg-primary/10 text-primary border-primary/30'
  }
}
</script>

<template>
  <div
    :data-card-id="props.card.id"
    class="p-3 rounded-lg bg-surface-elevated border shadow-xs hover:border-primary/50 transition-all cursor-grab active:cursor-grabbing flex flex-col gap-2 select-none"
    :class="[
      props.isDragHover ? 'border-primary ring-2 ring-primary/50' : 'border-border',
      props.isDragging ? 'opacity-30 border-dashed' : ''
    ]"
    @pointerdown="emit('pointerdown', $event)"
    @contextmenu.prevent="emit('contextmenu', $event)"
  >
    <!-- Card Meta: Priority & Tags -->
    <div class="flex flex-wrap items-center justify-between gap-1.5">
      <span
        class="text-[10px] uppercase font-bold px-1.5 py-0.5 rounded border"
        :class="getPriorityClass(props.card.prioridad)"
      >
        {{ props.card.prioridad }}
      </span>

      <div class="flex flex-wrap gap-1">
        <span
          v-for="tag in props.card.etiquetas"
          :key="tag.id"
          class="text-[10px] px-1.5 py-0.5 rounded font-medium border border-border bg-muted text-foreground"
        >
          {{ tag.nombre }}
        </span>
      </div>
    </div>

    <!-- Title & Description -->
    <div>
      <h4 class="text-xs font-semibold text-foreground leading-snug">
        {{ props.card.titulo }}
      </h4>
      <p
        v-if="props.card.descripcion"
        class="text-[11px] text-muted-foreground line-clamp-2 mt-1"
      >
        {{ props.card.descripcion }}
      </p>
    </div>

    <!-- Footer: Due date, Checklist & Actions -->
    <div class="flex items-center justify-between border-t border-border pt-2 text-[11px] text-muted-foreground">
      <div class="flex items-center gap-2">
        <span v-if="props.card.fechaVencimiento" class="flex items-center gap-1 font-mono">
          📅 {{ props.card.fechaVencimiento }}
        </span>
        <button
          type="button"
          class="hover:text-foreground flex items-center gap-1"
          @click.stop="emit('openChecklist')"
        >
          ☑ {{ props.card.completadasChecklist }}/{{ props.card.totalChecklist }}
        </button>
      </div>

      <div class="flex items-center gap-1">
        <button
          type="button"
          class="p-1 rounded-sm hover:bg-muted text-muted-foreground hover:text-foreground"
          :title="t('General.Edit')"
          @click.stop="emit('edit')"
        >
          ✎
        </button>
        <button
          type="button"
          class="p-1 rounded-sm hover:bg-muted text-destructive"
          :title="t('General.Delete')"
          @click.stop="emit('delete')"
        >
          ✕
        </button>
      </div>
    </div>
  </div>
</template>
