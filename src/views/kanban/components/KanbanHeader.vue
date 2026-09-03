<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import HelpButton from '@/components/ui/HelpButton.vue'
import {
  useKanbanStore,
  type KanbanTableroDto,
  type Uuid,
} from '@/stores/useKanbanStore'

const props = defineProps<{
  activeTableros: KanbanTableroDto[]
  currentTablero: KanbanTableroDto | null
  currentTableroId: Uuid | null
  canManage: boolean
  columnCount: number
  cardCount: number
}>()

const emit = defineEmits<{
  (e: 'selectTablero', id: Uuid): void
  (e: 'openCreateBoard'): void
  (e: 'openManageBoards'): void
  (e: 'openCreateColumn'): void
  (e: 'syncPreset', id: Uuid): void
  (e: 'boardContextMenu', event: MouseEvent, board: KanbanTableroDto): void
}>()

const { t } = useI18n()
const store = useKanbanStore()
</script>

<template>
  <div class="flex flex-col gap-3 border-b border-border pb-3 select-none">
    <div class="flex flex-wrap items-center justify-between gap-3">
      <div class="flex items-center gap-2 overflow-x-auto pb-1 max-w-full">
        <!-- Board Tab Button with visible chromatic color reflection -->
        <button
          v-for="b in props.activeTableros"
          :key="b.id"
          class="px-3.5 py-1.5 rounded-md text-sm font-medium transition-all whitespace-nowrap flex items-center gap-2 relative border"
          :class="[
            props.currentTableroId === b.id
              ? 'bg-surface-elevated text-foreground shadow-xs font-semibold'
              : 'bg-surface-card hover:bg-muted text-muted-foreground border-border',
            !b.activo ? 'opacity-50 border-dashed' : ''
          ]"
          :style="[
            props.currentTableroId === b.id
              ? { borderBottomColor: b.color || 'var(--color-primary, currentColor)', borderBottomWidth: '3px', borderTopColor: 'transparent' }
              : { borderBottomColor: 'transparent' }
          ]"
          :title="b.activo ? 'Clic derecho para opciones del tablero' : 'Tablero oculto'"
          @click="emit('selectTablero', b.id)"
          @contextmenu.prevent="emit('boardContextMenu', $event, b)"
        >
          <!-- Board Color Dot -->
          <span
            class="w-2.5 h-2.5 rounded-full shrink-0 border border-border shadow-2xs"
            :style="{ backgroundColor: b.color || 'var(--color-primary, currentColor)' }"
          />

          <span>{{ b.nombre }}</span>

          <!-- Pinned Indicator -->
          <span
            v-if="store.isTableroPinned(b.id)"
            class="text-[11px] leading-none text-primary"
            title="Tablero fijado al inicio"
          >
            📌
          </span>

          <span
            v-if="!b.activo"
            class="text-[9px] px-1 py-0.2 rounded font-mono bg-warning/20 text-warning"
          >
            OCULTO
          </span>
          <!-- Lock Icon for preset boards -->
          <span
            v-else-if="b.esPreset"
            class="text-xs leading-none text-muted-foreground"
            title="Tablero del sistema (protegido)"
          >
            🔒
          </span>
        </button>

        <button
          v-if="props.canManage"
          class="px-2.5 py-1.5 rounded-md text-sm font-medium border border-dashed border-border hover:bg-muted text-muted-foreground whitespace-nowrap"
          :title="t('Kanban.NewBoard')"
          @click="emit('openCreateBoard')"
        >
          + {{ t('Kanban.NewBoard') }}
        </button>

        <button
          v-if="props.canManage"
          class="px-2.5 py-1.5 rounded-md text-sm font-medium border border-border hover:bg-muted text-muted-foreground whitespace-nowrap"
          title="Gestionar tableros activos y ocultos"
          @click="emit('openManageBoards')"
        >
          ⚙ Tableros
        </button>

        <!-- Help for Kanban Boards -->
        <HelpButton topic-id="kanban-overview" title="Ayuda sobre el funcionamiento de los tableros Kanban" />
      </div>

      <div class="flex items-center gap-2">
        <div v-if="props.currentTablero?.esPreset" class="flex items-center gap-1">
          <button
            class="px-3 py-1.5 rounded-md text-xs font-medium border border-border hover:bg-muted text-foreground flex items-center gap-1.5"
            title="Reconcilia manualmente las entidades con la base de datos"
            @click="emit('syncPreset', props.currentTablero.id)"
          >
            <span>↻</span>
            <span>{{ t('Kanban.Sync') }}</span>
          </button>
          <HelpButton topic-id="kanban-sync" title="¿Qué hace la sincronización?" />
        </div>

        <button
          v-if="props.canManage && props.currentTableroId"
          class="px-3 py-1.5 rounded-md text-xs font-medium border border-border hover:bg-muted text-foreground"
          @click="emit('openCreateColumn')"
        >
          + {{ t('Kanban.NewColumn') }}
        </button>

        <HelpButton topic-id="kanban-columns" title="Ayuda sobre columnas y límites WIP" />
      </div>
    </div>

    <!-- Active Board Context Bar with Color Accent -->
    <div
      v-if="props.currentTablero"
      class="flex items-center justify-between px-3.5 py-2 rounded-lg bg-surface-card border text-xs"
      :style="{
        borderLeftColor: props.currentTablero.color || 'var(--color-primary, currentColor)',
        borderLeftWidth: '4px'
      }"
    >
      <div class="flex items-center gap-2.5">
        <span
          class="w-3 h-3 rounded-full shrink-0 border border-border"
          :style="{ backgroundColor: props.currentTablero.color || 'var(--color-primary, currentColor)' }"
        />
        <span class="font-bold text-foreground">{{ props.currentTablero.nombre }}</span>
        <span v-if="props.currentTablero.descripcion" class="text-muted-foreground">
          — {{ props.currentTablero.descripcion }}
        </span>
      </div>
      <div class="flex items-center gap-2 text-muted-foreground">
        <span>{{ props.columnCount }} columnas</span>
        <span>•</span>
        <span>{{ props.cardCount }} tarjetas visibles</span>
      </div>
    </div>
  </div>
</template>
