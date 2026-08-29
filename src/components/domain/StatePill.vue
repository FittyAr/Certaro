<script setup lang="ts">
import { computed } from 'vue'

/**
 * The state of a record, as a label plus a colour token. See `docs/16-frontend.md` §4.4.
 *
 * The mapping lives here and nowhere else. The legacy system had the state labels written in
 * Spanish inside a converter; here the label comes from i18n and the colour from a token, and a
 * new state has to be added in three places that a test checks.
 */

export type StateEntity = 'factura' | 'obra' | 'trabajo'

const props = defineProps<{ entity: StateEntity; value: number }>()

/** Variant name per numeric enum value, in the order the Rust enums declare them. */
const VARIANTS: Record<StateEntity, string[]> = {
  factura: ['Borrador', 'Emitida', 'PagadaParcial', 'Pagada', 'Vencida', 'Anulada'],
  obra: ['Activa', 'Pausada', 'Finalizada', 'Cancelada'],
  trabajo: ['Pendiente', 'EnProgreso', 'Pausado', 'Finalizado', 'Cancelado'],
}

/** Colour token per state, from the `--state-*` set. */
const TOKENS: Record<StateEntity, string[]> = {
  factura: ['draft', 'issued', 'partial', 'paid', 'overdue', 'void'],
  obra: ['active', 'paused', 'finished', 'cancelled'],
  trabajo: ['draft', 'active', 'paused', 'finished', 'cancelled'],
}

const variant = computed(() => VARIANTS[props.entity][props.value] ?? 'Unknown')
const token = computed(() => TOKENS[props.entity][props.value] ?? 'void')
const labelKey = computed(
  () => `State.${props.entity.charAt(0).toUpperCase()}${props.entity.slice(1)}.${variant.value}`,
)
</script>

<template>
  <!-- The label is always there: no information in this system is carried by colour alone. -->
  <span
    class="inline-flex items-center gap-1.5 rounded-full border border-border px-2 py-0.5 text-xs"
  >
    <span
      class="h-2 w-2 rounded-full"
      :style="{ backgroundColor: `hsl(var(--state-${token}))` }"
      aria-hidden="true"
    />
    {{ $t(labelKey) }}
  </span>
</template>
