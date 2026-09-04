<script setup lang="ts">
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import Textarea from 'primevue/textarea'
import CrudDrawer from '@/components/domain/CrudDrawer.vue'
import DateInput from '@/components/domain/DateInput.vue'
import FieldError from '@/components/domain/FieldError.vue'
import MoneyInput from '@/components/domain/MoneyInput.vue'
import type { useCrudDrawer } from '@/composables/useCrudDrawer'
import type { LookupItem } from '@/stores/useCatalogStore'
import type { FacturaInput } from '@/stores/useFacturasStore'

type Model = FacturaInput & { rowVersion?: string }

const props = defineProps<{
  drawer: ReturnType<typeof useCrudDrawer<Model>>
  opcionesCliente: LookupItem[]
}>()

function recalcularTotal(): void {
  const subtotal = Number(props.drawer.model.value.subtotal)
  const iva = Number(props.drawer.model.value.iva)
  props.drawer.model.value.total = (subtotal + iva).toFixed(4)
}

function aplicarAlicuotaIva(porcentaje: number): void {
  const subtotal = Number(props.drawer.model.value.subtotal)
  props.drawer.model.value.iva = ((subtotal * porcentaje) / 100).toFixed(4)
  recalcularTotal()
}
</script>

<template>
  <CrudDrawer :drawer="drawer" title-key="Entity.Factura">
    <div class="grid grid-cols-2 gap-3">
      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Facturas.Numero') }}</span>
        <InputText
          v-model="drawer.model.value.numero"
          :invalid="Boolean(drawer.fieldErrors.value.numero)"
          aria-describedby="fac-numero-error"
        />
        <FieldError id="fac-numero-error" :message="drawer.fieldErrors.value.numero" />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Facturas.Fecha') }}</span>
        <DateInput
          v-model="drawer.model.value.fecha"
          :invalid="Boolean(drawer.fieldErrors.value.fecha)"
        />
        <FieldError id="fac-fecha-error" :message="drawer.fieldErrors.value.fecha" />
      </label>
    </div>

    <label class="flex flex-col gap-1">
      <span class="text-sm">{{ $t('Facturas.Cliente') }}</span>
      <Select
        v-model="drawer.model.value.clienteId"
        :options="opcionesCliente"
        option-label="label"
        option-value="id"
        filter
        :invalid="Boolean(drawer.fieldErrors.value.clienteId)"
      />
      <FieldError id="fac-cliente-error" :message="drawer.fieldErrors.value.clienteId" />
    </label>

    <label class="flex flex-col gap-1">
      <span class="text-sm">{{ $t('Facturas.Vencimiento') }}</span>
      <DateInput
        v-model="drawer.model.value.fechaVencimiento"
        :invalid="Boolean(drawer.fieldErrors.value.fechaVencimiento)"
      />
      <FieldError id="fac-venc-error" :message="drawer.fieldErrors.value.fechaVencimiento" />
    </label>

    <div class="grid grid-cols-3 gap-3">
      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Facturas.Subtotal') }}</span>
        <MoneyInput
          v-model="drawer.model.value.subtotal"
          :min="0"
          :invalid="Boolean(drawer.fieldErrors.value.subtotal)"
          @update:model-value="recalcularTotal()"
        />
        <FieldError id="fac-subtotal-error" :message="drawer.fieldErrors.value.subtotal" />
      </label>
      <label class="flex flex-col gap-1">
        <div class="flex items-center justify-between">
          <span class="text-sm">{{ $t('Facturas.Iva') }}</span>
          <div class="flex gap-1 text-[11px]">
            <button
              type="button"
              class="rounded border border-border px-1 py-0.2 hover:bg-muted font-medium transition-colors"
              @click="aplicarAlicuotaIva(0)"
            >
              0%
            </button>
            <button
              type="button"
              class="rounded border border-border px-1 py-0.2 hover:bg-muted font-medium transition-colors"
              @click="aplicarAlicuotaIva(10.5)"
            >
              10.5%
            </button>
            <button
              type="button"
              class="rounded border border-border px-1 py-0.2 hover:bg-muted font-medium transition-colors"
              @click="aplicarAlicuotaIva(21)"
            >
              21%
            </button>
            <button
              type="button"
              class="rounded border border-border px-1 py-0.2 hover:bg-muted font-medium transition-colors"
              @click="aplicarAlicuotaIva(27)"
            >
              27%
            </button>
          </div>
        </div>
        <MoneyInput
          v-model="drawer.model.value.iva"
          :min="0"
          :invalid="Boolean(drawer.fieldErrors.value.iva)"
          @update:model-value="recalcularTotal()"
        />
        <FieldError id="fac-iva-error" :message="drawer.fieldErrors.value.iva" />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Facturas.Total') }}</span>
        <MoneyInput v-model="drawer.model.value.total" disabled />
      </label>
    </div>

    <label class="flex flex-col gap-1">
      <span class="text-sm">{{ $t('Facturas.Observaciones') }}</span>
      <Textarea v-model="drawer.model.value.observaciones" rows="3" auto-resize />
    </label>
  </CrudDrawer>
</template>
