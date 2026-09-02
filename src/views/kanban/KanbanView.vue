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

const { t } = useI18n()
const store = useKanbanStore()
const { can } = usePermission()

const canManage = computed(() => can('kanban:gestionar_tablero'))
const canCreate = computed(() => can('kanban:crear_tarjeta'))
const canMove = computed(() => can('kanban:mover_tarjeta'))

// Filters
const searchText = ref('')
const selectedPrioridad = ref<string>('all')
const draggedCard = ref<KanbanTarjetaDto | null>(null)

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

// Drag & Drop
function onDragStart(e: DragEvent, card: KanbanTarjetaDto) {
  if (!canMove.value) return
  draggedCard.value = card
  if (e.dataTransfer) {
    e.dataTransfer.effectAllowed = 'move'
    e.dataTransfer.setData('text/plain', card.id)
  }
}

async function onDrop(_e: DragEvent, targetColumnaId: Uuid) {
  if (!draggedCard.value || !canMove.value) return
  const card = draggedCard.value
  draggedCard.value = null

  if (card.columnaId === targetColumnaId) return

  const targetCards = getTarjetasPorColumna(targetColumnaId)
  const nuevoOrden = targetCards.length

  try {
    await store.moverTarjeta({
      tarjetaId: card.id,
      nuevaColumnaId: targetColumnaId,
      nuevoOrden,
      rowVersion: card.rowVersion,
    })
  } catch (_e) {
    // handled by store
  }
}

// Card CRUD
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
}

async function removeCard(card: KanbanTarjetaDto) {
  if (!confirm(t('Kanban.ConfirmDeleteCard'))) return
  await store.deleteTarjeta(card.id, card.rowVersion)
}

// Column CRUD
function openCreateColumn() {
  editingColumn.value = null
  columnFormNombre.value = ''
  columnFormColor.value = ''
  columnFormLimiteWip.value = null
  showColumnModal.value = true
}

function openEditColumn(col: KanbanColumnaDto) {
  editingColumn.value = col
  columnFormNombre.value = col.nombre
  columnFormColor.value = col.color ?? ''
  columnFormLimiteWip.value = col.limiteWip
  showColumnModal.value = true
}

