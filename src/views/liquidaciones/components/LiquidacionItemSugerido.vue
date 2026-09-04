<script setup lang="ts">
import Checkbox from 'primevue/checkbox'
import Textarea from 'primevue/textarea'

import DateText from '@/components/domain/DateText.vue'
import DecimalInput from '@/components/domain/DecimalInput.vue'
import MoneyInput from '@/components/domain/MoneyInput.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import type { LiquidacionSugerencia } from '@/stores/useLiquidacionesStore'

export interface AjusteLiquidacion {
  diasTrabajados: string
  tarifaAplicada: string
  observaciones: string | null
  adelantosIncluidos: Set<string>
}

defineProps<{
  sugerencia: LiquidacionSugerencia
  ajuste: AjusteLiquidacion
  totalBruto: string
  totalNeto: string
}>()

const emit = defineEmits<{
  (e: 'alternarAdelanto', movimientoId: string): void
}>()
</script>

<template>
  <div class="space-y-3 rounded-md border border-border p-3">
    <div class="flex items-baseline justify-between">
      <h4 class="font-semibold">{{ sugerencia.empleadoNombre }}</h4>
      <span class="text-xs text-muted-foreground">
        {{ $t(`Liquidaciones.Origen.${sugerencia.origen}`) }}
      </span>
    </div>

    <p v-if="sugerencia.feriadosNoDisponibles" class="rounded bg-warning/10 p-2 text-xs text-warning">
      {{ $t('Liquidaciones.Warning.FeriadosNoDisponibles') }}
    </p>

    <div class="grid grid-cols-2 gap-3 md:grid-cols-4">
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('Liquidaciones.Dias') }}</span>
        <DecimalInput v-model="ajuste.diasTrabajados" :min="0" />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('Liquidaciones.Tarifa') }}</span>
        <MoneyInput v-model="ajuste.tarifaAplicada" :min="0" />
      </label>
      <div class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('Liquidaciones.Recargos') }}</span>
        <span class="py-2 text-sm"><MoneyText :value="sugerencia.desglose.recargos" /></span>
      </div>
      <div class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">
          {{ $t('Liquidaciones.TotalBruto') }}
        </span>
        <span class="py-2 text-sm"><MoneyText :value="totalBruto" /></span>
      </div>
    </div>

    <div v-if="(sugerencia.adelantos?.length ?? 0) > 0" class="space-y-1">
      <span class="text-xs font-medium">{{ $t('Liquidaciones.Adelantos') }}</span>
      <label
        v-for="adelanto in sugerencia.adelantos"
        :key="adelanto.movimientoId"
        class="flex items-center gap-2 text-xs"
        :class="{ 'text-muted-foreground line-through': adelanto.yaDescontado }"
      >
        <Checkbox
          :model-value="ajuste.adelantosIncluidos.has(adelanto.movimientoId)"
          binary
          :disabled="adelanto.yaDescontado"
          @update:model-value="emit('alternarAdelanto', adelanto.movimientoId)"
        />
        <DateText :value="adelanto.fecha" />
        <span class="flex-1">{{ adelanto.concepto }}</span>
        <MoneyText :value="adelanto.monto" />
        <span v-if="adelanto.yaDescontado">
          {{ $t('Liquidaciones.YaDescontado') }}
        </span>
      </label>
    </div>

    <label class="flex flex-col gap-1">
      <span class="text-xs text-muted-foreground">
        {{ $t('Liquidaciones.Observaciones') }}
      </span>
      <Textarea v-model="ajuste.observaciones" rows="2" auto-resize />
    </label>

    <div class="flex justify-end gap-2 border-t border-border pt-2 text-sm">
      <span class="text-muted-foreground">{{ $t('Liquidaciones.TotalNeto') }}</span>
      <MoneyText :value="totalNeto" />
    </div>
  </div>
</template>
