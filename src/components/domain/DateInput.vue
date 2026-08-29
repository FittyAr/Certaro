<script setup lang="ts">
import DatePicker from 'primevue/datepicker'
import { computed } from 'vue'

import type { CivilDate, Instant } from '@/api/types'
import { useDateFormat } from '@/composables/useDateFormat'
import { useConfigStore } from '@/stores/useConfigStore'

/**
 * Wraps `DatePicker` and converts at both edges, so no view ever handles a JavaScript `Date`.
 *
 * With `instant` off the model is a `YYYY-MM-DD` civil date and no timezone is involved at all;
 * with it on the model is an ISO-8601 UTC instant. See `docs/16-frontend.md` §4.3.
 */
const props = withDefaults(
  defineProps<{
    instant?: boolean
    showTime?: boolean
    invalid?: boolean
    disabled?: boolean
    inputId?: string
    minDate?: Date
    maxDate?: Date
  }>(),
  {
    instant: false,
    showTime: false,
    invalid: false,
    disabled: false,
    inputId: undefined,
    minDate: undefined,
    maxDate: undefined,
  },
)

/** `undefined` is accepted so an optional filter field can be bound directly. */
const model = defineModel<CivilDate | Instant | null | undefined>({ required: true })

const { civilToDate, dateToCivil, instantToDate, dateToInstant } = useDateFormat()
const config = useConfigStore()

/** `DatePicker` speaks the same `dd/mm/yy` vocabulary in lowercase. */
const pickerFormat = computed(() =>
  (config.config?.locale.formatoFecha ?? 'dd/MM/yyyy')
    .replace('yyyy', 'yy')
    .replace('MM', 'mm')
    .toLowerCase(),
)

const firstDayOfWeek = computed(() => config.config?.locale.primerDiaSemana ?? 1)

const inner = computed({
  get: () =>
    props.instant ? instantToDate(model.value ?? null) : civilToDate(model.value ?? null),
  set: (date: Date | null) => {
    model.value = props.instant ? dateToInstant(date) : dateToCivil(date)
  },
})
</script>

<template>
  <DatePicker
    v-model="inner"
    :input-id="props.inputId"
    :disabled="props.disabled"
    :invalid="props.invalid"
    :show-time="props.instant && props.showTime"
    :date-format="pickerFormat"
    :first-day-of-week="firstDayOfWeek"
    :min-date="props.minDate"
    :max-date="props.maxDate"
    show-icon
    icon-display="input"
    show-button-bar
    fluid
  />
</template>
