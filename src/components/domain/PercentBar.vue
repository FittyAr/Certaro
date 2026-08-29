<script setup lang="ts">
import { computed } from 'vue'

import type { Decimal4 } from '@/api/types'
import { useMoney } from '@/composables/useMoney'

/**
 * A percentage as a bar plus its number. See `docs/16-frontend.md` §4.5.
 *
 * Over 100 the bar is coloured with the overdue token and is **not** clipped: imported data can
 * carry a wrong figure and hiding it would make it impossible to find.
 */
const props = defineProps<{ value: Decimal4 | null | undefined }>()

const { formatPercent } = useMoney()

const numeric = computed(() => Number(props.value ?? '0'))
const width = computed(() => `${Math.min(Math.max(numeric.value, 0), 100)}%`)
const overflowing = computed(() => numeric.value > 100)
</script>

<template>
  <div class="flex items-center gap-2">
    <div class="h-2 w-full overflow-hidden rounded-full bg-muted">
      <div
        class="h-full rounded-full"
        :style="{
          width,
          backgroundColor: overflowing ? 'hsl(var(--state-overdue))' : 'hsl(var(--primary))',
        }"
      />
    </div>
    <span class="w-16 text-right text-sm tabular-nums">{{ formatPercent(props.value) }}</span>
  </div>
</template>
