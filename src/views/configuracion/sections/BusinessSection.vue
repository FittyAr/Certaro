<script setup lang="ts">
import InputText from 'primevue/inputtext'
import { onMounted, watch } from 'vue'

import { Button } from '@/components/ui/button'
import { useConfigForm } from '@/composables/useConfigForm'
import { useSistemaStore } from '@/stores/useSistemaStore'

const sistema = useSistemaStore()
const { draft, saving, isDirty, apply, load } = useConfigForm(() => sistema.config?.business ?? null)

onMounted(() => load())
watch(() => sistema.config, () => load())
</script>

<template>
  <form v-if="draft" class="flex max-w-xl flex-col gap-4 p-4" @submit.prevent="apply">
    <label class="flex flex-col gap-1">
      <span class="text-sm">Nombre comercial</span>
      <InputText v-model="draft.nombreComercial" />
    </label>

    <label class="flex flex-col gap-1">
      <span class="text-sm">Lema</span>
      <InputText v-model="draft.lema" />
    </label>

    <label class="flex flex-col gap-1">
      <span class="text-sm">Contratista</span>
      <InputText v-model="draft.contratista" />
    </label>

    <div class="grid grid-cols-2 gap-4">
      <label class="flex flex-col gap-1">
        <span class="text-sm">CUIT</span>
        <InputText v-model="draft.cuit" />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-sm">Teléfono</span>
        <InputText v-model="draft.telefono" />
      </label>
    </div>

    <label class="flex flex-col gap-1">
      <span class="text-sm">Dirección</span>
      <InputText v-model="draft.direccion" />
    </label>

    <label class="flex flex-col gap-1">
      <span class="text-sm">Email</span>
      <InputText v-model="draft.email" />
    </label>

    <div class="grid grid-cols-2 gap-4">
      <label class="flex flex-col gap-1">
        <span class="text-sm">IVA sugerido (%)</span>
        <InputText v-model="draft.ivaSugerido" />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-sm">Días vencimiento factura</span>
        <InputText :model-value="String(draft.facturaDiasVencimientoDefault)" @update:model-value="(v: string | undefined) => { if (draft && v !== undefined) draft.facturaDiasVencimientoDefault = Number(v) }" />
      </label>
    </div>

    <div class="flex justify-end">
      <Button :disabled="!isDirty || saving" @click="apply">
        {{ $t('Configuracion.Apply') }}
      </Button>
    </div>
  </form>
</template>
