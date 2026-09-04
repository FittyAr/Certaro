<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute } from 'vue-router'
import ContextMenu from 'primevue/contextmenu'
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
import CardModal from './components/CardModal.vue'
import ColumnModal from './components/ColumnModal.vue'
import BoardModal from './components/BoardModal.vue'
import ManageBoardsModal from './components/ManageBoardsModal.vue'
import DeleteColumnModal from './components/DeleteColumnModal.vue'
import StrictDeleteBoardModal from './components/StrictDeleteBoardModal.vue'
import ChecklistModal from './components/ChecklistModal.vue'

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
const {
  showCardModal,
  editingCard,
  cardFormColumnaId,
  showColumnModal,
  editingColumn,
  showBoardModal,
  editingBoard,
  showManageBoardsModal,
  showDeleteColModal,
  colToDelete,
  showStrictDeleteBoardModal,
  boardToDelete,
  showChecklistModal,
  checklistCard,
  checklistItems,
  openCreateCard,
  openEditCard,
  handleSaveCard,
  removeCard,
  openCreateColumn,
  openEditColumn,
  handleSaveColumn,
  confirmDeleteColumna,
  executeDeleteColumn,
  openCreateBoard,
  openEditBoard,
  handleSaveBoard,
  handleDeleteBoardPrompt,
  executeStrictDeleteBoard,
  openChecklist,
  handleAddChecklist,
  handleToggleChecklist,
  handleRemoveChecklist,
} = useKanbanModals(store)

// Context Menus
const cardMenuRef = ref<InstanceType<typeof ContextMenu> | null>(null)
const columnMenuRef = ref<InstanceType<typeof ContextMenu> | null>(null)
const boardMenuRef = ref<InstanceType<typeof ContextMenu> | null>(null)

const contextCard = ref<KanbanTarjetaDto | null>(null)
const contextColumn = ref<KanbanColumnaDto | null>(null)
const contextBoard = ref<KanbanTableroDto | null>(null)

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
  onCardClickToEdit: openEditCard,
})

// -------------------------------------------------------------
// Context Menus
// -------------------------------------------------------------
function onCardContextMenu(event: MouseEvent, card: KanbanTarjetaDto) {
  contextCard.value = card
  cardMenuRef.value?.show(event)
}

function onColumnContextMenu(event: MouseEvent, col: KanbanColumnaDto) {
  contextColumn.value = col
  columnMenuRef.value?.show(event)
}

function onBoardContextMenu(event: MouseEvent, board: KanbanTableroDto) {
  contextBoard.value = board
  boardMenuRef.value?.show(event)
}

const cardMenuItems = computed(() => {
  const card = contextCard.value
  if (!card) return []

  const otherCols = sortedColumnas.value
    .filter((c) => c.id !== card.columnaId)
    .map((c) => ({
      label: c.nombre,
      command: async () => {
        const destCards = getTarjetasPorColumna(c.id)
        await store.reordenarTarjetas({
          tarjetaId: card.id,
          origenColumnaId: card.columnaId,
          destinoColumnaId: c.id,
          nuevoOrden: destCards.length,
          tarjetaIdsEnDestino: [...destCards.map((x) => x.id), card.id],
        })
      },
    }))

  const prioridades: PrioridadTarjeta[] = ['Baja', 'Normal', 'Alta', 'Urgente']
  const priorityItems = prioridades.map((p) => ({
    label: p,
    command: async () => {
      await store.updateTarjeta(card.id, {
        titulo: card.titulo,
        descripcion: card.descripcion,
        prioridad: p,
        fechaVencimiento: card.fechaVencimiento,
        etiquetaIds: card.etiquetas.map((e) => e.id),
        rowVersion: card.rowVersion,
      })
    },
  }))

  return [
    {
      label: t('General.Edit'),
      icon: 'pi pi-pencil',
      command: () => openEditCard(card),
    },
    {
      label: `${t('Kanban.Checklist')} (${card.completadasChecklist}/${card.totalChecklist})`,
      icon: 'pi pi-check-square',
      command: () => openChecklist(card),
    },
    { separator: true },
    {
      label: 'Mover a columna',
      icon: 'pi pi-arrow-right',
      disabled: !canMove.value || otherCols.length === 0,
      items: otherCols,
    },
    {
      label: t('Kanban.Priority'),
      icon: 'pi pi-flag',
      disabled: !canMove.value,
      items: priorityItems,
    },
    { separator: true },
    {
      label: t('General.Delete'),
      icon: 'pi pi-trash',
      command: () => removeCard(card),
    },
  ]
})

