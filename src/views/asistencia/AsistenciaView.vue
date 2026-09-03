<script setup lang="ts">
import Dialog from 'primevue/dialog'
import Select from 'primevue/select'
import ToggleSwitch from 'primevue/toggleswitch'
import { computed, onMounted, ref } from 'vue'

import DateInput from '@/components/domain/DateInput.vue'
import ListState from '@/components/domain/ListState.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import HelpButton from '@/components/ui/HelpButton.vue'
import { Button } from '@/components/ui/button'
import { useApiError, type ApiError } from '@/composables/useApiError'
import { useShortcuts } from '@/composables/useShortcuts'
import { useAsistenciaStore, type TipoJornada } from '@/stores/useAsistenciaStore'

/**
 * Attendance grid. See `docs/09-modulos-funcionales.md` §3.10.
 *
 * One click cycles the cell through the day types and back to empty. The write is idempotent on
 * `(empleado, fecha)`, so a fast typist clicking twice cannot create two records for one day.
 */

const { notify } = useApiError()
const store = useAsistenciaStore()

const TIPOS: TipoJornada[] = ['Completa', 'Media', 'Falta', 'FaltaJustificada', 'Feriado']

/** Single letters, because a cell is a square in a grid of thirty-one columns. */
const ABREVIATURAS: Record<TipoJornada, string> = {
  Completa: 'C',
  Media: '½',
  Falta: 'F',
  FaltaJustificada: 'J',
  Feriado: 'H',
}

/** Semantic tokens, so the palette follows the theme instead of being hardcoded per cell. */
const CLASES: Record<TipoJornada, string> = {
  Completa: 'bg-success/20 text-success',
  Media: 'bg-warning/20 text-warning',
  Falta: 'bg-destructive/20 text-destructive',
  FaltaJustificada: 'bg-state-issued/20 text-state-issued',
  Feriado: 'bg-accent/20 text-accent',
}

function primerDiaDelMes(): string {
  const now = new Date()
  return new Date(now.getFullYear(), now.getMonth(), 1).toISOString().slice(0, 10)
}

function ultimoDiaDelMes(): string {
  const now = new Date()
  return new Date(now.getFullYear(), now.getMonth() + 1, 0).toISOString().slice(0, 10)
}

const desde = ref(primerDiaDelMes())
const hasta = ref(ultimoDiaDelMes())
const loading = ref(false)
const firstLoad = ref(true)
const error = ref<ApiError | null>(null)

async function cargar(): Promise<void> {
  loading.value = true
  error.value = null
  try {
    await store.fetchGrilla({ desde: desde.value, hasta: hasta.value })
  } catch (e) {
    error.value = notify(e)
  } finally {
    loading.value = false
    firstLoad.value = false
  }
}

/** Cells being written, so a slow disk cannot let the same day be clicked twice. */
const enVuelo = ref<Set<string>>(new Set())

async function ciclar(empleadoId: string, fecha: string): Promise<void> {
  const clave = `${empleadoId}|${fecha}`
  if (enVuelo.value.has(clave)) return
  enVuelo.value.add(clave)
  try {
    await store.ciclar(empleadoId, fecha)
  } catch (e) {
    notify(e)
  } finally {
    enVuelo.value.delete(clave)
  }
}

const grilla = computed(() => store.grilla)

// ------------------------------------------------------------------ bulk entry

const rangoOpen = ref(false)
const guardando = ref(false)
const rango = ref<{
  empleadoId: string
  desde: string
  hasta: string
  tipoJornada: TipoJornada
  soloDiasHabiles: boolean
}>({
  empleadoId: '',
  desde: desde.value,
  hasta: hasta.value,
  tipoJornada: 'Completa',
  soloDiasHabiles: true,
})

const empleadosDeLaGrilla = computed(
  () => grilla.value?.filas.map((f) => ({ id: f.empleadoId, label: f.empleadoNombre })) ?? [],
)

function abrirRango(empleadoId?: string): void {
  rango.value = {
    empleadoId: empleadoId ?? empleadosDeLaGrilla.value[0]?.id ?? '',
    desde: desde.value,
    hasta: hasta.value,
    tipoJornada: 'Completa',
    soloDiasHabiles: true,
  }
  rangoOpen.value = true
}

async function guardarRango(): Promise<void> {
  if (guardando.value || !rango.value.empleadoId) return
  guardando.value = true
  try {
    await store.cargarRango({ ...rango.value, trabajoId: null })
    rangoOpen.value = false
  } catch (e) {
    notify(e)
  } finally {
    guardando.value = false
  }
}

useShortcuts({ 'ctrl+n': () => abrirRango() })

onMounted(cargar)
</script>

