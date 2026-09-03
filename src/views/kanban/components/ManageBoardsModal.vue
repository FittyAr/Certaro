<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import HelpButton from '@/components/ui/HelpButton.vue'
import {
  useKanbanStore,
  type KanbanTableroDto,
} from '@/stores/useKanbanStore'

const props = defineProps<{
  show: boolean
  tableros: KanbanTableroDto[]
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'createBoard'): void
  (e: 'editBoard', board: KanbanTableroDto): void
  (e: 'deleteBoard', board: KanbanTableroDto): void
}>()

const { t } = useI18n()
const store = useKanbanStore()
</script>

<template>
  <div
    v-if="props.show"
    class="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-xs p-4 select-none"
  >
    <div class="w-full max-w-lg rounded-xl bg-surface-card border border-border p-5 shadow-lg flex flex-col gap-4">
      <div class="flex items-center justify-between border-b border-border pb-2">
        <div class="flex items-center gap-2">
          <div>
            <h3 class="text-sm font-semibold text-foreground">Gestionar Tableros</h3>
            <p class="text-xs text-muted-foreground mt-0.5">
              Oculta o muestra tableros de la barra superior. Los tableros personalizados pueden eliminarse.
            </p>
          </div>
          <HelpButton topic-id="kanban-boards-management" title="Guía sobre gestión y ciclo de vida de tableros" />
        </div>
        <button type="button" class="text-muted-foreground hover:text-foreground" @click="emit('close')">✕</button>
      </div>

      <div class="flex flex-col gap-2 max-h-80 overflow-y-auto">
        <div
          v-for="b in props.tableros"
          :key="b.id"
          class="flex items-center justify-between gap-3 p-2.5 rounded-lg border text-xs"
          :class="b.activo ? 'border-border bg-surface-elevated' : 'border-dashed border-border bg-muted/30 opacity-70'"
        >
          <div class="flex items-center gap-2 min-w-0">
            <span
              class="w-3 h-3 rounded-full shrink-0 border border-border"
              :style="{ backgroundColor: b.color || 'var(--color-primary, currentColor)' }"
            />
            <span class="font-medium text-foreground truncate">{{ b.nombre }}</span>
            <span
              v-if="store.isTableroPinned(b.id)"
              class="text-xs text-primary"
              title="Fijado al inicio"
            >
              📌
            </span>
            <span
              v-if="b.esPreset"
              class="text-xs text-muted-foreground"
              title="Tablero preestablecido del sistema (protegido)"
            >
              🔒
            </span>
            <span
              v-if="!b.activo"
              class="text-[9px] px-1 py-0.2 rounded font-mono bg-warning/10 text-warning"
            >
              OCULTO
            </span>
          </div>

          <div class="flex items-center gap-1.5 shrink-0">
            <!-- Pin / Unpin button -->
            <button
              type="button"
              class="p-1 rounded text-xs border border-border hover:bg-muted"
              :class="store.isTableroPinned(b.id) ? 'text-primary bg-primary/10' : 'text-muted-foreground'"
              :title="store.isTableroPinned(b.id) ? 'Desfijar del inicio' : 'Fijar al inicio de la lista'"
              @click="store.togglePinTablero(b.id)"
            >
              📌
            </button>
            <!-- Toggle visibility -->
            <button
              type="button"
              class="px-2 py-1 rounded text-xs border border-border hover:bg-muted"
              :class="b.activo ? 'text-muted-foreground' : 'text-primary font-medium'"
              :title="b.activo ? 'Ocultar este tablero' : 'Hacer visible este tablero'"
              @click="store.toggleTableroActivo(b)"
            >
              {{ b.activo ? 'Ocultar' : 'Mostrar' }}
            </button>

            <!-- Edit -->
            <button
              type="button"
              class="p-1 rounded hover:bg-muted text-muted-foreground hover:text-foreground"
              title="Editar tablero"
              @click="emit('editBoard', b)"
            >
              ✎
            </button>

            <!-- Delete (only custom boards) -->
            <button
              v-if="!b.esPreset"
              type="button"
              class="p-1 rounded hover:bg-muted text-destructive"
              title="Eliminar tablero"
              @click="emit('deleteBoard', b)"
            >
              ✕
            </button>
          </div>
        </div>
      </div>

      <div class="flex justify-between items-center border-t border-border pt-3">
        <button
          type="button"
          class="px-3 py-1.5 rounded-md text-xs font-medium border border-border hover:bg-muted text-foreground"
          @click="emit('createBoard')"
        >
          + {{ t('Kanban.NewBoard') }}
        </button>
        <button
          type="button"
          class="px-4 py-1.5 rounded-md text-xs font-medium bg-primary text-primary-foreground hover:bg-primary/90"
          @click="emit('close')"
        >
          {{ t('General.Close') }}
        </button>
      </div>
    </div>
  </div>
</template>
