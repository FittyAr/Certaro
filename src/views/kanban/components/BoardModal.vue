<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { KANBAN_PRESET_COLORS } from '@/lib/kanbanColors'
import type { KanbanTableroDto } from '@/stores/useKanbanStore'

const props = defineProps<{
  show: boolean
  editingBoard: KanbanTableroDto | null
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (
    e: 'save',
    data: {
      nombre: string
      descripcion: string | null
      color: string | null
    },
  ): void
}>()

const { t } = useI18n()

const formNombre = ref('')
const formDescripcion = ref('')
const formColor = ref('')

watch(
  () => props.show,
  (open) => {
    if (open) {
      if (props.editingBoard) {
        formNombre.value = props.editingBoard.nombre
        formDescripcion.value = props.editingBoard.descripcion ?? ''
        formColor.value = props.editingBoard.color ?? ''
      } else {
        formNombre.value = ''
        formDescripcion.value = ''
        formColor.value = ''
      }
    }
  },
  { immediate: true },
)

function submit() {
  if (!formNombre.value.trim()) return
  emit('save', {
    nombre: formNombre.value.trim(),
    descripcion: formDescripcion.value.trim() || null,
    color: formColor.value || null,
  })
}
</script>

<template>
  <div
    v-if="props.show"
    class="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-xs p-4 select-none"
  >
    <div class="w-full max-w-sm rounded-xl bg-surface-card border border-border p-5 shadow-lg flex flex-col gap-4">
      <div class="flex items-center justify-between border-b border-border pb-2">
        <h3 class="text-sm font-semibold text-foreground">
          {{ props.editingBoard ? 'Editar Tablero' : t('Kanban.NewBoard') }}
        </h3>
        <button type="button" class="text-muted-foreground hover:text-foreground" @click="emit('close')">✕</button>
      </div>

      <div class="flex flex-col gap-3 text-xs">
        <div>
          <label class="block font-medium text-foreground mb-1">{{ t('Kanban.BoardName') }} *</label>
          <input
            v-model="formNombre"
            type="text"
            class="w-full px-3 py-1.5 rounded-md bg-background border border-border text-foreground focus:outline-hidden focus:ring-1 focus:ring-primary"
          />
        </div>

        <div>
          <label class="block font-medium text-foreground mb-1">{{ t('Kanban.Description') }}</label>
          <textarea
            v-model="formDescripcion"
            rows="2"
            class="w-full px-3 py-1.5 rounded-md bg-background border border-border text-foreground focus:outline-hidden"
          />
        </div>

        <div>
          <label class="block font-medium text-foreground mb-1">{{ t('Kanban.Color') }}</label>
          <div class="flex items-center gap-2">
            <input
              v-model="formColor"
              type="color"
              class="w-10 h-8 p-1 rounded-md bg-background border border-border cursor-pointer"
            />
            <span class="text-xs font-mono text-muted-foreground">{{ formColor || 'Predeterminado' }}</span>
          </div>
          <!-- Quick preset swatches -->
          <div class="flex items-center gap-1.5 mt-2">
            <button
              v-for="c in KANBAN_PRESET_COLORS"
              :key="c"
              type="button"
              class="w-5 h-5 rounded-full border border-border transition-transform hover:scale-110"
              :style="{ backgroundColor: c }"
              :title="c"
              @click="formColor = c"
            />
          </div>
        </div>
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
          class="px-4 py-1.5 rounded-md text-xs font-medium bg-primary text-primary-foreground hover:bg-primary/90"
          @click="submit"
        >
          {{ t('General.Save') }}
        </button>
      </div>
    </div>
  </div>
</template>
