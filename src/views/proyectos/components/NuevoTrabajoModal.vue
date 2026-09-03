<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import Select from 'primevue/select'
import InputText from 'primevue/inputtext'
import MoneyInput from '@/components/domain/MoneyInput.vue'
import DateInput from '@/components/domain/DateInput.vue'
import FieldError from '@/components/domain/FieldError.vue'
import type { LookupItem } from '@/stores/useCatalogStore'
import { useTrabajosStore, type TrabajoInput } from '@/stores/useTrabajosStore'
import { useApiError } from '@/composables/useApiError'

const props = defineProps<{
  show: boolean
  proyectoId: string | null
  proyectos: LookupItem[]
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'saved'): void
}>()

const { t } = useI18n()
const { notify } = useApiError()
const trabajosStore = useTrabajosStore()

const model = ref<TrabajoInput>({
  descripcion: '',
  proyectoId: '',
  presupuesto: '0.0000',
  fechaInicio: new Date().toISOString().slice(0, 10),
  fechaFin: null,
})
const fieldErrors = ref<Record<string, string>>({})
const saving = ref(false)

watch(
  () => props.show,
  (open) => {
    if (open) {
      fieldErrors.value = {}
      model.value = {
        descripcion: '',
        proyectoId: props.proyectoId ?? (props.proyectos[0]?.id ?? ''),
        presupuesto: '0.0000',
        fechaInicio: new Date().toISOString().slice(0, 10),
        fechaFin: null,
      }
    }
  },
  { immediate: true },
)

async function submit() {
  if (!model.value.descripcion.trim()) {
    fieldErrors.value.descripcion = t('Validation.Trabajo.DescripcionRequired')
    return
  }
  if (!model.value.proyectoId) {
    fieldErrors.value.proyectoId = t('Validation.Trabajo.ProyectoRequired')
    return
  }

  saving.value = true
  fieldErrors.value = {}
  try {
    await trabajosStore.create({
      descripcion: model.value.descripcion.trim(),
      proyectoId: model.value.proyectoId,
      presupuesto: model.value.presupuesto,
      fechaInicio: model.value.fechaInicio,
      fechaFin: model.value.fechaFin || null,
    })
    emit('saved')
    emit('close')
  } catch (err: unknown) {
    notify(err)
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <div
    v-if="props.show"
    class="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-xs p-4 select-none"
  >
    <div class="w-full max-w-md rounded-xl bg-surface-card border border-border p-5 shadow-2xl flex flex-col gap-4 animate-in fade-in zoom-in-95 duration-150">
      <div class="flex items-center justify-between border-b border-border pb-2">
        <h3 class="text-sm font-semibold text-foreground flex items-center gap-2">
          <span>💼</span>
          <span>{{ t('Trabajos.New') || 'Nuevo Trabajo' }}</span>
        </h3>
        <button type="button" class="text-muted-foreground hover:text-foreground" @click="emit('close')">✕</button>
      </div>

      <div class="flex flex-col gap-3 text-xs">
        <label class="flex flex-col gap-1">
          <span class="font-medium text-foreground">{{ t('Trabajos.Proyecto') }} *</span>
          <Select
            v-model="model.proyectoId"
            :options="props.proyectos"
            option-label="label"
            option-value="id"
            filter
            :placeholder="$t('General.Select')"
            class="w-full"
          />
          <FieldError id="trabajo-proyecto-error" :message="fieldErrors.proyectoId" />
        </label>

        <label class="flex flex-col gap-1">
          <span class="font-medium text-foreground">{{ t('Trabajos.Descripcion') }} *</span>
          <InputText
            v-model="model.descripcion"
            :placeholder="t('Trabajos.DescripcionHint') || 'Descripción del trabajo o tarea a realizar'"
            class="w-full"
            @keyup.enter="submit"
          />
          <FieldError id="trabajo-descripcion-error" :message="fieldErrors.descripcion" />
        </label>

        <label class="flex flex-col gap-1">
          <span class="font-medium text-foreground">{{ t('Trabajos.Presupuesto') }}</span>
          <MoneyInput v-model="model.presupuesto" />
        </label>

        <div class="grid grid-cols-2 gap-2">
          <label class="flex flex-col gap-1">
            <span class="font-medium text-foreground">{{ t('Trabajos.FechaInicio') }} *</span>
            <DateInput v-model="model.fechaInicio" />
          </label>
          <label class="flex flex-col gap-1">
            <span class="font-medium text-foreground">{{ t('Trabajos.FechaFin') }}</span>
            <DateInput v-model="model.fechaFin" />
          </label>
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
          class="px-4 py-1.5 rounded-md text-xs font-semibold bg-primary text-primary-foreground hover:bg-primary/90 transition-opacity disabled:opacity-50"
          :disabled="saving"
          @click="submit"
        >
          {{ t('General.Save') }}
        </button>
      </div>
    </div>
  </div>
</template>
