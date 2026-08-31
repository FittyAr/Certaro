<script setup lang="ts">
import InputText from 'primevue/inputtext'
import InputNumber from 'primevue/inputnumber'
import { onMounted, watch } from 'vue'

import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import { useConfigForm } from '@/composables/useConfigForm'
import { useSistemaStore } from '@/stores/useSistemaStore'

const sistema = useSistemaStore()
const { draft, saving, isDirty, apply, load } = useConfigForm(
  'business',
  () => sistema.config?.business ?? null,
)

onMounted(() => load())
watch(
  () => sistema.config,
  () => load(),
)
</script>

<template>
  <form v-if="draft" class="flex max-w-4xl flex-col gap-6" @submit.prevent="apply">
    <!-- Empresa -->
    <div class="rounded-lg border border-border bg-surface-card p-6 shadow-sm">
      <div class="mb-4 flex items-center gap-2 border-b border-border pb-3">
        <AppIcon name="building-2" :size="18" class="text-primary" />
        <h3 class="text-sm font-semibold text-foreground">{{ $t('Configuracion.NombreComercial') }}</h3>
      </div>

      <div class="grid gap-4 sm:grid-cols-2">
        <label class="flex flex-col gap-1.5 sm:col-span-2">
          <span class="text-xs font-medium text-foreground">{{ $t('Configuracion.NombreComercial') }}</span>
          <InputText v-model="draft.nombreComercial" class="w-full" />
        </label>

        <label class="flex flex-col gap-1.5 sm:col-span-2">
          <span class="text-xs font-medium text-foreground">{{ $t('Configuracion.Lema') }}</span>
          <InputText v-model="draft.lema" class="w-full" />
        </label>

        <label class="flex flex-col gap-1.5">
          <span class="text-xs font-medium text-foreground">{{ $t('Configuracion.Contratista') }}</span>
          <InputText v-model="draft.contratista" class="w-full" />
        </label>

        <label class="flex flex-col gap-1.5">
          <span class="text-xs font-medium text-foreground">{{ $t('Configuracion.Cuit') }}</span>
          <InputText v-model="draft.cuit" class="w-full" />
        </label>
      </div>
    </div>

    <!-- Contacto -->
    <div class="rounded-lg border border-border bg-surface-card p-6 shadow-sm">
      <div class="mb-4 flex items-center gap-2 border-b border-border pb-3">
        <AppIcon name="phone" :size="18" class="text-primary" />
        <h3 class="text-sm font-semibold text-foreground">{{ $t('Configuracion.Telefono') }} / {{ $t('Configuracion.Direccion') }}</h3>
      </div>

      <div class="grid gap-4 sm:grid-cols-2">
        <label class="flex flex-col gap-1.5">
          <span class="text-xs font-medium text-foreground">{{ $t('Configuracion.Telefono') }}</span>
          <InputText v-model="draft.telefono" class="w-full" />
        </label>

        <label class="flex flex-col gap-1.5">
          <span class="text-xs font-medium text-foreground">{{ $t('Configuracion.Email') }}</span>
          <InputText v-model="draft.email" class="w-full" />
        </label>

        <label class="flex flex-col gap-1.5 sm:col-span-2">
          <span class="text-xs font-medium text-foreground">{{ $t('Configuracion.Direccion') }}</span>
          <InputText v-model="draft.direccion" class="w-full" />
        </label>
      </div>
    </div>

    <!-- Facturación -->
    <div class="rounded-lg border border-border bg-surface-card p-6 shadow-sm">
      <div class="mb-4 flex items-center gap-2 border-b border-border pb-3">
        <AppIcon name="receipt" :size="18" class="text-primary" />
        <h3 class="text-sm font-semibold text-foreground">{{ $t('Configuracion.IvaSugerido') }}</h3>
      </div>

      <div class="grid gap-4 sm:grid-cols-2">
        <label class="flex flex-col gap-1.5">
          <span class="text-xs font-medium text-foreground">{{ $t('Configuracion.IvaSugerido') }}</span>
          <InputNumber
            :model-value="draft.ivaSugerido ? Number(draft.ivaSugerido) : 0"
            :min-fraction-digits="0"
            :max-fraction-digits="2"
            fluid
            @update:model-value="(v: number | null) => { if (draft && v !== null) draft.ivaSugerido = String(v) }"
          />
        </label>

        <label class="flex flex-col gap-1.5">
          <span class="text-xs font-medium text-foreground">{{ $t('Configuracion.FacturaDiasVencimiento') }}</span>
          <InputNumber v-model="draft.facturaDiasVencimientoDefault" :min="1" :max="365" fluid />
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
