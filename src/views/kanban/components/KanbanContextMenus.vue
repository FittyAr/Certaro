<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import ContextMenu from 'primevue/contextmenu'
import type {
  useKanbanStore,
  KanbanColumnaDto,
  KanbanTableroDto,
  KanbanTarjetaDto,
  PrioridadTarjeta,
  Uuid,
} from '@/stores/useKanbanStore'

const props = defineProps<{
  store: ReturnType<typeof useKanbanStore>
  sortedColumnas: KanbanColumnaDto[]
  canManage: boolean
  canCreate: boolean
  canMove: boolean
  getTarjetasPorColumna: (columnaId: Uuid) => any[]
  openEditCard: (card: KanbanTarjetaDto) => void
  openChecklist: (card: KanbanTarjetaDto) => void
  removeCard: (card: KanbanTarjetaDto) => void
  openCreateCard: (columnaId: Uuid) => void
  openEditColumn: (col: KanbanColumnaDto) => void
  moverColumna: (col: KanbanColumnaDto, dir: 'izq' | 'der') => void
  confirmDeleteColumna: (col: KanbanColumnaDto) => void
  openEditBoard: (b: KanbanTableroDto) => void
  handleDeleteBoardPrompt: (b: KanbanTableroDto) => void
}>()

const { t } = useI18n()

const cardMenuRef = ref<InstanceType<typeof ContextMenu> | null>(null)
const columnMenuRef = ref<InstanceType<typeof ContextMenu> | null>(null)
const boardMenuRef = ref<InstanceType<typeof ContextMenu> | null>(null)

const contextCard = ref<KanbanTarjetaDto | null>(null)
const contextColumn = ref<KanbanColumnaDto | null>(null)
const contextBoard = ref<KanbanTableroDto | null>(null)

function showCardMenu(event: MouseEvent, card: KanbanTarjetaDto) {
  contextCard.value = card
  cardMenuRef.value?.show(event)
}

function showColumnMenu(event: MouseEvent, col: KanbanColumnaDto) {
  contextColumn.value = col
  columnMenuRef.value?.show(event)
}

function showBoardMenu(event: MouseEvent, board: KanbanTableroDto) {
  contextBoard.value = board
  boardMenuRef.value?.show(event)
}

defineExpose({
  showCardMenu,
  showColumnMenu,
  showBoardMenu,
})

const cardMenuItems = computed(() => {
  const card = contextCard.value
  if (!card) return []

  const otherCols = props.sortedColumnas
    .filter((c) => c.id !== card.columnaId)
    .map((c) => ({
      label: c.nombre,
      command: async () => {
        const destCards = props.getTarjetasPorColumna(c.id)
        await props.store.reordenarTarjetas({
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
      await props.store.updateTarjeta(card.id, {
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
      command: () => props.openEditCard(card),
    },
    {
      label: `${t('Kanban.Checklist')} (${card.completadasChecklist}/${card.totalChecklist})`,
      icon: 'pi pi-check-square',
      command: () => props.openChecklist(card),
    },
    { separator: true },
    {
      label: 'Mover a columna',
      icon: 'pi pi-arrow-right',
      disabled: !props.canMove || otherCols.length === 0,
      items: otherCols,
    },
    {
      label: t('Kanban.Priority'),
      icon: 'pi pi-flag',
      disabled: !props.canMove,
      items: priorityItems,
    },
    { separator: true },
    {
      label: t('General.Delete'),
      icon: 'pi pi-trash',
      command: () => props.removeCard(card),
    },
  ]
})

const columnMenuItems = computed(() => {
  const col = contextColumn.value
  if (!col) return []
  const cols = props.sortedColumnas
  const idx = cols.findIndex((c) => c.id === col.id)
  const isFirst = idx <= 0
  const isLast = idx === -1 || idx >= cols.length - 1

  return [
    {
      label: t('Kanban.NewCard'),
      icon: 'pi pi-plus',
      disabled: !props.canCreate,
      command: () => props.openCreateCard(col.id),
    },
    {
      label: t('Kanban.EditColumn'),
      icon: 'pi pi-pencil',
      disabled: !props.canManage,
      command: () => props.openEditColumn(col),
    },
    { separator: true },
    {
      label: 'Mover a la izquierda',
      icon: 'pi pi-arrow-left',
      disabled: !props.canManage || isFirst,
      command: () => props.moverColumna(col, 'izq'),
    },
    {
      label: 'Mover a la derecha',
      icon: 'pi pi-arrow-right',
      disabled: !props.canManage || isLast,
      command: () => props.moverColumna(col, 'der'),
    },
    { separator: true },
    {
      label: t('Kanban.DeleteColumn'),
      icon: 'pi pi-trash',
      disabled: !props.canManage || Boolean(props.store.currentTablero?.esPreset),
      command: () => props.confirmDeleteColumna(col),
    },
  ]
})

const boardMenuItems = computed(() => {
  const b = contextBoard.value
  if (!b) return []

  return [
    {
      label: props.store.isTableroPinned(b.id) ? 'Desfijar del inicio' : 'Fijar tablero al inicio',
      icon: 'pi pi-thumbtack',
      command: () => props.store.togglePinTablero(b.id),
    },
    {
      label: 'Editar tablero',
      icon: 'pi pi-pencil',
      disabled: !props.canManage,
      command: () => props.openEditBoard(b),
    },
    {
      label: b.activo ? 'Ocultar tablero' : 'Mostrar tablero',
      icon: b.activo ? 'pi pi-eye-slash' : 'pi pi-eye',
      disabled: !props.canManage,
      command: () => props.store.toggleTableroActivo(b),
    },
    {
      label: t('Kanban.Sync'),
      icon: 'pi pi-sync',
      visible: Boolean(b.esPreset),
      command: () => props.store.syncPreset(b.id),
    },
    { separator: true },
    {
      label: 'Eliminar tablero',
      icon: 'pi pi-trash',
      disabled: !props.canManage || Boolean(b.esPreset),
      command: () => props.handleDeleteBoardPrompt(b),
    },
  ]
})
</script>

<template>
  <div>
    <ContextMenu ref="cardMenuRef" :model="cardMenuItems" />
    <ContextMenu ref="columnMenuRef" :model="columnMenuItems" />
    <ContextMenu ref="boardMenuRef" :model="boardMenuItems" />
  </div>
</template>
