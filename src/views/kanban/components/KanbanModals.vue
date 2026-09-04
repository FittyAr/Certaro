<script setup lang="ts">
import CardModal from './CardModal.vue'
import ColumnModal from './ColumnModal.vue'
import BoardModal from './BoardModal.vue'
import ManageBoardsModal from './ManageBoardsModal.vue'
import DeleteColumnModal from './DeleteColumnModal.vue'
import StrictDeleteBoardModal from './StrictDeleteBoardModal.vue'
import ChecklistModal from './ChecklistModal.vue'
import type { useKanbanModals } from '../composables/useKanbanModals'
import type { useKanbanStore, Uuid } from '@/stores/useKanbanStore'

defineProps<{
  modals: ReturnType<typeof useKanbanModals>
  store: ReturnType<typeof useKanbanStore>
  getTarjetasPorColumna: (columnaId: Uuid) => any[]
}>()
</script>

<template>
  <div>
    <CardModal
      :show="modals.showCardModal.value"
      :editing-card="modals.editingCard.value"
      :columna-id="modals.cardFormColumnaId.value"
      @close="modals.showCardModal.value = false"
      @save="modals.handleSaveCard"
    />

    <ColumnModal
      :show="modals.showColumnModal.value"
      :editing-column="modals.editingColumn.value"
      :default-orden="store.detalle?.columnas.length ?? 0"
      @close="modals.showColumnModal.value = false"
      @save="modals.handleSaveColumn"
    />

    <BoardModal
      :show="modals.showBoardModal.value"
      :editing-board="modals.editingBoard.value"
      @close="modals.showBoardModal.value = false"
      @save="modals.handleSaveBoard"
    />

    <ManageBoardsModal
      :show="modals.showManageBoardsModal.value"
      :tableros="store.tableros"
      @close="modals.showManageBoardsModal.value = false"
      @create-board="modals.openCreateBoard"
      @edit-board="modals.openEditBoard"
      @delete-board="modals.handleDeleteBoardPrompt"
    />

    <DeleteColumnModal
      :show="modals.showDeleteColModal.value"
      :column="modals.colToDelete.value"
      :card-count="modals.colToDelete.value ? getTarjetasPorColumna(modals.colToDelete.value.id).length : 0"
      @close="modals.showDeleteColModal.value = false"
      @confirm="modals.executeDeleteColumn"
    />

    <StrictDeleteBoardModal
      :show="modals.showStrictDeleteBoardModal.value"
      :board="modals.boardToDelete.value"
      @close="modals.showStrictDeleteBoardModal.value = false"
      @confirm="modals.executeStrictDeleteBoard"
    />

    <ChecklistModal
      :show="modals.showChecklistModal.value"
      :card="modals.checklistCard.value"
      :items="modals.checklistItems.value"
      @close="modals.showChecklistModal.value = false"
      @add-item="modals.handleAddChecklist"
      @toggle-item="modals.handleToggleChecklist"
      @remove-item="modals.handleRemoveChecklist"
    />
  </div>
</template>
