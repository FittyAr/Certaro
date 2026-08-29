<script setup lang="ts">
import { computed } from 'vue'

/**
 * The state of a record, as a label plus a colour token. See `docs/16-frontend.md` §4.4.
 *
 * The value is the variant name the backend serialises, not its position: the numeric order is a
 * storage detail, and reading it here would break the day a state is appended out of order.
 */

export type StateEntity = 'Factura' | 'Obra' | 'Trabajo'

const props = defineProps<{ entity: StateEntity; value: string }>()

/** Colour token per state, from the `--state-*` set. */
const TOKENS: Record<StateEntity, Record<string, string>> = {
  Factura: {
    Borrador: 'draft',
    Emitida: 'issued',
    PagadaParcial: 'partial',
    Pagada: 'paid',
    Vencida: 'overdue',
    Anulada: 'void',
  },
  Obra: {
    Activa: 'active',
    Pausada: 'paused',
    Finalizada: 'finished',
    Cancelada: 'cancelled',
  },
  Trabajo: {
    Presupuestado: 'draft',
    EnProceso: 'active',
    Pausado: 'paused',
    Finalizado: 'finished',
    Cancelado: 'cancelled',
  },
}

const token = computed(() => TOKENS[props.entity][props.value] ?? 'void')
const labelKey = computed(() => `State.${props.entity}.${props.value}`)
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
