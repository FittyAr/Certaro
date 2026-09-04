import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import type {
  useKanbanStore,
  KanbanColumnaDto,
  KanbanTableroDto,
  KanbanTarjetaDto,
  PrioridadTarjeta,
  Uuid,
} from '@/stores/useKanbanStore'

export function useKanbanModals(store: ReturnType<typeof useKanbanStore>) {
  const { t } = useI18n()

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

  // Card CRUD Handlers
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

  // Column CRUD Handlers
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

  // Board CRUD Handlers
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

  // Checklist Handlers
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

  return {
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
  }
}
