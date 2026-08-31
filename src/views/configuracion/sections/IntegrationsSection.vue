<script setup lang="ts">
import InputText from 'primevue/inputtext'
import InputNumber from 'primevue/inputnumber'
import ToggleSwitch from 'primevue/toggleswitch'
import { onMounted, watch } from 'vue'

import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import { useConfigForm } from '@/composables/useConfigForm'
import { useSistemaStore } from '@/stores/useSistemaStore'

const sistema = useSistemaStore()
const { draft, saving, isDirty, apply, load } = useConfigForm(
  'externalApis',
  () => sistema.config?.externalApis ?? null,
)

onMounted(() => load())
watch(
  () => sistema.config,
  () => load(),
)
</script>

<template>
  <form v-if="draft" class="flex max-w-4xl flex-col gap-6" @submit.prevent="apply">
    <!-- APIs Externas -->
    <div class="rounded-lg border border-border bg-surface-card p-6 shadow-sm">
      <div class="mb-4 flex items-center gap-2 border-b border-border pb-3">
        <AppIcon name="globe" :size="18" class="text-primary" />
        <h3 class="text-sm font-semibold text-foreground">{{ $t('Configuracion.DollarUrl') }} / {{ $t('Configuracion.HolidayUrl') }}</h3>
      </div>

      <div class="grid gap-4 sm:grid-cols-1">
        <label class="flex flex-col gap-1.5">
          <span class="text-xs font-medium text-foreground">{{ $t('Configuracion.DollarUrl') }}</span>
          <InputText v-model="draft.dollarUrl" class="w-full" />
        </label>

        <label class="flex flex-col gap-1.5">
          <span class="text-xs font-medium text-foreground">{{ $t('Configuracion.HolidayUrl') }}</span>
          <InputText v-model="draft.holidayUrl" class="w-full" />
        </label>
      </div>
    </div>

    <!-- Parámetros de Red -->
    <div class="rounded-lg border border-border bg-surface-card p-6 shadow-sm">
      <div class="mb-4 flex items-center gap-2 border-b border-border pb-3">
        <AppIcon name="wifi" :size="18" class="text-primary" />
        <h3 class="text-sm font-semibold text-foreground">{{ $t('Configuracion.TimeoutSeconds') }} / {{ $t('Configuracion.Reintentos') }}</h3>
      </div>

      <div class="grid gap-4 sm:grid-cols-2">
        <label class="flex flex-col gap-1.5">
          <span class="text-xs font-medium text-foreground">{{ $t('Configuracion.TimeoutSeconds') }}</span>
          <InputNumber v-model="draft.timeoutSeconds" :min="1" :max="120" fluid />
        </label>

        <label class="flex flex-col gap-1.5">
          <span class="text-xs font-medium text-foreground">{{ $t('Configuracion.Reintentos') }}</span>
          <InputNumber v-model="draft.reintentos" :min="0" :max="5" fluid />
        </label>

        <label class="flex flex-col gap-1.5">
          <span class="text-xs font-medium text-foreground">{{ $t('Configuracion.DollarCacheMinutes') }}</span>
          <InputNumber v-model="draft.dollarCacheMinutes" :min="1" :max="1440" fluid />
        </label>

        <div class="flex items-center justify-between rounded-lg border border-border bg-surface-raised p-4">
          <span class="text-xs font-medium text-foreground">{{ $t('Configuracion.DollarAutoUpdate') }}</span>
          <ToggleSwitch v-model="draft.dollarAutoUpdate" />
        </div>
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
