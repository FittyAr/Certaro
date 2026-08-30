<script setup lang="ts">
import InputText from 'primevue/inputtext'
import InputNumber from 'primevue/inputnumber'
import ToggleSwitch from 'primevue/toggleswitch'
import { onMounted, watch } from 'vue'

import { Button } from '@/components/ui/button'
import { useConfigForm } from '@/composables/useConfigForm'
import { useSistemaStore } from '@/stores/useSistemaStore'

const sistema = useSistemaStore()
const { draft, saving, isDirty, apply, load } = useConfigForm(
  () => sistema.config?.externalApis ?? null,
)

onMounted(() => load())
watch(() => sistema.config, () => load())
</script>

<template>
  <form v-if="draft" class="flex max-w-xl flex-col gap-4 p-4" @submit.prevent="apply">
    <label class="flex flex-col gap-1">
      <span class="text-sm">URL del dólar</span>
      <InputText v-model="draft.dollarUrl" />
    </label>

    <label class="flex flex-col gap-1">
      <span class="text-sm">URL de feriados (base)</span>
      <InputText v-model="draft.holidayUrl" />
    </label>

    <div class="grid grid-cols-2 gap-4">
      <label class="flex flex-col gap-1">
        <span class="text-sm">Timeout (segundos)</span>
        <InputNumber v-model="draft.timeoutSeconds" :min="1" :max="120" fluid />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-sm">Reintentos</span>
        <InputNumber v-model="draft.reintentos" :min="0" :max="5" fluid />
      </label>
    </div>

    <div class="grid grid-cols-2 gap-4">
      <label class="flex flex-col gap-1">
        <span class="text-sm">Caché dólar (minutos)</span>
        <InputNumber v-model="draft.dollarCacheMinutes" :min="1" :max="1440" fluid />
      </label>
      <label class="flex items-center gap-2 pt-6">
        <ToggleSwitch v-model="draft.dollarAutoUpdate" />
        <span class="text-sm">Actualizar dólar automáticamente</span>
      </label>
    </div>

    <div class="flex justify-end">
      <Button :disabled="!isDirty || saving" @click="apply">
        {{ $t('Configuracion.Apply') }}
      </Button>
    </div>
  </form>
</template>