async function saveColumn() {
  if (!columnFormNombre.value.trim() || !store.currentTableroId) return

  if (editingColumn.value) {
    await store.updateColumna(editingColumn.value.id, {
      nombre: columnFormNombre.value.trim(),
      color: columnFormColor.value || null,
      orden: editingColumn.value.orden,
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
}

async function removeColumn(col: KanbanColumnaDto) {
  if (!confirm(t('Kanban.ConfirmDeleteColumn'))) return
  await store.deleteColumna(col.id, col.rowVersion)
}

// Board CRUD
function openCreateBoard() {
  boardFormNombre.value = ''
  boardFormDescripcion.value = ''
  boardFormColor.value = ''
  showBoardModal.value = true
}

async function saveBoard() {
  if (!boardFormNombre.value.trim()) return
  await store.createTablero({
    nombre: boardFormNombre.value.trim(),
    descripcion: boardFormDescripcion.value.trim() || null,
    color: boardFormColor.value || null,
  })
  showBoardModal.value = false
}

// Checklist modal
async function openChecklist(card: KanbanTarjetaDto) {
  checklistCard.value = card
  checklistItems.value = await store.listChecklist(card.id)
  newChecklistTitle.value = ''
  showChecklistModal.value = true
}

async function addChecklist() {
  if (!newChecklistTitle.value.trim() || !checklistCard.value) return
  const item = await store.addChecklistItem({
    tarjetaId: checklistCard.value.id,
    titulo: newChecklistTitle.value.trim(),
  })
  checklistItems.value.push(item)
  newChecklistTitle.value = ''
}

async function toggleChecklist(item: any) {
  item.completada = !item.completada
  await store.updateChecklistItem(item.id, {
    titulo: item.titulo,
    completada: item.completada,
    orden: item.orden,
    rowVersion: item.rowVersion,
  })
}

async function removeChecklist(item: any) {
  if (!checklistCard.value) return
  await store.deleteChecklistItem(item.id, checklistCard.value.id, item.completada)
  checklistItems.value = checklistItems.value.filter((x) => x.id !== item.id)
}
</script>

<template>
  <div class="h-full flex flex-col gap-4 p-4 md:p-6 overflow-hidden bg-background text-foreground">
    <!-- Top Header: Board Switcher & Actions -->
    <div class="flex flex-wrap items-center justify-between gap-3 border-b border-border pb-3">
      <div class="flex items-center gap-2 overflow-x-auto pb-1 max-w-full">
        <button
          v-for="b in store.activeTableros"
          :key="b.id"
          class="px-3.5 py-1.5 rounded-md text-sm font-medium transition-colors whitespace-nowrap flex items-center gap-1.5"
          :class="
            store.currentTableroId === b.id
              ? 'bg-primary text-primary-foreground shadow-sm'
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
      <div class="relative flex-1 min-w-[200px]">
        <input
          v-model="searchText"
          type="text"
          :placeholder="t('Kanban.SearchCards')"
          class="w-full px-3 py-1.5 text-xs rounded-md bg-background border border-border text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-primary"
        />
      </div>

      <div class="flex items-center gap-1.5 text-xs text-muted-foreground">
        <span>{{ t('Kanban.Priority') }}:</span>
        <select
          v-model="selectedPrioridad"
          class="px-2.5 py-1 text-xs rounded-md bg-background border border-border text-foreground focus:outline-none"
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
      class="flex-1 flex gap-4 overflow-x-auto pb-4 items-start select-none"
    >
      <div
        v-for="col in store.detalle.columnas"
        :key="col.id"
        class="w-80 shrink-0 flex flex-col max-h-full rounded-xl bg-surface-card border border-border shadow-sm"
        @dragover.prevent
        @drop="onDrop($event, col.id)"
      >
        <!-- Column Header -->
        <div class="p-3 border-b border-border flex items-center justify-between gap-2">
          <div class="flex items-center gap-2 min-w-0">
            <span
              class="w-2.5 h-2.5 rounded-full shrink-0 bg-primary"
            />
            <h3 class="text-sm font-semibold text-foreground truncate">
              {{ col.nombre }}
            </h3>
            <span class="text-xs text-muted-foreground font-mono bg-muted px-1.5 py-0.5 rounded">
              {{ getTarjetasPorColumna(col.id).length }}
              <template v-if="col.limiteWip"> / {{ col.limiteWip }}</template>
            </span>
          </div>

          <div class="flex items-center gap-1">
            <button
              v-if="canCreate"
              class="p-1 rounded hover:bg-muted text-muted-foreground hover:text-foreground text-sm"
              :title="t('Kanban.NewCard')"
              @click="openCreateCard(col.id)"
            >
              +
            </button>
            <button
              v-if="canManage"
              class="p-1 rounded hover:bg-muted text-muted-foreground hover:text-foreground text-xs"
              :title="t('Kanban.EditColumn')"
              @click="openEditColumn(col)"
            >
              ✎
            </button>
            <button
              v-if="canManage && !store.currentTablero?.esPreset"
              class="p-1 rounded hover:bg-muted text-destructive text-xs"
              :title="t('Kanban.DeleteColumn')"
              @click="removeColumn(col)"
            >
              ✕
            </button>
          </div>
        </div>

        <!-- Cards Container -->
        <div class="flex-1 overflow-y-auto p-2.5 flex flex-col gap-2.5 min-h-[120px]">
          <div
            v-for="card in getTarjetasPorColumna(col.id)"
            :key="card.id"
            class="p-3 rounded-lg bg-surface-elevated border border-border shadow-sm hover:border-primary/40 transition-shadow cursor-grab active:cursor-grabbing flex flex-col gap-2"
            :draggable="canMove"
            @dragstart="onDragStart($event, card)"
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
                  class="hover:text-foreground flex items-center gap-1"
                  @click.stop="openChecklist(card)"
                >
                  ☑ {{ card.completadasChecklist }}/{{ card.totalChecklist }}
                </button>
              </div>

              <div class="flex items-center gap-1">
                <button
                  class="p-1 rounded hover:bg-muted text-muted-foreground hover:text-foreground"
                  :title="t('General.Edit')"
                  @click.stop="openEditCard(card)"
                >
                  ✎
                </button>
                <button
                  class="p-1 rounded hover:bg-muted text-destructive"
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
      class="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-sm p-4"
    >
      <div class="w-full max-w-md rounded-xl bg-surface-card border border-border p-5 shadow-lg flex flex-col gap-4">
        <div class="flex items-center justify-between border-b border-border pb-2">
          <h3 class="text-sm font-semibold text-foreground">
            {{ editingCard ? t('Kanban.EditCard') : t('Kanban.NewCard') }}
          </h3>
          <button class="text-muted-foreground hover:text-foreground" @click="showCardModal = false">✕</button>
        </div>

        <div class="flex flex-col gap-3 text-xs">
          <div>
            <label class="block font-medium text-foreground mb-1">{{ t('Kanban.Title') }} *</label>
            <input
              v-model="cardFormTitulo"
              type="text"
              class="w-full px-3 py-1.5 rounded-md bg-background border border-border text-foreground focus:outline-none focus:ring-1 focus:ring-primary"
            />
          </div>

          <div>
            <label class="block font-medium text-foreground mb-1">{{ t('Kanban.Description') }}</label>
            <textarea
              v-model="cardFormDescripcion"
              rows="3"
              class="w-full px-3 py-1.5 rounded-md bg-background border border-border text-foreground focus:outline-none focus:ring-1 focus:ring-primary"
            />
          </div>

          <div class="grid grid-cols-2 gap-2">
            <div>
              <label class="block font-medium text-foreground mb-1">{{ t('Kanban.Priority') }}</label>
              <select
                v-model="cardFormPrioridad"
                class="w-full px-2.5 py-1.5 rounded-md bg-background border border-border text-foreground focus:outline-none"
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
                class="w-full px-2.5 py-1.5 rounded-md bg-background border border-border text-foreground focus:outline-none"
              />
            </div>
          </div>
        </div>

        <div class="flex justify-end gap-2 border-t border-border pt-3">
          <button
            class="px-3 py-1.5 rounded-md text-xs font-medium border border-border hover:bg-muted text-muted-foreground"
            @click="showCardModal = false"
          >
            {{ t('General.Cancel') }}
          </button>
          <button
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
      class="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-sm p-4"
    >
      <div class="w-full max-w-sm rounded-xl bg-surface-card border border-border p-5 shadow-lg flex flex-col gap-4">
        <div class="flex items-center justify-between border-b border-border pb-2">
          <h3 class="text-sm font-semibold text-foreground">
            {{ editingColumn ? t('Kanban.EditColumn') : t('Kanban.NewColumn') }}
          </h3>
          <button class="text-muted-foreground hover:text-foreground" @click="showColumnModal = false">✕</button>
        </div>

        <div class="flex flex-col gap-3 text-xs">
          <div>
            <label class="block font-medium text-foreground mb-1">{{ t('Kanban.ColumnName') }} *</label>
            <input
              v-model="columnFormNombre"
              type="text"
              class="w-full px-3 py-1.5 rounded-md bg-background border border-border text-foreground focus:outline-none focus:ring-1 focus:ring-primary"
            />
          </div>

          <div class="grid grid-cols-2 gap-2">
            <div>
              <label class="block font-medium text-foreground mb-1">{{ t('Kanban.Color') }}</label>
              <input
                v-model="columnFormColor"
                type="color"
                class="w-full h-8 p-1 rounded-md bg-background border border-border cursor-pointer"
              />
            </div>
            <div>
              <label class="block font-medium text-foreground mb-1">{{ t('Kanban.WipLimit') }}</label>
              <input
                v-model.number="columnFormLimiteWip"
                type="number"
                min="1"
                class="w-full px-3 py-1.5 rounded-md bg-background border border-border text-foreground focus:outline-none"
              />
            </div>
          </div>
        </div>

        <div class="flex justify-end gap-2 border-t border-border pt-3">
          <button
            class="px-3 py-1.5 rounded-md text-xs font-medium border border-border hover:bg-muted text-muted-foreground"
            @click="showColumnModal = false"
          >
            {{ t('General.Cancel') }}
          </button>
          <button
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
      class="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-sm p-4"
    >
      <div class="w-full max-w-sm rounded-xl bg-surface-card border border-border p-5 shadow-lg flex flex-col gap-4">
        <div class="flex items-center justify-between border-b border-border pb-2">
          <h3 class="text-sm font-semibold text-foreground">{{ t('Kanban.NewBoard') }}</h3>
          <button class="text-muted-foreground hover:text-foreground" @click="showBoardModal = false">✕</button>
        </div>

        <div class="flex flex-col gap-3 text-xs">
          <div>
            <label class="block font-medium text-foreground mb-1">{{ t('Kanban.BoardName') }} *</label>
            <input
              v-model="boardFormNombre"
              type="text"
              class="w-full px-3 py-1.5 rounded-md bg-background border border-border text-foreground focus:outline-none focus:ring-1 focus:ring-primary"
            />
          </div>

          <div>
            <label class="block font-medium text-foreground mb-1">{{ t('Kanban.Description') }}</label>
            <textarea
              v-model="boardFormDescripcion"
              rows="2"
              class="w-full px-3 py-1.5 rounded-md bg-background border border-border text-foreground focus:outline-none focus:ring-1 focus:ring-primary"
            />
          </div>
        </div>

        <div class="flex justify-end gap-2 border-t border-border pt-3">
          <button
            class="px-3 py-1.5 rounded-md text-xs font-medium border border-border hover:bg-muted text-muted-foreground"
            @click="showBoardModal = false"
          >
            {{ t('General.Cancel') }}
          </button>
          <button
            class="px-4 py-1.5 rounded-md text-xs font-medium bg-primary text-primary-foreground hover:bg-primary/90"
            @click="saveBoard"
          >
            {{ t('General.Save') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Modal: Checklist Items -->
    <div
      v-if="showChecklistModal"
      class="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-sm p-4"
    >
      <div class="w-full max-w-md rounded-xl bg-surface-card border border-border p-5 shadow-lg flex flex-col gap-4">
        <div class="flex items-center justify-between border-b border-border pb-2">
          <h3 class="text-sm font-semibold text-foreground">
            {{ t('Kanban.Checklist') }} - {{ checklistCard?.titulo }}
          </h3>
          <button class="text-muted-foreground hover:text-foreground" @click="showChecklistModal = false">✕</button>
        </div>

        <!-- Checklist List -->
        <div class="flex flex-col gap-2 max-h-60 overflow-y-auto">
          <div
            v-for="item in checklistItems"
            :key="item.id"
            class="flex items-center justify-between gap-2 p-2 rounded-md bg-muted/40 text-xs"
          >
            <label class="flex items-center gap-2 cursor-pointer select-none">
              <input
                type="checkbox"
                :checked="item.completada"
                class="rounded border-border text-primary focus:ring-0"
                @change="toggleChecklist(item)"
              />
              <span :class="item.completada ? 'line-through text-muted-foreground' : 'text-foreground'">
                {{ item.titulo }}
              </span>
            </label>
            <button class="text-destructive hover:opacity-80 text-xs px-1" @click="removeChecklist(item)">✕</button>
          </div>

          <div v-if="checklistItems.length === 0" class="text-xs text-muted-foreground text-center py-4">
            {{ t('Kanban.EmptyChecklist') }}
          </div>
        </div>

        <!-- Add Item -->
        <div class="flex gap-2">
          <input
            v-model="newChecklistTitle"
            type="text"
            :placeholder="t('Kanban.NewChecklistItem')"
            class="flex-1 px-3 py-1.5 text-xs rounded-md bg-background border border-border text-foreground focus:outline-none focus:ring-1 focus:ring-primary"
            @keyup.enter="addChecklist"
          />
          <button
            class="px-3 py-1.5 rounded-md text-xs font-medium bg-primary text-primary-foreground hover:bg-primary/90"
            @click="addChecklist"
          >
            {{ t('Kanban.Add') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
