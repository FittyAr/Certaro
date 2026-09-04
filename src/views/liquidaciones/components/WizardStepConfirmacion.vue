<script setup lang="ts">
import Checkbox from 'primevue/checkbox'
import Select from 'primevue/select'
import MoneyText from '@/components/domain/MoneyText.vue'
import type { LookupItem } from '@/stores/useCatalogStore'
import type { LiquidacionInput, LiquidacionSugerencia } from '@/stores/useLiquidacionesStore'

defineProps<{
  sugerencias: LiquidacionSugerencia[]
  dtoDe: (s: LiquidacionSugerencia) => LiquidacionInput
  totalAdelantosDe: (s: LiquidacionSugerencia) => string
  totalNetoDelLote: string
  registrarEnCaja: boolean
  medioPago: 'Efectivo' | 'Transferencia' | 'Cheque'
  categoriaGastoId: string | null
  pagoProyectoId: string | null
  pagoTrabajoId: string | null
  categoriasOpciones: LookupItem[]
  opcionesProyecto: LookupItem[]
  opcionesTrabajo: LookupItem[]
  mediosPagoOpciones: { label: string; value: string }[]
  hayImputacionIndividual?: boolean
}>()

const emit = defineEmits<{
  (e: 'update:registrarEnCaja', val: boolean): void
  (e: 'update:medioPago', val: 'Efectivo' | 'Transferencia' | 'Cheque'): void
  (e: 'update:categoriaGastoId', val: string | null): void
  (e: 'update:pagoProyectoId', val: string | null): void
  (e: 'update:pagoTrabajoId', val: string | null): void
  (e: 'proyectoChange'): void
}>()
</script>

<template>
  <div class="space-y-3">
    <p class="text-sm">
      {{ $t('Liquidaciones.ConfirmarTexto', { cantidad: sugerencias?.length ?? 0 }) }}
    </p>
    <ul class="divide-y divide-border text-sm">
      <li
        v-for="s in sugerencias"
        :key="s.empleadoId"
        class="flex items-center justify-between py-2"
      >
        <span>{{ s.empleadoNombre }}</span>
        <MoneyText
          :value="(Number(dtoDe(s).totalBruto) - Number(totalAdelantosDe(s))).toFixed(4)"
        />
      </li>
    </ul>
    <div class="flex justify-end gap-2 border-t border-border pt-2 font-medium">
      <span>{{ $t('Liquidaciones.TotalDelLote') }}</span>
      <MoneyText :value="totalNetoDelLote" />
    </div>

    <!-- Cash ledger outflow options -->
    <div class="mt-4 rounded-lg border border-border bg-card/60 p-3 space-y-3">
      <label class="flex items-center gap-2 cursor-pointer font-medium text-sm">
        <Checkbox
          :model-value="registrarEnCaja"
          binary
          @update:model-value="emit('update:registrarEnCaja', $event)"
        />
        <span>{{ $t('Liquidaciones.RegistrarEnCaja') }}</span>
      </label>

      <div v-if="registrarEnCaja" class="pt-1 space-y-3">
        <div v-if="hayImputacionIndividual" class="rounded border border-primary/30 bg-primary/10 p-2.5 text-xs text-primary flex items-center gap-2">
          <span>ℹ️ {{ $t('Liquidaciones.MovimientosMultiples') }}</span>
        </div>

        <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
          <label class="flex flex-col gap-1 text-xs">
            <span class="text-muted-foreground">{{ $t('Liquidaciones.MedioPago') }}</span>
            <Select
              :model-value="medioPago"
              :options="mediosPagoOpciones"
              option-label="label"
              option-value="value"
              @update:model-value="emit('update:medioPago', $event)"
            />
          </label>
          <label class="flex flex-col gap-1 text-xs">
            <span class="text-muted-foreground">{{ $t('Liquidaciones.Categoria') }}</span>
            <Select
              :model-value="categoriaGastoId"
              :options="categoriasOpciones"
              option-label="label"
              option-value="id"
              filter
              show-clear
              placeholder="Seleccionar categoría"
              @update:model-value="emit('update:categoriaGastoId', $event)"
            />
          </label>
          <label class="flex flex-col gap-1 text-xs">
            <span class="text-muted-foreground">
              {{ hayImputacionIndividual ? $t('Liquidaciones.ImputarLoteGeneral') : $t('Liquidaciones.ImputarProyecto') }}
            </span>
            <Select
              :model-value="pagoProyectoId"
              :options="opcionesProyecto"
              option-label="label"
              option-value="id"
              filter
              show-clear
              :placeholder="$t('General.None')"
              @update:model-value="emit('update:pagoProyectoId', $event)"
              @change="emit('proyectoChange')"
            />
          </label>
          <label class="flex flex-col gap-1 text-xs">
            <span class="text-muted-foreground">{{ $t('Liquidaciones.ImputarTrabajo') }}</span>
            <Select
              :model-value="pagoTrabajoId"
              :options="opcionesTrabajo"
              option-label="label"
              option-value="id"
              filter
              show-clear
              :placeholder="$t('General.None')"
              :disabled="!pagoProyectoId && opcionesTrabajo.length === 0"
              @update:model-value="emit('update:pagoTrabajoId', $event)"
            />
          </label>
        </div>
      </div>
    </div>
  </div>
</template>
