<script setup lang="ts">
import { computed } from 'vue'

import type { CivilDate, Instant } from '@/api/types'
import { useDateFormat } from '@/composables/useDateFormat'

/**
 * Displays a date. A civil date is rendered as the calendar day it is, with no timezone
 * conversion; an instant is converted to local time. See `docs/16-frontend.md` §4.3.
 */
const props = withDefaults(
  defineProps<{
    value: CivilDate | Instant | null | undefined
    /** The value is an instant, not a calendar day. */
    instant?: boolean
    /** Renders the time as well. Only meaningful together with `instant`. */
    showTime?: boolean
    placeholder?: string
  }>(),
  { instant: false, showTime: false, placeholder: '—' },
)

const { formatCivil, formatInstant } = useDateFormat()

const text = computed(() => {
  if (!props.value) return props.placeholder
  return props.instant ? formatInstant(props.value, props.showTime) : formatCivil(props.value)
})
</script>

<template>
  <span class="tabular-nums">{{ text }}</span>
</template>
