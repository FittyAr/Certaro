<script setup lang="ts">
import { computed } from 'vue'

import type { Money } from '@/api/types'
import { useMoney } from '@/composables/useMoney'

/**
 * Displays an amount. Colour comes from a token chosen by the sign, never from the view.
 * See `docs/16-frontend.md` §4.2.
 */
const props = withDefaults(
  defineProps<{
    value: Money | null | undefined
    /** Colours the value by sign. Off for neutral figures such as a budget. */
    colored?: boolean
    showSign?: boolean
    hideSymbol?: boolean
    /** Placeholder when there is no value. */
    placeholder?: string
  }>(),
  { colored: false, showSign: false, hideSymbol: false, placeholder: '—' },
)

const { format, isNegative, isZero } = useMoney()

const text = computed(() =>
  props.value === null || props.value === undefined || props.value === ''
    ? props.placeholder
    : format(props.value, { showSign: props.showSign, hideSymbol: props.hideSymbol }),
)

const colorClass = computed(() => {
  if (!props.colored) return ''
  if (isZero(props.value)) return 'text-money-neutral'
  return isNegative(props.value) ? 'text-money-negative' : 'text-money-positive'
})
</script>

<template>
  <span class="tabular-nums" :class="colorClass">{{ text }}</span>
</template>
