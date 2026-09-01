<script setup lang="ts">
import InputGroup from 'primevue/inputgroup'
import InputGroupAddon from 'primevue/inputgroupaddon'
import InputText from 'primevue/inputtext'
import { computed, ref, watch } from 'vue'

import type { Money } from '@/api/types'
import { useMoney } from '@/composables/useMoney'
import { useConfigStore } from '@/stores/useConfigStore'
import { parseMoneyInput } from '@/lib/moneyInput'

const props = withDefaults(
  defineProps<{
    invalid?: boolean
    disabled?: boolean
    min?: number
    max?: number
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
const { format } = useMoney()
const config = useConfigStore()
const locale = computed(() => config.config?.locale)
const displayValue = ref('')
const focused = ref(false)

function display(raw: Money): string {
  return format(raw, { hideSymbol: true })
}

watch(
  () => model.value,
  (value) => {
    if (focused.value) return
    const next = display(value)
    if (next !== displayValue.value) displayValue.value = next
  },
  { immediate: true },
)

function onInput(event: Event): void {
  const input = event.target
  if (!(input instanceof HTMLInputElement)) return
  displayValue.value = input.value
  const parsed = parseMoneyInput(input.value, props.min, props.max)
  if (parsed !== null) model.value = parsed
}

function onFocus(event: FocusEvent): void {
  focused.value = true
  const input = event.target
  if (input instanceof HTMLInputElement) input.select()
}

function onBlur(): void {
  focused.value = false
  displayValue.value = display(model.value)
}
</script>

<template>
  <div v-if="!props.hideSymbol" class="w-full">
    <InputGroup class="w-full">
      <InputGroupAddon
        class="!bg-muted !text-muted-foreground !border-border !px-2.5 font-medium select-none"
      >
        {{ locale?.simboloMoneda ?? '$' }}
      </InputGroupAddon>
      <InputText
        :id="props.inputId"
        v-model="displayValue"
        :disabled="props.disabled"
        :invalid="props.invalid"
        inputmode="decimal"
        class="tabular-nums text-right"
        @input="onInput"
        @focus="onFocus"
        @blur="onBlur"
      />
    </InputGroup>
  </div>
  <InputText
    v-else
    :id="props.inputId"
    v-model="displayValue"
    :disabled="props.disabled"
    :invalid="props.invalid"
    inputmode="decimal"
    class="tabular-nums text-right w-full"
    @input="onInput"
    @focus="onFocus"
    @blur="onBlur"
  />
</template>
