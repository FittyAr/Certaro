<script setup lang="ts">
import Dialog from 'primevue/dialog'
import Select from 'primevue/select'
import ToggleSwitch from 'primevue/toggleswitch'
import DateInput from '@/components/domain/DateInput.vue'
import FieldError from '@/components/domain/FieldError.vue'
import MoneyInput from '@/components/domain/MoneyInput.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import { Button } from '@/components/ui/button'
import type { LookupItem } from '@/stores/useCatalogStore'
import type { FacturaDetalle, PagoFacturaInput } from '@/stores/useFacturasStore'

defineProps<{
  visible: boolean
  factura: FacturaDetalle | null
  nuevoPago: PagoFacturaInput
  pagoErrores: Record<string, string>
  medioPagoOptions: { label: string; value: string }[]
  registrarEnCaja: boolean
  guardandoPago: boolean
  pagoProyectoId?: string | null
  pagoTrabajoId?: string | null
  opcionesProyectos?: LookupItem[]
  pagoOpcionesTrabajos?: LookupItem[]
}>()

const emit = defineEmits<{
  (e: 'update:visible', val: boolean): void
  (e: 'update:registrarEnCaja', val: boolean): void
  (e: 'update:pagoProyectoId', val: string | null): void
  (e: 'update:pagoTrabajoId', val: string | null): void
  (e: 'proyectoChange'): void
  (e: 'registrar'): void
}>()
</script>

<template>
  <Dialog
    :visible="visible"
    modal
    :header="$t('Facturas.RegistrarPago')"
    class="w-[32rem]"
    :dismissable-mask="true"
    @update:visible="emit('update:visible', $event)"
  >
    <div v-if="factura" class="flex flex-col gap-4">
      <div class="flex items-center justify-between rounded-md border border-border bg-muted/20 p-3 text-sm">
        <div>
          <span class="text-xs text-muted-foreground">{{ $t('Facturas.Numero') }}</span>
          <p class="font-semibold text-foreground">{{ factura.numero }}</p>
        </div>
        <div class="text-right">
          <span class="text-xs text-muted-foreground">{{ $t('Facturas.Saldo') }}</span>
          <p><MoneyText :value="factura.saldo" colored /></p>
        </div>
      </div>

      <div class="grid grid-cols-2 gap-3">
        <label class="flex flex-col gap-1">
          <span class="text-xs text-muted-foreground">{{ $t('Facturas.Fecha') }}</span>
          <DateInput v-model="nuevoPago.fecha" :invalid="Boolean(pagoErrores.fecha)" />
          <FieldError id="cc-pago-fecha-error" :message="pagoErrores.fecha" />
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-xs text-muted-foreground">{{ $t('Facturas.Monto') }}</span>
          <MoneyInput v-model="nuevoPago.monto" :min="0" :invalid="Boolean(pagoErrores.monto)" />
          <FieldError id="cc-pago-monto-error" :message="pagoErrores.monto" />
        </label>
      </div>

      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('Facturas.MedioPago') }}</span>
        <Select
          v-model="nuevoPago.medioPago"
          :options="medioPagoOptions"
          option-label="label"
          option-value="value"
        />
      </label>

      <label class="flex items-center gap-2 cursor-pointer select-none rounded-md border border-border/80 bg-surface-raised p-2.5">
        <ToggleSwitch
          :model-value="registrarEnCaja"
          @update:model-value="emit('update:registrarEnCaja', $event)"
        />
        <div class="text-xs">
          <span class="font-medium text-foreground block">Registrar movimiento en caja</span>
          <span class="text-muted-foreground block">Ingresa como cobranza en el libro de caja</span>
        </div>
      </label>

      <div v-if="registrarEnCaja" class="grid grid-cols-2 gap-3 rounded border border-border/70 bg-muted/20 p-2.5">
        <label class="flex flex-col gap-1">
          <span class="text-xs text-muted-foreground">Imputar a Proyecto / Obra</span>
          <Select
            :model-value="pagoProyectoId"
            :options="opcionesProyectos ?? []"
            option-label="label"
            option-value="id"
            filter
            show-clear
            placeholder="General (Sin proyecto)"
            @update:model-value="emit('update:pagoProyectoId', $event)"
            @change="emit('proyectoChange')"
          />
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-xs text-muted-foreground">Trabajo / Frente de Obra</span>
          <Select
            :model-value="pagoTrabajoId"
            :options="pagoOpcionesTrabajos ?? []"
            option-label="label"
            option-value="id"
            filter
            show-clear
            placeholder="General"
            :disabled="!pagoProyectoId && (!pagoOpcionesTrabajos || pagoOpcionesTrabajos.length === 0)"
            @update:model-value="emit('update:pagoTrabajoId', $event)"
          />
        </label>
      </div>
    </div>

    <template #footer>
      <Button variant="outline" :disabled="guardandoPago" @click="emit('update:visible', false)">
        {{ $t('General.Cancel') }}
      </Button>
      <Button
        :disabled="guardandoPago || Number(nuevoPago.monto) <= 0"
        @click="emit('registrar')"
      >
        {{ $t('Facturas.RegistrarPago') }}
      </Button>
    </template>
  </Dialog>
</template>
