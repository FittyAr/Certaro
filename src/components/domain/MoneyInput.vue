<script setup lang="ts">
import InputGroup from 'primevue/inputgroup'
import InputGroupAddon from 'primevue/inputgroupaddon'
import InputNumber from 'primevue/inputnumber'
import { computed, ref, watch } from 'vue'

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

const currentNumber = ref<number | null>(model.value ? toInputValue(model.value) : null)

watch(
  () => model.value,
  (newVal) => {
    const parsed = newVal ? toInputValue(newVal) : null
    if (parsed !== currentNumber.value) {
      currentNumber.value = parsed
    }
  },
  { immediate: true },
)

function onUpdate(val: number | null | undefined): void {
  currentNumber.value = val ?? null
  model.value = val === null || val === undefined ? '0.0000' : fromInputValue(val)
}
</script>

<template>
  <InputGroup v-if="!props.hideSymbol" class="w-full">
    <InputGroupAddon
      class="!bg-muted !text-muted-foreground !border-border !px-2.5 font-medium select-none"
    >
      {{ locale?.simboloMoneda ?? '$' }}
    </InputGroupAddon>
    <InputNumber
      :model-value="currentNumber"
      :input-id="props.inputId"
      :disabled="props.disabled"
      :invalid="props.invalid"
      :min="props.min"
      :max="props.max"
      :locale="intlTag"
      :min-fraction-digits="decimals"
      :max-fraction-digits="decimals"
      :allow-empty="true"
      highlight-on-focus
      fluid
      input-class="tabular-nums text-right"
      @update:model-value="onUpdate"
    />
  </InputGroup>
  <InputNumber
    v-else
    :model-value="currentNumber"
    :input-id="props.inputId"
    :disabled="props.disabled"
    :invalid="props.invalid"
    :min="props.min"
    :max="props.max"
    :locale="intlTag"
    :min-fraction-digits="decimals"
    :max-fraction-digits="decimals"
    :allow-empty="true"
    highlight-on-focus
    fluid
    input-class="tabular-nums text-right"
    @update:model-value="onUpdate"
  />
</template>