<template>
  <section class="flex h-full flex-col gap-4 p-6">
    <PageHeader :title="$t('Menu.Asistencia')" :subtitle="$t('Asistencia.Subtitle')">
      <template #actions>
        <Button variant="outline" @click="abrirRango()">
          <AppIcon name="calendar-plus" :size="16" />
          {{ $t('Asistencia.CargaMasiva') }}
        </Button>
        <HelpButton topic-id="asistencia-overview" title="Ayuda sobre Asistencia en Obra" />
      </template>
    </PageHeader>

    <div class="flex flex-wrap items-end gap-3">
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('General.From') }}</span>
        <DateInput v-model="desde" @update:model-value="cargar()" />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-xs text-muted-foreground">{{ $t('General.To') }}</span>
        <DateInput v-model="hasta" @update:model-value="cargar()" />
      </label>

      <div class="ml-auto flex flex-wrap items-center gap-3 text-xs">
        <span v-for="tipo in TIPOS" :key="tipo" class="flex items-center gap-1">
          <span
            class="inline-flex h-5 w-5 items-center justify-center rounded"
            :class="CLASES[tipo]"
          >
            {{ ABREVIATURAS[tipo] }}
          </span>
          {{ $t(`TipoJornada.${tipo}`) }}
        </span>
      </div>
    </div>

    <ListState
      :loading="loading"
      :first-load="firstLoad"
      :error="error"
      :is-empty="(grilla?.filas.length ?? 0) === 0"
      :is-filtered="false"
      empty-key="Asistencia.Empty"
      class="flex-1"
      @retry="cargar()"
    >
      <div v-if="grilla" class="overflow-auto">
        <table class="w-full border-collapse text-xs">
          <thead>
            <tr>
              <th
                class="sticky left-0 z-10 bg-background p-2 text-left font-medium"
                :style="{ minWidth: '12rem' }"
              >
                {{ $t('Empleados.Nombre') }}
              </th>
              <th
                v-for="dia in grilla.dias"
                :key="dia.fecha"
                class="p-1 text-center font-medium"
                :class="{
                  'text-muted-foreground': dia.esFinDeSemana,
                  'text-accent': dia.esFeriado,
                }"
                :title="dia.feriadoNombre ?? undefined"
              >
                {{ dia.fecha.slice(8) }}
              </th>
              <th class="p-2 text-right font-medium">{{ $t('Asistencia.Jornadas') }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="fila in grilla.filas" :key="fila.empleadoId" class="border-t border-border">
              <th class="sticky left-0 z-10 bg-background p-2 text-left font-normal">
                <button
                  type="button"
                  class="underline-offset-2 hover:underline"
                  :title="$t('Asistencia.CargaMasiva')"
                  @click="abrirRango(fila.empleadoId)"
                >
                  {{ fila.empleadoNombre }}
                </button>
              </th>
              <td v-for="(celda, i) in fila.celdas" :key="celda.fecha" class="p-0.5 text-center">
                <button
                  type="button"
                  class="inline-flex h-7 w-7 items-center justify-center rounded border border-border transition-colors hover:border-primary"
                  :class="celda.tipoJornada ? CLASES[celda.tipoJornada] : 'text-transparent'"
                  :aria-label="`${fila.empleadoNombre} ${celda.fecha}`"
                  :title="
                    celda.tipoJornada
                      ? $t(`TipoJornada.${celda.tipoJornada}`)
                      : $t('Asistencia.SinRegistro')
                  "
                  :disabled="grilla.dias[i] === undefined"
                  @click="ciclar(fila.empleadoId, celda.fecha)"
                >
                  {{ celda.tipoJornada ? ABREVIATURAS[celda.tipoJornada] : '·' }}
                </button>
              </td>
              <td class="p-2 text-right tabular-nums">
                {{ fila.resumen.jornadasEquivalentes }}
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </ListState>

    <Dialog v-model:visible="rangoOpen" modal :header="$t('Asistencia.CargaMasiva')">
      <div class="flex w-80 flex-col gap-3">
        <label class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('Empleados.Nombre') }}</span>
          <Select
            v-model="rango.empleadoId"
            :options="empleadosDeLaGrilla"
            option-label="label"
            option-value="id"
          />
        </label>
        <div class="grid grid-cols-2 gap-3">
          <label class="flex flex-col gap-1">
            <span class="text-sm">{{ $t('General.From') }}</span>
            <DateInput v-model="rango.desde" />
          </label>
          <label class="flex flex-col gap-1">
            <span class="text-sm">{{ $t('General.To') }}</span>
            <DateInput v-model="rango.hasta" />
          </label>
        </div>
        <label class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('Asistencia.TipoJornada') }}</span>
          <Select
            v-model="rango.tipoJornada"
            :options="TIPOS"
            :option-label="(o: TipoJornada) => $t(`TipoJornada.${o}`)"
          />
        </label>
        <label class="flex items-center gap-2 cursor-pointer select-none">
          <ToggleSwitch v-model="rango.soloDiasHabiles" />
          <span class="text-sm font-medium text-foreground/90">{{ $t('Asistencia.SoloDiasHabiles') }}</span>
        </label>
      </div>

      <template #footer>
        <Button variant="outline" :disabled="guardando" @click="rangoOpen = false">
          {{ $t('General.Cancel') }}
        </Button>
        <Button :disabled="guardando" @click="guardarRango()">{{ $t('General.Save') }}</Button>
      </template>
    </Dialog>
  </section>
</template>
