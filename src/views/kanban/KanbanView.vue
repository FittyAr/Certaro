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

// Pointer-based Drag & Drop State
type DragType = 'card' | 'column' | null
const dragType = ref<DragType>(null)
const draggingCard = ref<KanbanTarjetaDto | null>(null)
const draggingColumn = ref<KanbanColumnaDto | null>(null)
const dragPosition = ref({ x: 0, y: 0 })
const dragHoverColumnaId = ref<Uuid | null>(null)
const dragHoverCardId = ref<Uuid | null>(null)

let startX = 0
let startY = 0
let activeCard: KanbanTarjetaDto | null = null
let activeColumn: KanbanColumnaDto | null = null
let hasMovedEnough = false

// Context Menus
const cardMenuRef = ref<InstanceType<typeof ContextMenu> | null>(null)
const columnMenuRef = ref<InstanceType<typeof ContextMenu> | null>(null)
const boardMenuRef = ref<InstanceType<typeof ContextMenu> | null>(null)

const contextCard = ref<KanbanTarjetaDto | null>(null)
const contextColumn = ref<KanbanColumnaDto | null>(null)
const contextBoard = ref<KanbanTableroDto | null>(null)

// Modals State
const showCardModal = ref(false)
const editingCard = ref<KanbanTarjetaDto | null>(null)
const cardFormColumnaId = ref<Uuid>('')

const showColumnModal = ref(false)
const editingColumn = ref<KanbanColumnaDto | null>(null)

const showBoardModal = ref(false)
const editingBoard = ref<KanbanTableroDto | null>(null)

const showManageBoardsModal = ref(false)

const showDeleteColModal = ref(false)
const colToDelete = ref<KanbanColumnaDto | null>(null)

const showStrictDeleteBoardModal = ref(false)
const boardToDelete = ref<KanbanTableroDto | null>(null)

const showChecklistModal = ref(false)
const checklistCard = ref<KanbanTarjetaDto | null>(null)
const checklistItems = ref<any[]>([])

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

// -------------------------------------------------------------
// Pointer Dragging for CARDS & COLUMNS
// -------------------------------------------------------------
function onCardPointerDown(e: PointerEvent, card: KanbanTarjetaDto) {
  if (!canMove.value || e.button !== 0) return
  const target = e.target as HTMLElement | null
  if (target && target.closest('button, input, select, textarea, a')) return

  startX = e.clientX
  startY = e.clientY
  activeCard = card
  activeColumn = null
  hasMovedEnough = false
  dragType.value = null

  window.addEventListener('pointermove', onGlobalPointerMove)
  window.addEventListener('pointerup', onGlobalPointerUp)
}

function onColumnPointerDown(e: PointerEvent, col: KanbanColumnaDto) {
  if (!canManage.value || e.button !== 0) return
  const target = e.target as HTMLElement | null
  if (target && target.closest('button, input, select, textarea, a')) return

  startX = e.clientX
  startY = e.clientY
  activeColumn = col
  activeCard = null
  hasMovedEnough = false
  dragType.value = null

  window.addEventListener('pointermove', onGlobalPointerMove)
  window.addEventListener('pointerup', onGlobalPointerUp)
}

function onGlobalPointerMove(e: PointerEvent) {
  const dx = e.clientX - startX
  const dy = e.clientY - startY
  const dist = Math.sqrt(dx * dx + dy * dy)

  if (!hasMovedEnough && dist > 5) {
    hasMovedEnough = true
    if (activeCard) {
      dragType.value = 'card'
      draggingCard.value = activeCard
    } else if (activeColumn) {
      dragType.value = 'column'
      draggingColumn.value = activeColumn
    }
  }

  if (hasMovedEnough) {
    dragPosition.value = { x: e.clientX, y: e.clientY }

    const el = document.elementFromPoint(e.clientX, e.clientY)
    const colEl = el?.closest('[data-columna-id]') as HTMLElement | null
    if (colEl) {
      const colId = colEl.getAttribute('data-columna-id') as Uuid
      dragHoverColumnaId.value = colId

      if (dragType.value === 'card') {
        const cardEl = el?.closest('[data-card-id]') as HTMLElement | null
        if (cardEl && cardEl.getAttribute('data-card-id') !== activeCard?.id) {
          dragHoverCardId.value = cardEl.getAttribute('data-card-id') as Uuid
        } else {
          dragHoverCardId.value = null
        }
      }
    } else {
      dragHoverColumnaId.value = null
      dragHoverCardId.value = null
    }
  }
}

