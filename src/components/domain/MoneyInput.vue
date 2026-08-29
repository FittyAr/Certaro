<script setup lang="ts">
import InputNumber from 'primevue/inputnumber'
import { computed } from 'vue'

import type { Money } from '@/api/types'
import { useMoney } from '@/composables/useMoney'
import { useConfigStore } from '@/stores/useConfigStore'

/**
 * The only place in the frontend where an amount is a `number`, and only because `InputNumber`
 * needs one. The model stays the four-decimal string the backend speaks.
 * See `docs/16-frontend.md` §4.2.
 */
const props = withDefaults(
  defineProps<{
    invalid?: boolean
    disabled?: boolean
    min?: number
    max?: number
    /** Hides the currency symbol, for quantity-like fields that happen to be amounts. */
    hideSymbol?: boolean
    inputId?: string
  }>(),
  {
    invalid: false,
    disabled: false,
    hideSymbol: false,
    min: undefined,
    max: undefined,
    inputId: undefined,
  },
)

const model = defineModel<Money>({ required: true })

const { toInputValue, fromInputValue } = useMoney()
const config = useConfigStore()

const locale = computed(() => config.config?.locale)
const decimals = computed(() => locale.value?.decimalesMoneda ?? 2)

/**
 * `InputNumber` groups digits with an Intl tag rather than with explicit separators, so the
 * configured separators are honoured by picking the tag that uses them. Spanish (Argentina) is
 * `1.234,56` and English (United States) is `1,234.56`, which covers both configurations.
 */
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
    :prefix="props.hideSymbol ? undefined : `${locale?.simboloMoneda ?? '$'} `"
    :min-fraction-digits="decimals"
    :max-fraction-digits="decimals"
    :allow-empty="false"
    fluid
    input-class="tabular-nums text-right"
  />
</template>
