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
import type { MovimientoListItem } from '@/stores/useMovimientosStore'

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
  modoCobro?: 'nuevo' | 'anticipo'
  anticiposDisponibles?: MovimientoListItem[]
  anticipoSeleccionadoId?: string | null
}>()

const emit = defineEmits<{
  (e: 'update:visible', val: boolean): void
  (e: 'update:registrarEnCaja', val: boolean): void
  (e: 'update:pagoProyectoId', val: string | null): void
  (e: 'update:pagoTrabajoId', val: string | null): void
  (e: 'update:modoCobro', val: 'nuevo' | 'anticipo'): void
  (e: 'update:anticipoSeleccionadoId', val: string | null): void
  (e: 'anticipoChange'): void
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

      <!-- Mode switcher if advances exist -->
      <div v-if="anticiposDisponibles && anticiposDisponibles.length > 0" class="flex items-center gap-3 rounded border border-info/30 bg-info/10 p-2 text-xs">
        <span class="font-medium text-info-foreground">Modo:</span>
        <label class="flex items-center gap-1 cursor-pointer select-none">
          <input
            type="radio"
            value="nuevo"
            :checked="modoCobro !== 'anticipo'"
            class="text-primary focus:ring-0"
            @change="emit('update:modoCobro', 'nuevo')"
          />
          <span>Nuevo cobro</span>
        </label>
        <label class="flex items-center gap-1 cursor-pointer select-none">
          <input
            type="radio"
            value="anticipo"
            :checked="modoCobro === 'anticipo'"
            class="text-primary focus:ring-0"
            @change="emit('update:modoCobro', 'anticipo')"
          />
          <span class="font-semibold text-primary">
            Imputar anticipo ({{ anticiposDisponibles.length }})
          </span>
        </label>
      </div>

      <!-- Advance selector -->
      <div v-if="modoCobro === 'anticipo'" class="rounded border border-border/80 bg-surface-raised p-2.5 flex flex-col gap-1.5">
        <label class="flex flex-col gap-1">
          <span class="text-xs font-medium text-foreground">Seleccionar anticipo a imputar:</span>
          <Select
            :model-value="anticipoSeleccionadoId"
            :options="anticiposDisponibles ?? []"
            option-label="concepto"
            option-value="id"
            placeholder="Elija un anticipo existente..."
            @update:model-value="emit('update:anticipoSeleccionadoId', $event)"
            @change="emit('anticipoChange')"
          >
            <template #option="{ option }">
              <div class="flex justify-between items-center w-full gap-2 text-xs">
                <span>{{ option.fecha.slice(0, 10) }} · {{ option.concepto }}</span>
                <span class="font-semibold text-money-positive">${{ option.total }}</span>
              </div>
            </template>
          </Select>
        </label>
        <p class="text-[11px] text-muted-foreground">
          Asocia el dinero previamente recibido a esta factura sin duplicar el ingreso en caja.
        </p>
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

      <label v-if="modoCobro !== 'anticipo'" class="flex items-center gap-2 cursor-pointer select-none rounded-md border border-border/80 bg-surface-raised p-2.5">
        <ToggleSwitch
          :model-value="registrarEnCaja"
          @update:model-value="emit('update:registrarEnCaja', $event)"
        />
        <div class="text-xs">
          <span class="font-medium text-foreground block">Registrar movimiento en caja</span>
          <span class="text-muted-foreground block">Ingresa como cobranza en el libro de caja</span>
        </div>
      </label>
      <div v-else class="rounded-md border border-border/60 bg-muted/30 p-2 text-xs text-muted-foreground italic">
        El movimiento ya existe en caja (se vinculará a la factura seleccionada).
      </div>

      <div v-if="modoCobro !== 'anticipo' && registrarEnCaja" class="grid grid-cols-2 gap-3 rounded border border-border/70 bg-muted/20 p-2.5">
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
