<script setup lang="ts">
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import { onMounted, watch } from 'vue'

import { Button } from '@/components/ui/button'
import { useConfigForm } from '@/composables/useConfigForm'
import { useSistemaStore } from '@/stores/useSistemaStore'

const sistema = useSistemaStore()
const { draft, saving, isDirty, apply, load } = useConfigForm(() => sistema.config?.locale ?? null)

const themeOptions = [
  { label: 'Claro', value: 'light' },
  { label: 'Oscuro', value: 'dark' },
  { label: 'Sistema', value: 'system' },
]

const languageOptions = [
  { label: 'Español', value: 'es' },
  { label: 'English', value: 'en' },
]

onMounted(() => load())
watch(() => sistema.config, () => load())
</script>

<template>
  <form v-if="draft" class="flex max-w-xl flex-col gap-4 p-4" @submit.prevent="apply">
    <div class="grid grid-cols-2 gap-4">
      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Configuracion.General') }} — Idioma</span>
        <Select v-model="draft.language" :options="languageOptions" option-label="label" option-value="value" />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-sm">Tema</span>
        <Select v-model="sistema.config!.application.theme" :options="themeOptions" option-label="label" option-value="value" />
      </label>
    </div>

    <div class="grid grid-cols-2 gap-4">
      <label class="flex flex-col gap-1">
        <span class="text-sm">Formato de fecha</span>
        <InputText v-model="draft.formatoFecha" />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-sm">Símbolo de moneda</span>
        <InputText v-model="draft.simboloMoneda" />
      </label>
    </div>

    <div class="grid grid-cols-2 gap-4">
      <label class="flex flex-col gap-1">
        <span class="text-sm">Separador de miles</span>
        <InputText v-model="draft.separadorMiles" />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-sm">Separador decimal</span>
        <InputText v-model="draft.separadorDecimal" />
      </label>
    </div>

    <div class="flex justify-end">
      <Button :disabled="!isDirty || saving" @click="apply">
        {{ $t('Configuracion.Apply') }}
      </Button>
    </div>
  </form>
</template>
