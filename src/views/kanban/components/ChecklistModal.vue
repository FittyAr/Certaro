<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import type { KanbanTarjetaDto } from '@/stores/useKanbanStore'

const props = defineProps<{
  show: boolean
  card: KanbanTarjetaDto | null
  items: any[]
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'addItem', titulo: string): void
  (e: 'toggleItem', item: any): void
  (e: 'removeItem', item: any): void
}>()

const { t } = useI18n()
const newTitle = ref('')

function submitAdd() {
  if (!newTitle.value.trim()) return
  emit('addItem', newTitle.value.trim())
  newTitle.value = ''
}
</script>

<template>
  <div
    v-if="props.show && props.card"
    class="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-xs p-4 select-none"
  >
    <div class="w-full max-w-sm rounded-xl bg-surface-card border border-border p-5 shadow-lg flex flex-col gap-4">
      <div class="flex items-center justify-between border-b border-border pb-2">
        <h3 class="text-sm font-semibold text-foreground">
          {{ t('Kanban.Checklist') }}: {{ props.card.titulo }}
        </h3>
        <button type="button" class="text-muted-foreground hover:text-foreground" @click="emit('close')">✕</button>
      </div>

      <div class="flex flex-col gap-2 max-h-60 overflow-y-auto">
        <div
          v-for="item in props.items"
          :key="item.id"
          class="flex items-center justify-between gap-2 p-1.5 rounded hover:bg-muted/50 text-xs"
        >
          <label class="flex items-center gap-2 cursor-pointer flex-1 min-w-0">
            <input
              type="checkbox"
              :checked="item.completada"
              class="rounded border-border text-primary"
              @change="emit('toggleItem', item)"
            />
            <span
              class="truncate"
              :class="item.completada ? 'line-through text-muted-foreground' : 'text-foreground'"
            >
              {{ item.titulo }}
            </span>
          </label>
          <button
            type="button"
            class="p-1 text-destructive hover:bg-destructive/10 rounded"
            @click="emit('removeItem', item)"
          >
            ✕
          </button>
        </div>
        <div v-if="props.items.length === 0" class="text-xs text-muted-foreground py-2 text-center">
          {{ t('Kanban.EmptyChecklist') }}
        </div>
      </div>

      <div class="flex gap-2 border-t border-border pt-3">
        <input
          v-model="newTitle"
          type="text"
          :placeholder="t('Kanban.NewChecklistItem')"
          class="flex-1 px-2.5 py-1 text-xs rounded-md bg-background border border-border text-foreground focus:outline-hidden"
          @keyup.enter="submitAdd"
        />
        <button
          type="button"
          class="px-3 py-1 rounded-md text-xs font-medium bg-primary text-primary-foreground hover:bg-primary/90"
          @click="submitAdd"
        >
          {{ t('Kanban.Add') }}
        </button>
      </div>
    </div>
  </div>
</template>
