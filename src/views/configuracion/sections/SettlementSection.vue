<script setup lang="ts">
import InputNumber from 'primevue/inputnumber'
import ToggleSwitch from 'primevue/toggleswitch'
import { onMounted, watch } from 'vue'

import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import { useConfigForm } from '@/composables/useConfigForm'
import { useSistemaStore } from '@/stores/useSistemaStore'

const sistema = useSistemaStore()
const { draft, saving, isDirty, apply, load } = useConfigForm(
  'settlement',
  () => sistema.config?.settlement ?? null,
)

onMounted(() => load())
watch(
  () => sistema.config,
  () => load(),
)
</script>

<template>
  <form v-if="draft" class="flex max-w-4xl flex-col gap-6" @submit.prevent="apply">
    <!-- Multiplicadores -->
    <div class="rounded-lg border border-border bg-surface-card p-6 shadow-sm">
      <div class="mb-4 flex items-center gap-2 border-b border-border pb-3">
        <AppIcon name="percent" :size="18" class="text-primary" />
        <h3 class="text-sm font-semibold text-foreground">{{ $t('Configuracion.MultiplicadorSabado') }} / {{ $t('Configuracion.MultiplicadorDomingo') }}</h3>
      </div>

      <div class="grid gap-4 sm:grid-cols-3">
        <label class="flex flex-col gap-1.5">
          <span class="text-xs font-medium text-foreground">{{ $t('Configuracion.MultiplicadorSabado') }}</span>
          <InputNumber
            :model-value="draft.multiplicadorSabado ? Number(draft.multiplicadorSabado) : 0"
            :min-fraction-digits="1"
            :max-fraction-digits="2"
            :step="0.1"
            fluid
            @update:model-value="(v: number | null) => { if (draft && v !== null) draft.multiplicadorSabado = String(v) }"
          />
        </label>

        <label class="flex flex-col gap-1.5">
          <span class="text-xs font-medium text-foreground">{{ $t('Configuracion.MultiplicadorDomingo') }}</span>
          <InputNumber
            :model-value="draft.multiplicadorDomingo ? Number(draft.multiplicadorDomingo) : 0"
            :min-fraction-digits="1"
            :max-fraction-digits="2"
            :step="0.1"
            fluid
            @update:model-value="(v: number | null) => { if (draft && v !== null) draft.multiplicadorDomingo = String(v) }"
          />
        </label>

        <label class="flex flex-col gap-1.5">
          <span class="text-xs font-medium text-foreground">{{ $t('Configuracion.MultiplicadorFeriado') }}</span>
          <InputNumber
            :model-value="draft.multiplicadorFeriado ? Number(draft.multiplicadorFeriado) : 0"
            :min-fraction-digits="1"
            :max-fraction-digits="2"
            :step="0.1"
            fluid
            @update:model-value="(v: number | null) => { if (draft && v !== null) draft.multiplicadorFeriado = String(v) }"
          />
        </label>
      </div>
    </div>

    <!-- Inclusión de días -->
    <div class="rounded-lg border border-border bg-surface-card p-6 shadow-sm">
      <div class="mb-4 flex items-center gap-2 border-b border-border pb-3">
        <AppIcon name="calendar-check" :size="18" class="text-primary" />
        <h3 class="text-sm font-semibold text-foreground">{{ $t('Configuracion.IncluirSabado') }} / {{ $t('Configuracion.IncluirDomingo') }}</h3>
      </div>

      <div class="grid gap-4 sm:grid-cols-3">
        <div class="flex items-center justify-between rounded-lg border border-border bg-surface-raised p-4">
          <span class="text-xs font-medium text-foreground">{{ $t('Configuracion.IncluirSabado') }}</span>
          <ToggleSwitch v-model="draft.incluirSabado" />
        </div>

        <div class="flex items-center justify-between rounded-lg border border-border bg-surface-raised p-4">
          <span class="text-xs font-medium text-foreground">{{ $t('Configuracion.IncluirDomingo') }}</span>
          <ToggleSwitch v-model="draft.incluirDomingo" />
        </div>

        <div class="flex items-center justify-between rounded-lg border border-border bg-surface-raised p-4">
          <span class="text-xs font-medium text-foreground">{{ $t('Configuracion.IncluirFeriado') }}</span>
          <ToggleSwitch v-model="draft.incluirFeriado" />
        </div>
      </div>
    </div>

    <!-- Períodos y Sincronización -->
    <div class="rounded-lg border border-border bg-surface-card p-6 shadow-sm">
      <div class="mb-4 flex items-center gap-2 border-b border-border pb-3">
        <AppIcon name="clock" :size="18" class="text-primary" />
        <h3 class="text-sm font-semibold text-foreground">{{ $t('Configuracion.PeriodoPorDefectoDias') }}</h3>
      </div>

      <div class="grid gap-4 sm:grid-cols-2">
        <label class="flex flex-col gap-1.5">
          <span class="text-xs font-medium text-foreground">{{ $t('Configuracion.PeriodoPorDefectoDias') }}</span>
          <InputNumber v-model="draft.periodoPorDefectoDias" :min="1" :max="365" fluid />
        </label>

        <label class="flex flex-col gap-1.5">
          <span class="text-xs font-medium text-foreground">{{ $t('Configuracion.AniosFeriadosASincronizar') }}</span>
          <InputNumber v-model="draft.aniosFeriadosASincronizar" :min="1" :max="5" fluid />
        </label>

        <div class="flex items-center justify-between rounded-lg border border-border bg-surface-raised p-4 sm:col-span-2">
          <span class="text-xs font-medium text-foreground">{{ $t('Configuracion.SincronizarFeriadosAlIniciar') }}</span>
          <ToggleSwitch v-model="draft.sincronizarFeriadosAlIniciar" />
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
