<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import type { KanbanColumnaDto } from '@/stores/useKanbanStore'

const props = defineProps<{
  show: boolean
  column: KanbanColumnaDto | null
  cardCount: number
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'confirm'): void
}>()

const { t } = useI18n()
</script>

<template>
  <div
    v-if="props.show && props.column"
    class="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-xs p-4 select-none"
  >
    <div class="w-full max-w-sm rounded-xl bg-surface-card border border-destructive/40 p-5 shadow-lg flex flex-col gap-4">
      <div class="flex items-center justify-between border-b border-border pb-2">
        <h3 class="text-sm font-semibold text-destructive flex items-center gap-1.5">
          <span>⚠</span>
          <span>Eliminar Columna</span>
        </h3>
        <button type="button" class="text-muted-foreground hover:text-foreground" @click="emit('close')">✕</button>
      </div>

      <div class="text-xs text-foreground flex flex-col gap-2">
        <p>
          ¿Estás seguro de que deseas eliminar la columna <strong>"{{ props.column.nombre }}"</strong>?
        </p>
        <p
          v-if="props.cardCount > 0"
          class="p-2 rounded bg-destructive/10 border border-destructive/20 text-destructive"
        >
          Esta columna contiene <strong>{{ props.cardCount }} tarjetas</strong> que también serán eliminadas permanentemente.
        </p>
      </div>

      <div class="flex justify-end gap-2 border-t border-border pt-3">
        <button
          type="button"
          class="px-3 py-1.5 rounded-md text-xs font-medium border border-border hover:bg-muted text-muted-foreground"
          @click="emit('close')"
        >
          {{ t('General.Cancel') }}
        </button>
        <button
          type="button"
          class="px-4 py-1.5 rounded-md text-xs font-medium bg-destructive text-destructive-foreground hover:bg-destructive/90"
          @click="emit('confirm')"
        >
          {{ t('General.Delete') }}
        </button>
      </div>
    </div>
  </div>
</template>