const columnMenuItems = computed(() => {
  const col = contextColumn.value
  if (!col) return []
  const cols = sortedColumnas.value
  const idx = cols.findIndex((c) => c.id === col.id)
  const isFirst = idx <= 0
  const isLast = idx === -1 || idx >= cols.length - 1

  return [
    {
      label: t('Kanban.NewCard'),
      icon: 'pi pi-plus',
      disabled: !canCreate.value,
      command: () => openCreateCard(col.id),
    },
    {
      label: t('Kanban.EditColumn'),
      icon: 'pi pi-pencil',
      disabled: !canManage.value,
      command: () => openEditColumn(col),
    },
    { separator: true },
    {
      label: 'Mover a la izquierda',
      icon: 'pi pi-arrow-left',
      disabled: !canManage.value || isFirst,
      command: () => moverColumna(col, 'izq'),
    },
    {
      label: 'Mover a la derecha',
      icon: 'pi pi-arrow-right',
      disabled: !canManage.value || isLast,
      command: () => moverColumna(col, 'der'),
    },
    { separator: true },
    {
      label: t('Kanban.DeleteColumn'),
      icon: 'pi pi-trash',
      disabled: !canManage.value || Boolean(store.currentTablero?.esPreset),
      command: () => confirmDeleteColumna(col),
    },
  ]
})

const boardMenuItems = computed(() => {
  const b = contextBoard.value
  if (!b) return []

  return [
    {
      label: store.isTableroPinned(b.id) ? 'Desfijar del inicio' : 'Fijar tablero al inicio',
      icon: 'pi pi-thumbtack',
      command: () => store.togglePinTablero(b.id),
    },
    {
      label: 'Editar tablero',
      icon: 'pi pi-pencil',
      disabled: !canManage.value,
      command: () => openEditBoard(b),
    },
    {
      label: b.activo ? 'Ocultar tablero' : 'Mostrar tablero',
      icon: b.activo ? 'pi pi-eye-slash' : 'pi pi-eye',
      disabled: !canManage.value,
      command: () => store.toggleTableroActivo(b),
    },
    {
      label: t('Kanban.Sync'),
      icon: 'pi pi-sync',
      visible: Boolean(b.esPreset),
      command: () => store.syncPreset(b.id),
    },
    { separator: true },
    {
      label: 'Eliminar tablero',
      icon: 'pi pi-trash',
      disabled: !canManage.value || Boolean(b.esPreset),
      command: () => handleDeleteBoardPrompt(b),
    },
  ]
})
</script>

<template>
  <div class="h-full flex flex-col gap-4 p-4 md:p-6 overflow-hidden bg-background text-foreground select-none">
    <!-- Context Menus (PrimeVue) -->
    <ContextMenu ref="cardMenuRef" :model="cardMenuItems" />
    <ContextMenu ref="columnMenuRef" :model="columnMenuItems" />
    <ContextMenu ref="boardMenuRef" :model="boardMenuItems" />

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
      @open-create-board="openCreateBoard"
      @open-manage-boards="showManageBoardsModal = true"
      @open-create-column="openCreateColumn"
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
        @create-card="openCreateCard(col.id)"
        @edit-column="openEditColumn(col)"
        @delete-column="confirmDeleteColumna(col)"
        @edit-card="openEditCard"
        @delete-card="removeCard"
        @open-checklist="openChecklist"
      />
    </div>

    <!-- Modals -->
    <CardModal
      :show="showCardModal"
      :editing-card="editingCard"
      :columna-id="cardFormColumnaId"
      @close="showCardModal = false"
      @save="handleSaveCard"
    />

    <ColumnModal
      :show="showColumnModal"
      :editing-column="editingColumn"
      :default-orden="store.detalle?.columnas.length ?? 0"
      @close="showColumnModal = false"
      @save="handleSaveColumn"
    />

    <BoardModal
      :show="showBoardModal"
      :editing-board="editingBoard"
      @close="showBoardModal = false"
      @save="handleSaveBoard"
    />

    <ManageBoardsModal
      :show="showManageBoardsModal"
      :tableros="store.tableros"
      @close="showManageBoardsModal = false"
      @create-board="openCreateBoard"
      @edit-board="openEditBoard"
      @delete-board="handleDeleteBoardPrompt"
    />

    <DeleteColumnModal
      :show="showDeleteColModal"
      :column="colToDelete"
      :card-count="colToDelete ? getTarjetasPorColumna(colToDelete.id).length : 0"
      @close="showDeleteColModal = false"
      @confirm="executeDeleteColumn"
    />

    <StrictDeleteBoardModal
      :show="showStrictDeleteBoardModal"
      :board="boardToDelete"
      @close="showStrictDeleteBoardModal = false"
      @confirm="executeStrictDeleteBoard"
    />

    <ChecklistModal
      :show="showChecklistModal"
      :card="checklistCard"
      :items="checklistItems"
      @close="showChecklistModal = false"
      @add-item="handleAddChecklist"
      @toggle-item="handleToggleChecklist"
      @remove-item="handleRemoveChecklist"
    />
  </div>
</template>
