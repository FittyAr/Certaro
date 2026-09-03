<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import KanbanCard from './KanbanCard.vue'
import type { KanbanColumnaDto, KanbanTarjetaDto, Uuid } from '@/stores/useKanbanStore'

const props = defineProps<{
  col: KanbanColumnaDto
  index: number
  totalColumns: number
  cards: KanbanTarjetaDto[]
  canManage: boolean
  canCreate: boolean
  showManualMoveButtons: boolean
  isPresetBoard: boolean
  dragHoverColumnaId: Uuid | null
  draggingColumnId: Uuid | null
  dragHoverCardId: Uuid | null
  draggingCardId: Uuid | null
}>()

const emit = defineEmits<{
  (e: 'columnPointerDown', event: PointerEvent): void
  (e: 'columnContextMenu', event: MouseEvent): void
  (e: 'cardPointerDown', event: PointerEvent, card: KanbanTarjetaDto): void
  (e: 'cardContextMenu', event: MouseEvent, card: KanbanTarjetaDto): void
  (e: 'moveColumn', direction: 'izq' | 'der'): void
  (e: 'createCard'): void
  (e: 'editColumn'): void
  (e: 'deleteColumn'): void
  (e: 'editCard', card: KanbanTarjetaDto): void
  (e: 'deleteCard', card: KanbanTarjetaDto): void
  (e: 'openChecklist', card: KanbanTarjetaDto): void
}>()

const { t } = useI18n()
</script>

<template>
  <div
    :data-columna-id="props.col.id"
    :class="[
      'w-80 shrink-0 flex flex-col max-h-full rounded-xl bg-surface-card border transition-all duration-150 shadow-xs overflow-hidden select-none',
      props.dragHoverColumnaId === props.col.id ? 'border-primary ring-2 ring-primary/40 bg-primary/5' : 'border-border',
      props.draggingColumnId === props.col.id ? 'opacity-30 border-dashed' : ''
    ]"
    :style="{
      borderTopColor: props.col.color || 'var(--color-primary, currentColor)',
      borderTopWidth: '4px'
    }"
  >
    <!-- Column Header -->
    <div
      class="p-3 border-b border-border flex items-center justify-between gap-2 bg-muted/20 select-none"
      :class="props.canManage ? 'cursor-grab active:cursor-grabbing' : ''"
      @pointerdown="emit('columnPointerDown', $event)"
      @contextmenu.prevent="emit('columnContextMenu', $event)"
    >
      <div class="flex items-center gap-2 min-w-0">
        <!-- Drag Handle icon for column -->
        <span v-if="props.canManage" class="text-muted-foreground text-xs leading-none">⋮⋮</span>
        <!-- Column colored dot indicator -->
        <span
          class="w-3 h-3 rounded-full shrink-0 border border-border shadow-xs"
          :style="{ backgroundColor: props.col.color || 'var(--color-primary, currentColor)' }"
        />
        <h3 class="text-sm font-semibold text-foreground truncate" :title="props.col.nombre">
          {{ props.col.nombre }}
        </h3>
        <span class="text-xs text-muted-foreground font-mono bg-muted px-1.5 py-0.5 rounded">
          {{ props.cards.length }}
          <template v-if="props.col.limiteWip"> / {{ props.col.limiteWip }}</template>
        </span>
      </div>

      <div class="flex items-center gap-1">
        <!-- Reorder column buttons (configured in Settings > General) -->
        <div v-if="props.showManualMoveButtons && props.canManage && props.totalColumns > 1" class="flex items-center">
          <button
            v-if="props.index > 0"
            type="button"
            class="p-1 rounded-sm hover:bg-muted text-muted-foreground hover:text-foreground text-[10px] leading-none"
            title="Mover columna a la izquierda"
            @click.stop="emit('moveColumn', 'izq')"
          >
            ◀
          </button>
          <button
            v-if="props.index < props.totalColumns - 1"
            type="button"
            class="p-1 rounded-sm hover:bg-muted text-muted-foreground hover:text-foreground text-[10px] leading-none"
            title="Mover columna a la derecha"
            @click.stop="emit('moveColumn', 'der')"
          >
            ▶
          </button>
        </div>

        <button
          v-if="props.canCreate"
          class="p-1 rounded-sm hover:bg-muted text-muted-foreground hover:text-foreground text-sm"
          :title="t('Kanban.NewCard')"
          @click.stop="emit('createCard')"
        >
          +
        </button>
        <button
          v-if="props.canManage"
          class="p-1 rounded-sm hover:bg-muted text-muted-foreground hover:text-foreground text-xs"
          :title="t('Kanban.EditColumn')"
          @click.stop="emit('editColumn')"
        >
          ✎
        </button>
        <button
          v-if="props.canManage && !props.isPresetBoard"
          class="p-1 rounded-sm hover:bg-muted text-destructive text-xs"
          :title="t('Kanban.DeleteColumn')"
          @click.stop="emit('deleteColumn')"
        >
          ✕
        </button>
      </div>
    </div>

    <!-- Cards Container -->
    <div class="flex-1 overflow-y-auto p-2.5 flex flex-col gap-2.5 min-h-30">
      <KanbanCard
        v-for="card in props.cards"
        :key="card.id"
        :card="card"
        :is-dragging="props.draggingCardId === card.id"
        :is-drag-hover="props.dragHoverCardId === card.id"
        @pointerdown="emit('cardPointerDown', $event, card)"
        @contextmenu="emit('cardContextMenu', $event, card)"
        @edit="emit('editCard', card)"
        @delete="emit('deleteCard', card)"
        @open-checklist="emit('openChecklist', card)"
      />
    </div>
  </div>
</template>
