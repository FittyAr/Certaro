<script setup lang="ts">
import { computed } from 'vue'

import type { Decimal4, Money } from '@/api/types'
import MoneyText from '@/components/domain/MoneyText.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import { useMoney } from '@/composables/useMoney'

/**
 * One headline figure with its change against the previous period. See `docs/09` §3.1.
 *
 * The change arrives already computed: a `null` is not zero, it means there was no basis for
 * comparison, and it prints as a dash rather than as a growth of infinity (doc 06 §9.5).
 */
const props = withDefaults(
  defineProps<{
    label: string
    /** Either an amount or a plain count; the count is passed already as text. */
    value?: Money | null
    count?: number | null
    variacion?: Decimal4 | null
    /** Colours the amount by sign. Off for figures that are neutral by nature. */
    colored?: boolean
  }>(),
  { value: null, count: null, variacion: null, colored: false },
)

const { formatPercent, isNegative, isZero } = useMoney()

const tendencia = computed<'up' | 'down' | 'flat' | null>(() => {
  if (props.variacion === null || props.variacion === undefined) return null
  if (isZero(props.variacion)) return 'flat'
  return isNegative(props.variacion) ? 'down' : 'up'
})

const iconoTendencia = computed(() => {
  if (tendencia.value === 'up') return 'trending-up'
  return tendencia.value === 'down' ? 'trending-down' : 'minus'
})

/**
 * A drop in income is bad and a drop in expenses is good, so the caller says which direction is
 * the good one instead of the card assuming that up is always green.
 */
const claseTendencia = computed(() => {
  if (tendencia.value === 'flat' || tendencia.value === null) return 'text-muted-foreground'
  return tendencia.value === 'up' ? 'text-money-positive' : 'text-money-negative'
})
</script>

<template>
  <article class="rounded-lg border border-border bg-surface-card p-4">
    <p class="text-sm text-muted-foreground">{{ props.label }}</p>

    <p class="mt-1 text-2xl font-semibold tracking-tight">
      <MoneyText v-if="props.count === null" :value="props.value" :colored="props.colored" />
      <span v-else class="tabular-nums">{{ props.count }}</span>
    </p>

    <p v-if="tendencia" class="mt-1 flex items-center gap-1 text-xs" :class="claseTendencia">
      <AppIcon :name="iconoTendencia" :size="14" />
      <span class="tabular-nums">{{ formatPercent(props.variacion) }}</span>
      <span class="text-muted-foreground">{{ $t('Dashboard.VsAnterior') }}</span>
    </p>
    <p v-else-if="props.variacion === null && props.count === null" class="mt-1 text-xs text-muted-foreground">
      {{ $t('Dashboard.SinBase') }}
    </p>
  </article>
</template>