async function onGlobalPointerUp(_e: PointerEvent) {
  window.removeEventListener('pointermove', onGlobalPointerMove)
  window.removeEventListener('pointerup', onGlobalPointerUp)

  if (hasMovedEnough) {
    // 1. Dropping a CARD
    if (dragType.value === 'card' && draggingCard.value && dragHoverColumnaId.value) {
      const card = draggingCard.value
      const targetColId = dragHoverColumnaId.value
      const targetCardId = dragHoverCardId.value
      const origenColumnaId = card.columnaId

      const colCards = getTarjetasPorColumna(targetColId).filter((c) => c.id !== card.id)

      let nuevoOrden = colCards.length
      if (targetCardId) {
        const targetIdx = colCards.findIndex((c) => c.id === targetCardId)
        if (targetIdx !== -1) {
          colCards.splice(targetIdx, 0, card)
          nuevoOrden = targetIdx
        } else {
          colCards.push(card)
        }
      } else {
        colCards.push(card)
      }

      const tarjetaIdsEnDestino = colCards.map((c) => c.id)

      try {
        await store.reordenarTarjetas({
          tarjetaId: card.id,
          origenColumnaId,
          destinoColumnaId: targetColId,
          nuevoOrden,
          tarjetaIdsEnDestino,
        })
      } catch (err: unknown) {
        alert(err instanceof Error ? err.message : 'Error al mover tarjeta')
      }
    }

    // 2. Dropping a COLUMN
    if (dragType.value === 'column' && draggingColumn.value && dragHoverColumnaId.value && store.currentTableroId) {
      const sourceCol = draggingColumn.value
      const targetColId = dragHoverColumnaId.value

      if (sourceCol.id !== targetColId) {
        const cols = [...sortedColumnas.value]
        const fromIdx = cols.findIndex((c) => c.id === sourceCol.id)
        const toIdx = cols.findIndex((c) => c.id === targetColId)

        if (fromIdx !== -1 && toIdx !== -1) {
          const [moved] = cols.splice(fromIdx, 1)
          if (moved) {
            cols.splice(toIdx, 0, moved)
            const newColumnaIds = cols.map((c) => c.id)
            try {
              await store.reordenarColumnas({
                tableroId: store.currentTableroId,
                columnaIds: newColumnaIds,
              })
            } catch (err: unknown) {
              alert(err instanceof Error ? err.message : 'Error al reordenar columnas')
            }
          }
        }
      }
    }
  } else if (activeCard) {
    openEditCard(activeCard)
  }

  activeCard = null
  activeColumn = null
  draggingCard.value = null
  draggingColumn.value = null
  dragHoverColumnaId.value = null
  dragHoverCardId.value = null
  hasMovedEnough = false
  dragType.value = null
}

// -------------------------------------------------------------
// Quick Button Column reordering
// -------------------------------------------------------------
async function moverColumna(col: KanbanColumnaDto, direccion: 'izq' | 'der') {
  if (!canManage.value || !store.detalle || !store.currentTableroId) return
  const cols = [...sortedColumnas.value]
  const idx = cols.findIndex((c) => c.id === col.id)
  if (idx === -1) return

  const targetIdx = direccion === 'izq' ? idx - 1 : idx + 1
  if (targetIdx < 0 || targetIdx >= cols.length) return

  const [moved] = cols.splice(idx, 1)
  if (!moved) return
  cols.splice(targetIdx, 0, moved)

  const newColumnaIds = cols.map((c) => c.id)

  try {
    await store.reordenarColumnas({
      tableroId: store.currentTableroId,
      columnaIds: newColumnaIds,
    })
  } catch (err: unknown) {
    alert(err instanceof Error ? err.message : 'Error al reordenar columnas')
  }
}

// -------------------------------------------------------------
// Card CRUD Handlers
// -------------------------------------------------------------
function openCreateCard(columnaId: Uuid) {
  editingCard.value = null
  cardFormColumnaId.value = columnaId
  showCardModal.value = true
}

function openEditCard(card: KanbanTarjetaDto) {
  editingCard.value = card
  cardFormColumnaId.value = card.columnaId
  showCardModal.value = true
}

async function handleSaveCard(data: {
  titulo: string
  descripcion: string | null
  prioridad: PrioridadTarjeta
  fechaVencimiento: string | null
  etiquetaIds: Uuid[]
}) {
  try {
    if (editingCard.value) {
      await store.updateTarjeta(editingCard.value.id, {
        ...data,
        rowVersion: editingCard.value.rowVersion,
      })
    } else {
      await store.createTarjeta({
        columnaId: cardFormColumnaId.value,
        ...data,
      })
    }
    showCardModal.value = false
  } catch (err: unknown) {
    alert(err instanceof Error ? err.message : 'Error al guardar tarjeta')
  }
}

async function removeCard(card: KanbanTarjetaDto) {
  if (!confirm(t('Kanban.ConfirmDeleteCard'))) return
  try {
    await store.deleteTarjeta(card.id, card.rowVersion)
  } catch (err: unknown) {
    alert(err instanceof Error ? err.message : 'Error al eliminar tarjeta')
  }
}

