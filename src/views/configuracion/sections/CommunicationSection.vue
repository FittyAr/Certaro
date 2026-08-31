<script setup lang="ts">
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import Textarea from 'primevue/textarea'
import { computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import { useConfigForm } from '@/composables/useConfigForm'
import { useSistemaStore } from '@/stores/useSistemaStore'

const { t } = useI18n()
const sistema = useSistemaStore()
const { draft, saving, isDirty, apply, load } = useConfigForm(
  'communication',
  () => sistema.config?.communication ?? null,
)

const emailOptions = computed(() => [
  { label: t('Configuracion.TemaSistema'), value: 'systemDefault' },
  { label: 'Gmail', value: 'gmail' },
  { label: 'Outlook', value: 'outlook' },
  { label: 'Yahoo', value: 'yahoo' },
])

onMounted(() => load())
watch(
  () => sistema.config,
  () => load(),
)
</script>

<template>
  <form v-if="draft" class="flex max-w-4xl flex-col gap-6" @submit.prevent="apply">
    <!-- Canales de comunicación -->
    <div class="rounded-lg border border-border bg-surface-card p-6 shadow-sm">
      <div class="mb-4 flex items-center gap-2 border-b border-border pb-3">
        <AppIcon name="mail" :size="18" class="text-primary" />
        <h3 class="text-sm font-semibold text-foreground">{{ $t('Configuracion.EmailCliente') }} / {{ $t('Configuracion.CodigoPais') }}</h3>
      </div>

      <div class="grid gap-4 sm:grid-cols-2">
        <label class="flex flex-col gap-1.5">
          <span class="text-xs font-medium text-foreground">{{ $t('Configuracion.EmailCliente') }}</span>
          <Select v-model="draft.emailCliente" :options="emailOptions" option-label="label" option-value="value" fluid />
        </label>

        <label class="flex flex-col gap-1.5">
          <span class="text-xs font-medium text-foreground">{{ $t('Configuracion.CodigoPais') }}</span>
          <InputText v-model="draft.codigoPais" class="w-full" />
        </label>

        <label class="flex flex-col gap-1.5 sm:col-span-2">
          <span class="text-xs font-medium text-foreground">{{ $t('Configuracion.EmailLiquidacionAsunto') }}</span>
          <InputText v-model="draft.emailLiquidacionAsunto" class="w-full" />
        </label>
      </div>
    </div>

    <!-- Plantillas de WhatsApp -->
    <div class="rounded-lg border border-border bg-surface-card p-6 shadow-sm">
      <div class="mb-4 flex items-center gap-2 border-b border-border pb-3">
        <AppIcon name="message-square" :size="18" class="text-primary" />
        <h3 class="text-sm font-semibold text-foreground">{{ $t('Configuracion.WhatsAppTemplate') }}</h3>
      </div>

      <div class="flex flex-col gap-4">
        <label class="flex flex-col gap-1.5">
          <span class="text-xs font-medium text-foreground">{{ $t('Configuracion.WhatsAppTemplate') }}</span>
          <Textarea v-model="draft.whatsAppTemplate" rows="3" class="w-full" />
        </label>

        <label class="flex flex-col gap-1.5">
          <span class="text-xs font-medium text-foreground">{{ $t('Configuracion.WhatsAppLiquidacionTemplate') }}</span>
          <Textarea v-model="draft.whatsAppLiquidacionTemplate" rows="3" class="w-full" />
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
