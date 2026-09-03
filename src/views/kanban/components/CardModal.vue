<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import type { KanbanTarjetaDto, PrioridadTarjeta, Uuid } from '@/stores/useKanbanStore'

const props = defineProps<{
  show: boolean
  editingCard: KanbanTarjetaDto | null
  columnaId: Uuid
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (
    e: 'save',
    data: {
      titulo: string
      descripcion: string | null
      prioridad: PrioridadTarjeta
      fechaVencimiento: string | null
      etiquetaIds: Uuid[]
    },
  ): void
}>()

const { t } = useI18n()

const formTitulo = ref('')
const formDescripcion = ref('')
const formPrioridad = ref<PrioridadTarjeta>('Normal')
const formFechaVencimiento = ref('')
const formEtiquetas = ref<Uuid[]>([])

watch(
  () => props.show,
  (open) => {
    if (open) {
      if (props.editingCard) {
        formTitulo.value = props.editingCard.titulo
        formDescripcion.value = props.editingCard.descripcion ?? ''
        formPrioridad.value = props.editingCard.prioridad
        formFechaVencimiento.value = props.editingCard.fechaVencimiento ?? ''
        formEtiquetas.value = props.editingCard.etiquetas.map((e) => e.id)
      } else {
        formTitulo.value = ''
        formDescripcion.value = ''
        formPrioridad.value = 'Normal'
        formFechaVencimiento.value = ''
        formEtiquetas.value = []
      }
    }
  },
  { immediate: true },
)

function submit() {
  if (!formTitulo.value.trim()) return
  emit('save', {
    titulo: formTitulo.value.trim(),
    descripcion: formDescripcion.value.trim() || null,
    prioridad: formPrioridad.value,
    fechaVencimiento: formFechaVencimiento.value || null,
    etiquetaIds: formEtiquetas.value,
  })
}
</script>

<template>
  <div
    v-if="props.show"
    class="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-xs p-4 select-none"
  >
    <div class="w-full max-w-md rounded-xl bg-surface-card border border-border p-5 shadow-lg flex flex-col gap-4">
      <div class="flex items-center justify-between border-b border-border pb-2">
        <h3 class="text-sm font-semibold text-foreground">
          {{ props.editingCard ? t('Kanban.EditCard') : t('Kanban.NewCard') }}
        </h3>
        <button type="button" class="text-muted-foreground hover:text-foreground" @click="emit('close')">✕</button>
      </div>

      <div class="flex flex-col gap-3 text-xs">
        <div>
          <label class="block font-medium text-foreground mb-1">{{ t('Kanban.Title') }} *</label>
          <input
            v-model="formTitulo"
            type="text"
            class="w-full px-3 py-1.5 rounded-md bg-background border border-border text-foreground focus:outline-hidden focus:ring-1 focus:ring-primary"
          />
        </div>

        <div>
          <label class="block font-medium text-foreground mb-1">{{ t('Kanban.Description') }}</label>
          <textarea
            v-model="formDescripcion"
            rows="3"
            class="w-full px-3 py-1.5 rounded-md bg-background border border-border text-foreground focus:outline-hidden focus:ring-1 focus:ring-primary"
          />
        </div>

        <div class="grid grid-cols-2 gap-2">
          <div>
            <label class="block font-medium text-foreground mb-1">{{ t('Kanban.Priority') }}</label>
            <select
              v-model="formPrioridad"
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
              v-model="formFechaVencimiento"
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
          @click="emit('close')"
        >
          {{ t('General.Cancel') }}
        </button>
        <button
          type="button"
          class="px-4 py-1.5 rounded-md text-xs font-medium bg-primary text-primary-foreground hover:bg-primary/90"
          @click="submit"
        >
          {{ t('General.Save') }}
        </button>
      </div>
    </div>
  </div>
</template>
