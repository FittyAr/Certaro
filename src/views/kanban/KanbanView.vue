<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  useKanbanStore,
  type KanbanColumnaDto,
  type KanbanTarjetaDto,
  type PrioridadTarjeta,
  type Uuid,
} from '@/stores/useKanbanStore'
import { usePermission } from '@/composables/usePermission'
import { KANBAN_PRESET_COLORS } from '@/lib/kanbanColors'

const { t } = useI18n()
const store = useKanbanStore()
const { can } = usePermission()

const canManage = computed(() => can('kanban:gestionar_tablero'))
const canCreate = computed(() => can('kanban:crear_tarjeta'))
const canMove = computed(() => can('kanban:mover_tarjeta'))

// Filters
const searchText = ref('')
const selectedPrioridad = ref<string>('all')

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

// Modals
const showCardModal = ref(false)
const editingCard = ref<KanbanTarjetaDto | null>(null)
const cardFormColumnaId = ref<Uuid>('')
const cardFormTitulo = ref('')
const cardFormDescripcion = ref('')
const cardFormPrioridad = ref<PrioridadTarjeta>('Normal')
const cardFormFechaVencimiento = ref('')
const cardFormEtiquetas = ref<Uuid[]>([])

const showColumnModal = ref(false)
const editingColumn = ref<KanbanColumnaDto | null>(null)
const columnFormNombre = ref('')
const columnFormColor = ref('')
const columnFormOrden = ref<number>(0)
const columnFormLimiteWip = ref<number | null>(null)

const showBoardModal = ref(false)
const boardFormNombre = ref('')
const boardFormDescripcion = ref('')
const boardFormColor = ref('')

const showChecklistModal = ref(false)
const checklistCard = ref<KanbanTarjetaDto | null>(null)
const checklistItems = ref<any[]>([])
const newChecklistTitle = ref('')

onMounted(async () => {
  await store.fetchTableros()
})

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
// Pointer Dragging for CARDS
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

// -------------------------------------------------------------
// Pointer Dragging for COLUMNS
// -------------------------------------------------------------
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
    // It was a simple click: open edit modal
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
// Card CRUD
// -------------------------------------------------------------
function openCreateCard(columnaId: Uuid) {
  editingCard.value = null
  cardFormColumnaId.value = columnaId
  cardFormTitulo.value = ''
  cardFormDescripcion.value = ''
  cardFormPrioridad.value = 'Normal'
  cardFormFechaVencimiento.value = ''
  cardFormEtiquetas.value = []
  showCardModal.value = true
}

function openEditCard(card: KanbanTarjetaDto) {
  editingCard.value = card
  cardFormColumnaId.value = card.columnaId
  cardFormTitulo.value = card.titulo
  cardFormDescripcion.value = card.descripcion ?? ''
  cardFormPrioridad.value = card.prioridad
  cardFormFechaVencimiento.value = card.fechaVencimiento ?? ''
  cardFormEtiquetas.value = card.etiquetas.map((e) => e.id)
  showCardModal.value = true
}

