<script setup lang="ts">
import InputNumber from 'primevue/inputnumber'
import { computed } from 'vue'

import type { Decimal4 } from '@/api/types'
import { useMoney } from '@/composables/useMoney'
import { useConfigStore } from '@/stores/useConfigStore'

/**
 * A `Decimal4` that is not money: a quantity, a percentage or a multiplier.
 * Same contract as `MoneyInput` — the model stays the four-decimal string the backend speaks and
 * the `number` exists only inside the widget. See `docs/16-frontend.md` §4.2.
 */
const props = withDefaults(
  defineProps<{
    invalid?: boolean
    disabled?: boolean
    min?: number
    max?: number
    /** Visible decimals. Defaults to the configured percentage precision. */
    decimals?: number
    suffix?: string
    inputId?: string
  }>(),
  {
    invalid: false,
    disabled: false,
    min: undefined,
    max: undefined,
    decimals: undefined,
    suffix: undefined,
    inputId: undefined,
  },
)

const model = defineModel<Decimal4>({ required: true })

const { toInputValue, fromInputValue } = useMoney()
const config = useConfigStore()

const locale = computed(() => config.config?.locale)
const decimals = computed(() => props.decimals ?? locale.value?.decimalesPorcentaje ?? 2)
const intlTag = computed(() => (locale.value?.separadorDecimal === '.' ? 'en-US' : 'es-AR'))

const inner = computed({
  get: () => toInputValue(model.value),
  set: (n: number | null) => {
    model.value = fromInputValue(n ?? 0)
  },
})
</script>

<template>
  <InputNumber
    v-model="inner"
    :input-id="props.inputId"
    :disabled="props.disabled"
    :invalid="props.invalid"
    :min="props.min"
    :max="props.max"
    :locale="intlTag"
    :suffix="props.suffix"
    :min-fraction-digits="0"
    :max-fraction-digits="decimals"
    :allow-empty="false"
    fluid
    input-class="tabular-nums text-right"
  />
</template>
