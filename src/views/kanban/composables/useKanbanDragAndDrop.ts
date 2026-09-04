import { ref, type ComputedRef } from 'vue'
import type {
  useKanbanStore,
  KanbanColumnaDto,
  KanbanTarjetaDto,
  Uuid,
} from '@/stores/useKanbanStore'

export type DragType = 'card' | 'column' | null

export function useKanbanDragAndDrop(options: {
  store: ReturnType<typeof useKanbanStore>
  canMove: ComputedRef<boolean>
  canManage: ComputedRef<boolean>
  sortedColumnas: ComputedRef<KanbanColumnaDto[]>
  getTarjetasPorColumna: (columnaId: Uuid) => KanbanTarjetaDto[]
  onCardClickToEdit: (card: KanbanTarjetaDto) => void
}) {
  const { store, canMove, canManage, sortedColumnas, getTarjetasPorColumna, onCardClickToEdit } = options

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
      onCardClickToEdit(activeCard)
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

  return {
    dragType,
    draggingCard,
    draggingColumn,
    dragPosition,
    dragHoverColumnaId,
    dragHoverCardId,
    onCardPointerDown,
    onColumnPointerDown,
    moverColumna,
  }
}
