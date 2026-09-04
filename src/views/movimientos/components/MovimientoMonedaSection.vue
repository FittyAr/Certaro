<script setup lang="ts">
import Select from 'primevue/select'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import FieldError from '@/components/domain/FieldError.vue'
import MoneyInput from '@/components/domain/MoneyInput.vue'
import type { Moneda } from '@/stores/useMovimientosStore'

const props = defineProps<{
  moneda: Moneda
  cotizacionAplicada: string | null
  fieldErrorCotizacion?: string
}>()

const emit = defineEmits<{
  (e: 'update:moneda', value: Moneda): void
  (e: 'update:cotizacionAplicada', value: string | null): void
}>()

const { t } = useI18n()

const monedaOptions = computed<{ label: string; value: Moneda }[]>(() => [
  { label: t('Movimientos.Moneda.Ars'), value: 'Ars' },
  { label: t('Movimientos.Moneda.Usd'), value: 'Usd' },
])

const pideCotizacion = computed(() => props.moneda === 'Usd')

function onMonedaChange(value: Moneda) {
  emit('update:moneda', value)
  if (value !== 'Usd') {
    emit('update:cotizacionAplicada', null)
  }
}
</script>

<template>
  <div class="grid grid-cols-2 gap-3">
    <label class="flex flex-col gap-1">
      <span class="text-sm">{{ $t('Movimientos.Moneda.Label') }}</span>
      <Select
        :model-value="moneda"
        :options="monedaOptions"
        option-label="label"
        option-value="value"
        @update:model-value="(val) => onMonedaChange(val)"
      />
    </label>

    <label v-if="pideCotizacion" class="flex flex-col gap-1">
      <span class="text-sm">{{ $t('Movimientos.Cotizacion') }}</span>
      <MoneyInput
        :model-value="cotizacionAplicada ?? '0.0000'"
        :min="0"
        :invalid="Boolean(fieldErrorCotizacion)"
        @update:model-value="(value) => emit('update:cotizacionAplicada', value)"
      />
      <FieldError
        id="mov-cotizacion-error"
        :message="fieldErrorCotizacion"
      />
    </label>
  </div>
</template>
