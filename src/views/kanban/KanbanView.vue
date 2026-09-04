<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute } from 'vue-router'
import {
  useKanbanStore,
  type KanbanColumnaDto,
  type KanbanTableroDto,
  type KanbanTarjetaDto,
  type PrioridadTarjeta,
  type Uuid,
} from '@/stores/useKanbanStore'
import { useProyectosStore } from '@/stores/useProyectosStore'
import { usePermission } from '@/composables/usePermission'

import KanbanHeader from './components/KanbanHeader.vue'
import KanbanFilterBar from './components/KanbanFilterBar.vue'
import KanbanColumn from './components/KanbanColumn.vue'
import KanbanModals from './components/KanbanModals.vue'
import KanbanContextMenus from './components/KanbanContextMenus.vue'

import { useKanbanModals } from './composables/useKanbanModals'
import { useKanbanDragAndDrop } from './composables/useKanbanDragAndDrop'

const { t } = useI18n()
const route = useRoute()
const store = useKanbanStore()
const proyectosStore = useProyectosStore()
const { can } = usePermission()

const canManage = computed(() => can('kanban:gestionar_tablero'))
const canCreate = computed(() => can('kanban:crear_tarjeta'))
const canMove = computed(() => can('kanban:mover_tarjeta'))

// Filters
const searchText = ref('')
const selectedPrioridad = ref<string>('all')
const selectedProyectoId = ref<string>('all')
const proyectosOptions = ref<{ label: string; value: string }[]>([])

// Modals management composable
const modals = useKanbanModals(store)

// Context Menus Component Ref
const contextMenusRef = ref<InstanceType<typeof KanbanContextMenus> | null>(null)

onMounted(async () => {
  await store.fetchTableros()
  try {
    const proys = await proyectosStore.lookup()
    proyectosOptions.value = [
      { label: 'Todos los proyectos', value: 'all' },
      ...proys.map((p) => ({ label: p.label, value: p.id })),
    ]
  } catch (_e) {
    // Proyectos lookup optional
  }

  if (route.query.proyectoId && typeof route.query.proyectoId === 'string') {
    selectedProyectoId.value = route.query.proyectoId
  }
})

watch(
  () => route.query.proyectoId,
  (newVal) => {
    if (newVal && typeof newVal === 'string') {
      selectedProyectoId.value = newVal
    }
  },
)

const sortedColumnas = computed(() => {
  if (!store.detalle) return []
  return [...store.detalle.columnas].sort((a, b) => a.orden - b.orden)
})

const filteredTarjetas = computed(() => {
  if (!store.detalle) return []
  return store.detalle.tarjetas.filter((t) => {
    if (searchText.value.trim()) {
      const q = searchText.value.toLowerCase()
      const matchTitle = t.titulo.toLowerCase().includes(q)
      const matchDesc = t.descripcion?.toLowerCase().includes(q)
      if (!matchTitle && !matchDesc) return false
    }
    if (selectedPrioridad.value !== 'all' && t.prioridad !== selectedPrioridad.value) {
      return false
    }
    if (selectedProyectoId.value !== 'all') {
      if (!t.proyectoId || t.proyectoId !== selectedProyectoId.value) {
        return false
      }
    }
    return true
  })
})

function getTarjetasPorColumna(columnaId: Uuid) {
  return filteredTarjetas.value
    .filter((t) => t.columnaId === columnaId)
    .sort((a, b) => a.orden - b.orden)
}

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

// Drag & Drop Composable
const {
  draggingCard,
  draggingColumn,
  dragPosition,
  dragHoverColumnaId,
  dragHoverCardId,
  onCardPointerDown,
  onColumnPointerDown,
  moverColumna,
} = useKanbanDragAndDrop({
  store,
  canMove,
  canManage,
  sortedColumnas,
  getTarjetasPorColumna,
  onCardClickToEdit: modals.openEditCard,
})

// -------------------------------------------------------------
// Context Menus
// -------------------------------------------------------------
function onCardContextMenu(event: MouseEvent, card: KanbanTarjetaDto) {
  contextMenusRef.value?.showCardMenu(event, card)
}

function onColumnContextMenu(event: MouseEvent, col: KanbanColumnaDto) {
  contextMenusRef.value?.showColumnMenu(event, col)
}

function onBoardContextMenu(event: MouseEvent, board: KanbanTableroDto) {
  contextMenusRef.value?.showBoardMenu(event, board)
}
</script>

