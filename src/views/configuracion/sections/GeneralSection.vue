<script setup lang="ts">
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useToast } from 'primevue/usetoast'

import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import { useSistemaStore, type Cambios } from '@/stores/useSistemaStore'
import { useUiStore } from '@/stores/useUiStore'
import { useKanbanStore } from '@/stores/useKanbanStore'
import { isSupportedLocale, setLocale } from '@/i18n'

interface GeneralDraft {
  theme: 'light' | 'dark' | 'system'
  language: string
  formatoFecha: string
  simboloMoneda: string
  separadorMiles: string
  separadorDecimal: string
}

const sistema = useSistemaStore()
const ui = useUiStore()
const kanbanStore = useKanbanStore()
const toast = useToast()
const { t } = useI18n()

const draft = ref<GeneralDraft | null>(null)
const saving = ref(false)

const themeOptions = computed(() => [
  { label: t('Configuracion.TemaClaro'), value: 'light' },
  { label: t('Configuracion.TemaOscuro'), value: 'dark' },
  { label: t('Configuracion.TemaSistema'), value: 'system' },
])

const languageOptions = computed(() => [
  { label: 'Español', value: 'es' },
  { label: 'English', value: 'en' },
])

function load(): void {
  if (sistema.config) {
    draft.value = {
      theme: sistema.config.application.theme,
      language: sistema.config.locale.language,
      formatoFecha: sistema.config.locale.formatoFecha,
      simboloMoneda: sistema.config.locale.simboloMoneda,
      separadorMiles: sistema.config.locale.separadorMiles,
      separadorDecimal: sistema.config.locale.separadorDecimal,
    }
  }
}

const cambios = computed<Cambios>(() => {
  if (!draft.value || !sistema.config) return {}
  const diff: Cambios = {}
  if (draft.value.theme !== sistema.config.application.theme) {
    diff['application.theme'] = draft.value.theme
  }
  if (draft.value.language !== sistema.config.locale.language) {
    diff['locale.language'] = draft.value.language
  }
  if (draft.value.formatoFecha !== sistema.config.locale.formatoFecha) {
    diff['locale.formatoFecha'] = draft.value.formatoFecha
  }
  if (draft.value.simboloMoneda !== sistema.config.locale.simboloMoneda) {
    diff['locale.simboloMoneda'] = draft.value.simboloMoneda
  }
  if (draft.value.separadorMiles !== sistema.config.locale.separadorMiles) {
    diff['locale.separadorMiles'] = draft.value.separadorMiles
  }
  if (draft.value.separadorDecimal !== sistema.config.locale.separadorDecimal) {
    diff['locale.separadorDecimal'] = draft.value.separadorDecimal
  }
  return diff
})

const isDirty = computed(() => Object.keys(cambios.value).length > 0)

async function apply(): Promise<void> {
  if (!isDirty.value) return
  saving.value = true
  try {
    const updated = await sistema.applyConfig(cambios.value)
    ui.setTheme(updated.application.theme)
    if (isSupportedLocale(updated.locale.language)) {
      setLocale(updated.locale.language)
    }
    toast.add({
      severity: 'success',
      summary: t('Configuracion.Applied'),
      life: 3000,
    })
    load()
  } finally {
    saving.value = false
  }
}

onMounted(() => load())
watch(
  () => sistema.config,
  () => load(),
)
</script>

<template>
  <form v-if="draft" class="flex max-w-4xl flex-col gap-6" @submit.prevent="apply">
    <div class="rounded-lg border border-border bg-surface-card p-6 shadow-sm">
      <div class="mb-4 flex items-center gap-2 border-b border-border pb-3">
        <AppIcon name="palette" :size="18" class="text-primary" />
        <h3 class="text-sm font-semibold text-foreground">{{ $t('Configuracion.General') }}</h3>
      </div>

      <div class="grid gap-4 sm:grid-cols-2">
        <label class="flex flex-col gap-1.5">
          <span class="text-xs font-medium text-foreground">{{ $t('Configuracion.Idioma') }}</span>
          <Select v-model="draft.language" :options="languageOptions" option-label="label" option-value="value" fluid />
        </label>
        <label class="flex flex-col gap-1.5">
          <span class="text-xs font-medium text-foreground">{{ $t('Configuracion.Tema') }}</span>
          <Select v-model="draft.theme" :options="themeOptions" option-label="label" option-value="value" fluid />
        </label>
      </div>
    </div>

    <div class="rounded-lg border border-border bg-surface-card p-6 shadow-sm">
      <div class="mb-4 flex items-center gap-2 border-b border-border pb-3">
        <AppIcon name="globe" :size="18" class="text-primary" />
        <h3 class="text-sm font-semibold text-foreground">{{ $t('Configuracion.FormatoFecha') }} / {{ $t('Configuracion.SimboloMoneda') }}</h3>
      </div>

      <div class="grid gap-4 sm:grid-cols-2">
        <label class="flex flex-col gap-1.5">
          <span class="text-xs font-medium text-foreground">{{ $t('Configuracion.FormatoFecha') }}</span>
          <InputText v-model="draft.formatoFecha" class="w-full" />
        </label>
        <label class="flex flex-col gap-1.5">
          <span class="text-xs font-medium text-foreground">{{ $t('Configuracion.SimboloMoneda') }}</span>
          <InputText v-model="draft.simboloMoneda" class="w-full" />
        </label>
        <label class="flex flex-col gap-1.5">
          <span class="text-xs font-medium text-foreground">{{ $t('Configuracion.SeparadorMiles') }}</span>
          <InputText v-model="draft.separadorMiles" class="w-full" />
        </label>
        <label class="flex flex-col gap-1.5">
          <span class="text-xs font-medium text-foreground">{{ $t('Configuracion.SeparadorDecimal') }}</span>
          <InputText v-model="draft.separadorDecimal" class="w-full" />
        </label>
      </div>
    </div>

    <div class="rounded-lg border border-border bg-surface-card p-6 shadow-sm">
      <div class="mb-4 flex items-center gap-2 border-b border-border pb-3">
        <AppIcon name="columns" :size="18" class="text-primary" />
        <h3 class="text-sm font-semibold text-foreground">Tablero Kanban</h3>
      </div>

      <div class="flex items-center justify-between">
        <div class="flex flex-col gap-0.5">
          <span class="text-xs font-medium text-foreground">Botones de movimiento en columnas (◀ / ▶)</span>
          <span class="text-xs text-muted-foreground">Muestra botones en la cabecera de las columnas para reordenar sin necesidad de arrastrar</span>
        </div>
        <label class="relative inline-flex items-center cursor-pointer">
          <input
            v-model="kanbanStore.showColumnMoveButtons"
            type="checkbox"
            class="rounded border-border text-primary"
          />
        </label>
      </div>
    </div>

    <div class="flex justify-end gap-3">
      <Button :disabled="!isDirty || saving" class="flex items-center gap-2" @click="apply">
        <AppIcon name="save" :size="16" />
        {{ $t('Configuracion.Apply') }}
      </Button>
    </div>
  </form>
</template>
