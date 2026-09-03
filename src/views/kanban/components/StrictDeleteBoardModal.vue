<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import type { KanbanTableroDto } from '@/stores/useKanbanStore'

const props = defineProps<{
  show: boolean
  board: KanbanTableroDto | null
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'confirm'): void
}>()

const { t } = useI18n()
const confirmInput = ref('')

watch(
  () => props.show,
  (open) => {
    if (open) {
      confirmInput.value = ''
    }
  },
)

function submit() {
  if (!props.board) return
  if (confirmInput.value.trim().toLowerCase() === props.board.nombre.trim().toLowerCase()) {
    emit('confirm')
  }
}
</script>

<template>
  <div
    v-if="props.show && props.board"
    class="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-xs p-4 select-none"
  >
    <div class="w-full max-w-md rounded-xl bg-surface-card border-2 border-destructive/50 p-5 shadow-2xl flex flex-col gap-4">
      <div class="flex items-center justify-between border-b border-border pb-2">
        <h3 class="text-sm font-bold text-destructive flex items-center gap-1.5">
          <span>⚠</span>
          <span>Confirmación Crítica: Eliminar Tablero</span>
        </h3>
        <button type="button" class="text-muted-foreground hover:text-foreground" @click="emit('close')">✕</button>
      </div>

      <div class="text-xs text-foreground flex flex-col gap-3">
        <div class="p-3 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive leading-relaxed">
          <p class="font-semibold mb-1">Este tablero contiene tarjetas y datos activos.</p>
          <p>
            Si continúas, el tablero <strong>"{{ props.board.nombre }}"</strong> y todas sus columnas y tarjetas serán eliminados permanentemente.
          </p>
        </div>

        <div>
          <label class="block font-medium text-foreground mb-1.5">
            Para confirmar, escribe exactamente el nombre del tablero:
            <strong class="font-mono text-primary select-all">{{ props.board.nombre }}</strong>
          </label>
          <input
            v-model="confirmInput"
            type="text"
            :placeholder="props.board.nombre"
            class="w-full px-3 py-2 rounded-md bg-background border border-border text-foreground font-mono focus:outline-hidden focus:ring-2 focus:ring-destructive"
            @keyup.enter="submit"
          />
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
          class="px-4 py-1.5 rounded-md text-xs font-semibold bg-destructive text-destructive-foreground hover:bg-destructive/90 transition-opacity disabled:opacity-30 disabled:cursor-not-allowed"
          :disabled="confirmInput.trim().toLowerCase() !== props.board.nombre.trim().toLowerCase()"
          @click="submit"
        >
          Eliminar permanentemente
        </button>
      </div>
    </div>
  </div>
</template>