// -------------------------------------------------------------
// Column CRUD Handlers
// -------------------------------------------------------------
function openCreateColumn() {
  editingColumn.value = null
  showColumnModal.value = true
}

function openEditColumn(col: KanbanColumnaDto) {
  editingColumn.value = col
  showColumnModal.value = true
}

async function handleSaveColumn(data: {
  nombre: string
  color: string | null
  orden: number
  limiteWip: number | null
}) {
  if (!store.currentTableroId) return
  try {
    if (editingColumn.value) {
      await store.updateColumna(editingColumn.value.id, {
        ...data,
        rowVersion: editingColumn.value.rowVersion,
      })
    } else {
      await store.createColumna({
        tableroId: store.currentTableroId,
        ...data,
      })
    }
    showColumnModal.value = false
  } catch (err: unknown) {
    alert(err instanceof Error ? err.message : 'Error al guardar columna')
  }
}

function confirmDeleteColumna(col: KanbanColumnaDto) {
  colToDelete.value = col
  showDeleteColModal.value = true
}

async function executeDeleteColumn() {
  if (!colToDelete.value) return
  const col = colToDelete.value
  showDeleteColModal.value = false
  try {
    await store.deleteColumna(col.id, col.rowVersion)
  } catch (err: unknown) {
    alert(err instanceof Error ? err.message : 'Error al eliminar columna')
  }
}

// -------------------------------------------------------------
// Board CRUD Handlers
// -------------------------------------------------------------
function openCreateBoard() {
  editingBoard.value = null
  showBoardModal.value = true
}

function openEditBoard(b: KanbanTableroDto) {
  editingBoard.value = b
  showBoardModal.value = true
}

async function handleSaveBoard(data: {
  nombre: string
  descripcion: string | null
  color: string | null
}) {
  try {
    if (editingBoard.value) {
      await store.updateTablero(editingBoard.value.id, {
        ...data,
        activo: editingBoard.value.activo,
        rowVersion: editingBoard.value.rowVersion,
      })
    } else {
      await store.createTablero(data)
    }
    showBoardModal.value = false
  } catch (err: unknown) {
    alert(err instanceof Error ? err.message : 'Error al guardar tablero')
  }
}

async function handleDeleteBoardPrompt(board: KanbanTableroDto) {
  if (board.esPreset) {
    alert('Los tableros presets del sistema no se pueden eliminar. Puedes ocultarlo usando la opción Ocultar.')
    return
  }

  let cardCount = 0
  if (store.currentTableroId === board.id && store.detalle) {
    cardCount = store.detalle.tarjetas.length
  }

  if (cardCount === 0) {
    try {
      await store.deleteTablero(board.id, board.rowVersion)
    } catch (err: unknown) {
      alert(err instanceof Error ? err.message : 'Error al eliminar tablero')
    }
    return
  }

  boardToDelete.value = board
  showStrictDeleteBoardModal.value = true
}

async function executeStrictDeleteBoard() {
  if (!boardToDelete.value) return
  const b = boardToDelete.value
  showStrictDeleteBoardModal.value = false
  try {
    await store.deleteTablero(b.id, b.rowVersion)
  } catch (err: unknown) {
    alert(err instanceof Error ? err.message : 'Error al eliminar tablero')
  }
}

// -------------------------------------------------------------
// Checklist Handlers
// -------------------------------------------------------------
async function openChecklist(card: KanbanTarjetaDto) {
  checklistCard.value = card
  checklistItems.value = await store.listChecklist(card.id)
  showChecklistModal.value = true
}

async function handleAddChecklist(titulo: string) {
  if (!checklistCard.value) return
  try {
    const item = await store.addChecklistItem({
      tarjetaId: checklistCard.value.id,
      titulo,
    })
    checklistItems.value.push(item)
  } catch (err: unknown) {
    alert(err instanceof Error ? err.message : 'Error al agregar item al checklist')
  }
}

async function handleToggleChecklist(item: any) {
  item.completada = !item.completada
  try {
    await store.updateChecklistItem(item.id, {
      titulo: item.titulo,
      completada: item.completada,
      orden: item.orden,
      rowVersion: item.rowVersion,
    })
  } catch (err: unknown) {
    item.completada = !item.completada
    alert(err instanceof Error ? err.message : 'Error al actualizar checklist')
  }
}

async function handleRemoveChecklist(item: any) {
  if (!checklistCard.value) return
  try {
    await store.deleteChecklistItem(item.id, checklistCard.value.id, item.completada)
    checklistItems.value = checklistItems.value.filter((x) => x.id !== item.id)
  } catch (err: unknown) {
    alert(err instanceof Error ? err.message : 'Error al eliminar checklist')
  }
}
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
