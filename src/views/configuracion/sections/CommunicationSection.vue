<script setup lang="ts">
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import Textarea from 'primevue/textarea'
import { onMounted, watch } from 'vue'

import { Button } from '@/components/ui/button'
import { useConfigForm } from '@/composables/useConfigForm'
import { useSistemaStore } from '@/stores/useSistemaStore'

const sistema = useSistemaStore()
const { draft, saving, isDirty, apply, load } = useConfigForm(
  () => sistema.config?.communication ?? null,
)

const emailOptions = [
  { label: 'Sistema', value: 'systemDefault' },
  { label: 'Gmail', value: 'gmail' },
  { label: 'Outlook', value: 'outlook' },
  { label: 'Yahoo', value: 'yahoo' },
]

onMounted(() => load())
watch(() => sistema.config, () => load())
</script>

<template>
  <form v-if="draft" class="flex max-w-xl flex-col gap-4 p-4" @submit.prevent="apply">
    <div class="grid grid-cols-2 gap-4">
      <label class="flex flex-col gap-1">
        <span class="text-sm">Cliente de email</span>
        <Select v-model="draft.emailCliente" :options="emailOptions" option-label="label" option-value="value" />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-sm">Código de país (WhatsApp)</span>
        <InputText v-model="draft.codigoPais" />
      </label>
    </div>

    <label class="flex flex-col gap-1">
      <span class="text-sm">Asunto liquidación por email</span>
      <InputText v-model="draft.emailLiquidacionAsunto" />
    </label>

    <label class="flex flex-col gap-1">
      <span class="text-sm">Plantilla WhatsApp</span>
      <Textarea v-model="draft.whatsAppTemplate" rows="2" />
    </label>

    <label class="flex flex-col gap-1">
      <span class="text-sm">Plantilla WhatsApp liquidación</span>
      <Textarea v-model="draft.whatsAppLiquidacionTemplate" rows="2" />
    </label>

    <div class="flex justify-end">
      <Button :disabled="!isDirty || saving" @click="apply">
        {{ $t('Configuracion.Apply') }}
      </Button>
    </div>
  </form>
</template>