async function saveCard() {
  if (!cardFormTitulo.value.trim()) return

  try {
    if (editingCard.value) {
      await store.updateTarjeta(editingCard.value.id, {
        titulo: cardFormTitulo.value.trim(),
        descripcion: cardFormDescripcion.value.trim() || null,
        prioridad: cardFormPrioridad.value,
        fechaVencimiento: cardFormFechaVencimiento.value || null,
        etiquetaIds: cardFormEtiquetas.value,
        rowVersion: editingCard.value.rowVersion,
      })
    } else {
      await store.createTarjeta({
        columnaId: cardFormColumnaId.value,
        titulo: cardFormTitulo.value.trim(),
        descripcion: cardFormDescripcion.value.trim() || null,
        prioridad: cardFormPrioridad.value,
        fechaVencimiento: cardFormFechaVencimiento.value || null,
        etiquetaIds: cardFormEtiquetas.value,
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
// Column CRUD
// -------------------------------------------------------------
function openCreateColumn() {
  editingColumn.value = null
  columnFormNombre.value = ''
  columnFormColor.value = ''
  columnFormOrden.value = store.detalle?.columnas.length ?? 0
  columnFormLimiteWip.value = null
  showColumnModal.value = true
}

function openEditColumn(col: KanbanColumnaDto) {
  editingColumn.value = col
  columnFormNombre.value = col.nombre
  columnFormColor.value = col.color ?? ''
  columnFormOrden.value = col.orden
  columnFormLimiteWip.value = col.limiteWip
  showColumnModal.value = true
}

async function saveColumn() {
  if (!columnFormNombre.value.trim() || !store.currentTableroId) return

  try {
    if (editingColumn.value) {
      await store.updateColumna(editingColumn.value.id, {
        nombre: columnFormNombre.value.trim(),
        color: columnFormColor.value || null,
        orden: columnFormOrden.value ?? editingColumn.value.orden,
        limiteWip: columnFormLimiteWip.value,
        rowVersion: editingColumn.value.rowVersion,
      })
    } else {
      await store.createColumna({
        tableroId: store.currentTableroId,
        nombre: columnFormNombre.value.trim(),
        color: columnFormColor.value || null,
        limiteWip: columnFormLimiteWip.value,
      })
    }
    showColumnModal.value = false
  } catch (err: unknown) {
    alert(err instanceof Error ? err.message : 'Error al guardar columna')
  }
}

async function removeColumn(col: KanbanColumnaDto) {
  if (!confirm(t('Kanban.ConfirmDeleteColumn'))) return
  try {
    await store.deleteColumna(col.id, col.rowVersion)
  } catch (err: unknown) {
    alert(err instanceof Error ? err.message : 'Error al eliminar columna')
  }
}

// -------------------------------------------------------------
// Board CRUD
// -------------------------------------------------------------
function openCreateBoard() {
  boardFormNombre.value = ''
  boardFormDescripcion.value = ''
  boardFormColor.value = ''
  showBoardModal.value = true
}

async function saveBoard() {
  if (!boardFormNombre.value.trim()) return
  try {
    await store.createTablero({
      nombre: boardFormNombre.value.trim(),
      descripcion: boardFormDescripcion.value.trim() || null,
      color: boardFormColor.value || null,
    })
    showBoardModal.value = false
  } catch (err: unknown) {
    alert(err instanceof Error ? err.message : 'Error al crear tablero')
  }
}

// -------------------------------------------------------------
// Checklist modal
// -------------------------------------------------------------
async function openChecklist(card: KanbanTarjetaDto) {
  checklistCard.value = card
  checklistItems.value = await store.listChecklist(card.id)
  newChecklistTitle.value = ''
  showChecklistModal.value = true
}

async function addChecklist() {
  if (!newChecklistTitle.value.trim() || !checklistCard.value) return
  try {
    const item = await store.addChecklistItem({
      tarjetaId: checklistCard.value.id,
      titulo: newChecklistTitle.value.trim(),
    })
    checklistItems.value.push(item)
    newChecklistTitle.value = ''
  } catch (err: unknown) {
    alert(err instanceof Error ? err.message : 'Error al agregar item al checklist')
  }
}

async function toggleChecklist(item: any) {
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

async function removeChecklist(item: any) {
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

    <!-- Top Header: Board Switcher & Actions -->
    <div class="flex flex-wrap items-center justify-between gap-3 border-b border-border pb-3">
      <div class="flex items-center gap-2 overflow-x-auto pb-1 max-w-full">
        <button
          v-for="b in store.activeTableros"
          :key="b.id"
          class="px-3.5 py-1.5 rounded-md text-sm font-medium transition-colors whitespace-nowrap flex items-center gap-1.5"
          :class="
            store.currentTableroId === b.id
              ? 'bg-primary text-primary-foreground shadow-xs'
              : 'bg-surface-card hover:bg-muted text-muted-foreground border border-border'
          "
          @click="store.selectTablero(b.id)"
        >
          <span>{{ b.nombre }}</span>
          <span
            v-if="b.esPreset"
            class="text-[10px] px-1.5 py-0.2 rounded font-mono font-semibold"
            :class="
              store.currentTableroId === b.id
                ? 'bg-primary-foreground/20 text-primary-foreground'
                : 'bg-muted text-muted-foreground'
            "
          >
            PRESET
          </span>
        </button>

        <button
          v-if="canManage"
          class="px-2.5 py-1.5 rounded-md text-sm font-medium border border-dashed border-border hover:bg-muted text-muted-foreground"
          :title="t('Kanban.NewBoard')"
          @click="openCreateBoard"
        >
          + {{ t('Kanban.NewBoard') }}
        </button>
      </div>

      <div class="flex items-center gap-2">
        <button
          v-if="store.currentTablero?.esPreset"
          class="px-3 py-1.5 rounded-md text-xs font-medium border border-border hover:bg-muted text-foreground flex items-center gap-1.5"
          @click="store.syncPreset(store.currentTablero.id)"
        >
          <span>↻</span>
          <span>{{ t('Kanban.Sync') }}</span>
        </button>

        <button
          v-if="canManage && store.currentTableroId"
          class="px-3 py-1.5 rounded-md text-xs font-medium border border-border hover:bg-muted text-foreground"
          @click="openCreateColumn"
        >
          + {{ t('Kanban.NewColumn') }}
        </button>
      </div>
    </div>

    <!-- Filter Bar -->
    <div class="flex flex-wrap items-center gap-3 bg-surface-card border border-border p-2.5 rounded-lg">
      <div class="relative flex-1 min-w-50">
        <input
          v-model="searchText"
          type="text"
          :placeholder="t('Kanban.SearchCards')"
          class="w-full px-3 py-1.5 text-xs rounded-md bg-background border border-border text-foreground placeholder:text-muted-foreground focus:outline-hidden focus:ring-1 focus:ring-primary"
        />
      </div>

      <div class="flex items-center gap-1.5 text-xs text-muted-foreground">
        <span>{{ t('Kanban.Priority') }}:</span>
        <select
          v-model="selectedPrioridad"
          class="px-2.5 py-1 text-xs rounded-md bg-background border border-border text-foreground focus:outline-hidden"
        >
          <option value="all">{{ t('Kanban.All') }}</option>
          <option value="Baja">{{ t('Kanban.PriorityLow') }}</option>
          <option value="Normal">{{ t('Kanban.PriorityNormal') }}</option>
          <option value="Alta">{{ t('Kanban.PriorityHigh') }}</option>
          <option value="Urgente">{{ t('Kanban.PriorityUrgent') }}</option>
        </select>
      </div>
    </div>

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
      <div
        v-for="(col, index) in sortedColumnas"
        :key="col.id"
        :data-columna-id="col.id"
        :class="[
          'w-80 shrink-0 flex flex-col max-h-full rounded-xl bg-surface-card border transition-all duration-150 shadow-xs overflow-hidden',
          dragHoverColumnaId === col.id ? 'border-primary ring-2 ring-primary/40 bg-primary/5' : 'border-border',
          draggingColumn?.id === col.id ? 'opacity-30 border-dashed' : ''
        ]"
        :style="{
          borderTopColor: col.color || 'var(--color-primary, currentColor)',
          borderTopWidth: '4px'
        }"
      >
        <!-- Column Header: Click & drag to reorder columns -->
        <div
          class="p-3 border-b border-border flex items-center justify-between gap-2 bg-muted/20 select-none"
          :class="canManage ? 'cursor-grab active:cursor-grabbing' : ''"
          @pointerdown="onColumnPointerDown($event, col)"
        >
          <div class="flex items-center gap-2 min-w-0">
            <!-- Drag Handle icon for column -->
            <span v-if="canManage" class="text-muted-foreground text-xs leading-none">⋮⋮</span>
            <!-- Column colored dot indicator -->
            <span
              class="w-3 h-3 rounded-full shrink-0 border border-border shadow-xs"
              :style="{ backgroundColor: col.color || 'var(--color-primary, currentColor)' }"
            />
            <h3 class="text-sm font-semibold text-foreground truncate" :title="col.nombre">
              {{ col.nombre }}
            </h3>
            <span class="text-xs text-muted-foreground font-mono bg-muted px-1.5 py-0.5 rounded">
              {{ getTarjetasPorColumna(col.id).length }}
              <template v-if="col.limiteWip"> / {{ col.limiteWip }}</template>
            </span>
          </div>

          <div class="flex items-center gap-1">
            <!-- Reorder column buttons -->
            <div v-if="canManage && sortedColumnas.length > 1" class="flex items-center">
              <button
                v-if="index > 0"
                type="button"
                class="p-1 rounded-sm hover:bg-muted text-muted-foreground hover:text-foreground text-[10px] leading-none"
                title="Mover columna a la izquierda"
                @click.stop="moverColumna(col, 'izq')"
              >
                ◀
              </button>
              <button
                v-if="index < sortedColumnas.length - 1"
                type="button"
                class="p-1 rounded-sm hover:bg-muted text-muted-foreground hover:text-foreground text-[10px] leading-none"
                title="Mover columna a la derecha"
                @click.stop="moverColumna(col, 'der')"
              >
                ▶
              </button>
            </div>

            <button
              v-if="canCreate"
              class="p-1 rounded-sm hover:bg-muted text-muted-foreground hover:text-foreground text-sm"
              :title="t('Kanban.NewCard')"
              @click.stop="openCreateCard(col.id)"
            >
              +
            </button>
            <button
              v-if="canManage"
              class="p-1 rounded-sm hover:bg-muted text-muted-foreground hover:text-foreground text-xs"
              :title="t('Kanban.EditColumn')"
              @click.stop="openEditColumn(col)"
            >
              ✎
            </button>
            <button
              v-if="canManage && !store.currentTablero?.esPreset"
              class="p-1 rounded-sm hover:bg-muted text-destructive text-xs"
              :title="t('Kanban.DeleteColumn')"
              @click.stop="removeColumn(col)"
            >
              ✕
            </button>
          </div>
        </div>

        <!-- Cards Container -->
        <div class="flex-1 overflow-y-auto p-2.5 flex flex-col gap-2.5 min-h-30">
          <div
            v-for="card in getTarjetasPorColumna(col.id)"
            :key="card.id"
            :data-card-id="card.id"
            class="p-3 rounded-lg bg-surface-elevated border shadow-xs hover:border-primary/50 transition-all cursor-grab active:cursor-grabbing flex flex-col gap-2 select-none"
            :class="[
              dragHoverCardId === card.id ? 'border-primary ring-2 ring-primary/50' : 'border-border',
              draggingCard?.id === card.id ? 'opacity-30 border-dashed' : ''
            ]"
            @pointerdown="onCardPointerDown($event, card)"
          >
            <!-- Card Meta: Priority & Tags -->
            <div class="flex flex-wrap items-center justify-between gap-1.5">
              <span
                class="text-[10px] uppercase font-bold px-1.5 py-0.5 rounded border"
                :class="getPriorityClass(card.prioridad)"
              >
                {{ card.prioridad }}
              </span>

              <div class="flex flex-wrap gap-1">
                <span
                  v-for="tag in card.etiquetas"
                  :key="tag.id"
                  class="text-[10px] px-1.5 py-0.5 rounded font-medium border border-border bg-muted text-foreground"
                >
                  {{ tag.nombre }}
                </span>
              </div>
            </div>

            <!-- Title & Description -->
            <div>
              <h4 class="text-xs font-semibold text-foreground leading-snug">
                {{ card.titulo }}
              </h4>
              <p
                v-if="card.descripcion"
                class="text-[11px] text-muted-foreground line-clamp-2 mt-1"
              >
                {{ card.descripcion }}
              </p>
            </div>

            <!-- Footer: Due date, Checklist & Actions -->
            <div class="flex items-center justify-between border-t border-border pt-2 text-[11px] text-muted-foreground">
              <div class="flex items-center gap-2">
                <span v-if="card.fechaVencimiento" class="flex items-center gap-1 font-mono">
                  📅 {{ card.fechaVencimiento }}
                </span>
                <button
                  type="button"
                  class="hover:text-foreground flex items-center gap-1"
                  @click.stop="openChecklist(card)"
                >
                  ☑ {{ card.completadasChecklist }}/{{ card.totalChecklist }}
                </button>
              </div>

              <div class="flex items-center gap-1">
                <button
                  type="button"
                  class="p-1 rounded-sm hover:bg-muted text-muted-foreground hover:text-foreground"
                  :title="t('General.Edit')"
                  @click.stop="openEditCard(card)"
                >
                  ✎
                </button>
                <button
                  type="button"
                  class="p-1 rounded-sm hover:bg-muted text-destructive"
                  :title="t('General.Delete')"
                  @click.stop="removeCard(card)"
                >
                  ✕
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Modal: Card Create / Edit -->
    <div
      v-if="showCardModal"
      class="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-xs p-4"
    >
      <div class="w-full max-w-md rounded-xl bg-surface-card border border-border p-5 shadow-lg flex flex-col gap-4">
        <div class="flex items-center justify-between border-b border-border pb-2">
          <h3 class="text-sm font-semibold text-foreground">
            {{ editingCard ? t('Kanban.EditCard') : t('Kanban.NewCard') }}
          </h3>
          <button type="button" class="text-muted-foreground hover:text-foreground" @click="showCardModal = false">✕</button>
        </div>

        <div class="flex flex-col gap-3 text-xs">
          <div>
            <label class="block font-medium text-foreground mb-1">{{ t('Kanban.Title') }} *</label>
            <input
              v-model="cardFormTitulo"
              type="text"
              class="w-full px-3 py-1.5 rounded-md bg-background border border-border text-foreground focus:outline-hidden focus:ring-1 focus:ring-primary"
            />
          </div>

          <div>
            <label class="block font-medium text-foreground mb-1">{{ t('Kanban.Description') }}</label>
            <textarea
              v-model="cardFormDescripcion"
              rows="3"
              class="w-full px-3 py-1.5 rounded-md bg-background border border-border text-foreground focus:outline-hidden focus:ring-1 focus:ring-primary"
            />
          </div>

          <div class="grid grid-cols-2 gap-2">
            <div>
              <label class="block font-medium text-foreground mb-1">{{ t('Kanban.Priority') }}</label>
              <select
                v-model="cardFormPrioridad"
                class="w-full px-2.5 py-1.5 rounded-md bg-background border border-border text-foreground focus:outline-hidden"
              >
                <option value="Baja">{{ t('Kanban.PriorityLow') }}</option>
                <option value="Normal">{{ t('Kanban.PriorityNormal') }}</option>
                <option value="Alta">{{ t('Kanban.PriorityHigh') }}</option>
                <option value="Urgente">{{ t('Kanban.PriorityUrgent') }}</option>
              </select>
            </div>

            <div>
              <label class="block font-medium text-foreground mb-1">{{ t('Kanban.DueDate') }}</label>
              <input
                v-model="cardFormFechaVencimiento"
                type="date"
                class="w-full px-2.5 py-1.5 rounded-md bg-background border border-border text-foreground focus:outline-hidden"
              />
            </div>
          </div>
        </div>

        <div class="flex justify-end gap-2 border-t border-border pt-3">
          <button
            type="button"
            class="px-3 py-1.5 rounded-md text-xs font-medium border border-border hover:bg-muted text-muted-foreground"
            @click="showCardModal = false"
          >
            {{ t('General.Cancel') }}
          </button>
          <button
            type="button"
            class="px-4 py-1.5 rounded-md text-xs font-medium bg-primary text-primary-foreground hover:bg-primary/90"
            @click="saveCard"
          >
            {{ t('General.Save') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Modal: Column Create / Edit -->
    <div
      v-if="showColumnModal"
      class="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-xs p-4"
    >
      <div class="w-full max-w-sm rounded-xl bg-surface-card border border-border p-5 shadow-lg flex flex-col gap-4">
        <div class="flex items-center justify-between border-b border-border pb-2">
          <h3 class="text-sm font-semibold text-foreground">
            {{ editingColumn ? t('Kanban.EditColumn') : t('Kanban.NewColumn') }}
          </h3>
          <button type="button" class="text-muted-foreground hover:text-foreground" @click="showColumnModal = false">✕</button>
        </div>

        <div class="flex flex-col gap-3 text-xs">
          <div>
            <label class="block font-medium text-foreground mb-1">{{ t('Kanban.ColumnName') }} *</label>
            <input
              v-model="columnFormNombre"
              type="text"
              class="w-full px-3 py-1.5 rounded-md bg-background border border-border text-foreground focus:outline-hidden focus:ring-1 focus:ring-primary"
            />
          </div>

          <!-- Color selection with picker and palette swatches -->
          <div>
            <label class="block font-medium text-foreground mb-1">{{ t('Kanban.Color') }}</label>
            <div class="flex items-center gap-2">
              <input
                v-model="columnFormColor"
                type="color"
                class="w-10 h-8 p-1 rounded-md bg-background border border-border cursor-pointer"
              />
              <span class="text-xs font-mono text-muted-foreground">{{ columnFormColor || 'Predeterminado' }}</span>
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
                @click="columnFormColor = c"
              />
            </div>
          </div>

          <div class="grid grid-cols-2 gap-2">
            <div>
              <label class="block font-medium text-foreground mb-1">Orden</label>
              <input
                v-model.number="columnFormOrden"
                type="number"
                min="0"
                class="w-full px-3 py-1.5 rounded-md bg-background border border-border text-foreground focus:outline-hidden"
              />
            </div>
            <div>
              <label class="block font-medium text-foreground mb-1">{{ t('Kanban.WipLimit') }}</label>
              <input
                v-model.number="columnFormLimiteWip"
                type="number"
                min="1"
                class="w-full px-3 py-1.5 rounded-md bg-background border border-border text-foreground focus:outline-hidden"
              />
            </div>
          </div>
        </div>

        <div class="flex justify-end gap-2 border-t border-border pt-3">
          <button
            type="button"
            class="px-3 py-1.5 rounded-md text-xs font-medium border border-border hover:bg-muted text-muted-foreground"
            @click="showColumnModal = false"
          >
            {{ t('General.Cancel') }}
          </button>
          <button
            type="button"
            class="px-4 py-1.5 rounded-md text-xs font-medium bg-primary text-primary-foreground hover:bg-primary/90"
            @click="saveColumn"
          >
            {{ t('General.Save') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Modal: Board Create -->
    <div
      v-if="showBoardModal"
      class="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-xs p-4"
    >
      <div class="w-full max-w-sm rounded-xl bg-surface-card border border-border p-5 shadow-lg flex flex-col gap-4">
        <div class="flex items-center justify-between border-b border-border pb-2">
          <h3 class="text-sm font-semibold text-foreground">{{ t('Kanban.NewBoard') }}</h3>
          <button type="button" class="text-muted-foreground hover:text-foreground" @click="showBoardModal = false">✕</button>
        </div>

        <div class="flex flex-col gap-3 text-xs">
          <div>
            <label class="block font-medium text-foreground mb-1">{{ t('Kanban.BoardName') }} *</label>
            <input
              v-model="boardFormNombre"
              type="text"
              class="w-full px-3 py-1.5 rounded-md bg-background border border-border text-foreground focus:outline-hidden focus:ring-1 focus:ring-primary"
            />
          </div>

          <div>
            <label class="block font-medium text-foreground mb-1">{{ t('Kanban.Description') }}</label>
            <textarea
              v-model="boardFormDescripcion"
              rows="2"
              class="w-full px-3 py-1.5 rounded-md bg-background border border-border text-foreground focus:outline-hidden"
            />
          </div>

          <div>
            <label class="block font-medium text-foreground mb-1">{{ t('Kanban.Color') }}</label>
            <input
              v-model="boardFormColor"
              type="color"
              class="w-full h-8 p-1 rounded-md bg-background border border-border cursor-pointer"
            />
          </div>
        </div>

        <div class="flex justify-end gap-2 border-t border-border pt-3">
          <button
            type="button"
            class="px-3 py-1.5 rounded-md text-xs font-medium border border-border hover:bg-muted text-muted-foreground"
            @click="showBoardModal = false"
          >
            {{ t('General.Cancel') }}
          </button>
          <button
            type="button"
            class="px-4 py-1.5 rounded-md text-xs font-medium bg-primary text-primary-foreground hover:bg-primary/90"
            @click="saveBoard"
          >
            {{ t('General.Save') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Modal: Checklist -->
    <div
      v-if="showChecklistModal"
      class="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-xs p-4"
    >
      <div class="w-full max-w-sm rounded-xl bg-surface-card border border-border p-5 shadow-lg flex flex-col gap-4">
        <div class="flex items-center justify-between border-b border-border pb-2">
          <h3 class="text-sm font-semibold text-foreground">
            {{ t('Kanban.Checklist') }}: {{ checklistCard?.titulo }}
          </h3>
          <button type="button" class="text-muted-foreground hover:text-foreground" @click="showChecklistModal = false">✕</button>
        </div>

        <div class="flex flex-col gap-2 max-h-60 overflow-y-auto">
          <div
            v-for="item in checklistItems"
            :key="item.id"
            class="flex items-center justify-between gap-2 p-1.5 rounded hover:bg-muted/50 text-xs"
          >
            <label class="flex items-center gap-2 cursor-pointer flex-1 min-w-0">
              <input
                type="checkbox"
                :checked="item.completada"
                class="rounded border-border text-primary"
                @change="toggleChecklist(item)"
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
              @click="removeChecklist(item)"
            >
              ✕
            </button>
          </div>
          <div v-if="checklistItems.length === 0" class="text-xs text-muted-foreground py-2 text-center">
            {{ t('Kanban.EmptyChecklist') }}
          </div>
        </div>

        <div class="flex gap-2 border-t border-border pt-3">
          <input
            v-model="newChecklistTitle"
            type="text"
            :placeholder="t('Kanban.NewChecklistItem')"
            class="flex-1 px-2.5 py-1 text-xs rounded-md bg-background border border-border text-foreground focus:outline-hidden"
            @keyup.enter="addChecklist"
          />
          <button
            type="button"
            class="px-3 py-1 rounded-md text-xs font-medium bg-primary text-primary-foreground hover:bg-primary/90"
            @click="addChecklist"
          >
            {{ t('Kanban.Add') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