<template>
  <div class="h-full flex flex-col gap-4 p-4 md:p-6 overflow-hidden bg-background text-foreground select-none">
    <!-- Context Menus (PrimeVue) -->
    <KanbanContextMenus
      ref="contextMenusRef"
      :store="store"
      :sorted-columnas="sortedColumnas"
      :can-manage="canManage"
      :can-create="canCreate"
      :can-move="canMove"
      :get-tarjetas-por-columna="getTarjetasPorColumna"
      :open-edit-card="modals.openEditCard"
      :open-checklist="modals.openChecklist"
      :remove-card="modals.removeCard"
      :open-create-card="modals.openCreateCard"
      :open-edit-column="modals.openEditColumn"
      :mover-columna="moverColumna"
      :confirm-delete-columna="modals.confirmDeleteColumna"
      :open-edit-board="modals.openEditBoard"
      :handle-delete-board-prompt="modals.handleDeleteBoardPrompt"
    />

    <!-- Floating Drag Ghost for Card -->
    <div
      v-if="draggingCard"
      class="fixed pointer-events-none z-50 p-3 rounded-lg bg-surface-elevated border-2 border-primary shadow-2xl opacity-95 w-72"
      :style="{
        left: `${dragPosition.x - 140}px`,
        top: `${dragPosition.y - 30}px`,
      }"
    >
      <div class="flex items-center justify-between mb-1">
        <span
          class="text-[10px] uppercase font-bold px-1.5 py-0.5 rounded border"
          :class="getPriorityClass(draggingCard.prioridad)"
        >
          {{ draggingCard.prioridad }}
        </span>
      </div>
      <h4 class="text-xs font-semibold text-foreground truncate">
        {{ draggingCard.titulo }}
      </h4>
    </div>

    <!-- Floating Drag Ghost for Column -->
    <div
      v-if="draggingColumn"
      class="fixed pointer-events-none z-50 p-3 rounded-xl bg-surface-card border-2 border-primary shadow-2xl opacity-95 w-72"
      :style="{
        left: `${dragPosition.x - 140}px`,
        top: `${dragPosition.y - 25}px`,
      }"
    >
      <h3 class="text-sm font-semibold text-foreground truncate flex items-center gap-2">
        <span>⋮⋮</span>
        <span>{{ draggingColumn.nombre }}</span>
      </h3>
    </div>

    <!-- Top Header: Board Switcher, Info Bar, Actions & Help -->
    <KanbanHeader
      :active-tableros="store.activeTableros"
      :current-tablero="store.currentTablero"
      :current-tablero-id="store.currentTableroId"
      :can-manage="canManage"
      :column-count="sortedColumnas.length"
      :card-count="filteredTarjetas.length"
      @select-tablero="store.selectTablero"
      @open-create-board="modals.openCreateBoard"
      @open-manage-boards="modals.showManageBoardsModal.value = true"
      @open-create-column="modals.openCreateColumn"
      @sync-preset="store.syncPreset"
      @board-context-menu="onBoardContextMenu"
    />

    <!-- Filter Bar -->
    <KanbanFilterBar
      v-model:search-text="searchText"
      v-model:selected-prioridad="selectedPrioridad"
      v-model:selected-proyecto-id="selectedProyectoId"
      :proyectos-options="proyectosOptions"
    />

    <!-- Loading / Error states -->
    <div v-if="store.loading && !store.detalle" class="py-12 text-center text-sm text-muted-foreground">
      {{ t('General.Loading') }}
    </div>

    <div
      v-else-if="store.error"
      class="p-4 rounded-lg bg-destructive/10 border border-destructive/30 text-destructive text-sm"
    >
      {{ store.error }}
    </div>

    <!-- Kanban Columns Horizontal Board -->
    <div
      v-else-if="store.detalle"
      class="flex-1 flex gap-4 overflow-x-auto pb-4 items-start"
    >
      <KanbanColumn
        v-for="(col, index) in sortedColumnas"
        :key="col.id"
        :col="col"
        :index="index"
        :total-columns="sortedColumnas.length"
        :cards="getTarjetasPorColumna(col.id)"
        :can-manage="canManage"
        :can-create="canCreate"
        :show-manual-move-buttons="store.showColumnMoveButtons"
        :is-preset-board="Boolean(store.currentTablero?.esPreset)"
        :drag-hover-columna-id="dragHoverColumnaId"
        :dragging-column-id="draggingColumn?.id ?? null"
        :drag-hover-card-id="dragHoverCardId"
        :dragging-card-id="draggingCard?.id ?? null"
        @column-pointer-down="onColumnPointerDown($event, col)"
        @column-context-menu="onColumnContextMenu($event, col)"
        @card-pointer-down="onCardPointerDown"
        @card-context-menu="onCardContextMenu"
        @move-column="moverColumna(col, $event)"
        @create-card="modals.openCreateCard(col.id)"
        @edit-column="modals.openEditColumn(col)"
        @delete-column="modals.confirmDeleteColumna(col)"
        @edit-card="modals.openEditCard"
        @delete-card="modals.removeCard"
        @open-checklist="modals.openChecklist"
      />
    </div>

    <!-- Modals -->
    <KanbanModals
      :modals="modals"
      :store="store"
      :get-tarjetas-por-columna="getTarjetasPorColumna"
    />
  </div>
</template>
