<script setup lang="ts">
import Select from 'primevue/select'
import ToggleSwitch from 'primevue/toggleswitch'
import { computed, onMounted, ref, watch } from 'vue'

import DateInput from '@/components/domain/DateInput.vue'
import ListState from '@/components/domain/ListState.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import HelpButton from '@/components/ui/HelpButton.vue'
import { Button } from '@/components/ui/button'
import { useApiError, type ApiError } from '@/composables/useApiError'
import { useShortcuts } from '@/composables/useShortcuts'
import type { LookupItem } from '@/stores/useCatalogStore'
import { useProyectosStore } from '@/stores/useProyectosStore'
import { useTrabajosStore } from '@/stores/useTrabajosStore'
import { useAsistenciaStore, type TipoJornada } from '@/stores/useAsistenciaStore'
import AsistenciaTablaGrid from './components/AsistenciaTablaGrid.vue'
import AsistenciaCargaMasivaModal from './components/AsistenciaCargaMasivaModal.vue'

/**
 * Attendance grid. See `docs/09-modulos-funcionales.md` §3.10.
 *
 * One click cycles the cell through the day types and back to empty. The write is idempotent on
 * `(empleado, fecha)`, so a fast typist clicking twice cannot create two records for one day.
 */

const { notify } = useApiError()
const store = useAsistenciaStore()
const proyectosStore = useProyectosStore()
const trabajosStore = useTrabajosStore()

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

const filtroProyectoId = ref<string | null>(null)
const soloAsignados = ref(false)
const opcionesProyecto = ref<LookupItem[]>([])
const trabajosDelProyecto = ref<Set<string>>(new Set())

watch(filtroProyectoId, async (newVal) => {
  if (!newVal) {
    trabajosDelProyecto.value.clear()
    return
  }
  try {
    const trs = await trabajosStore.lookup(newVal)
    trabajosDelProyecto.value = new Set(trs.map((t) => t.id))
  } catch {
    trabajosDelProyecto.value.clear()
  }
})

const grilla = computed(() => {
  const g = store.grilla
  if (!g) return null
  if (!filtroProyectoId.value || !soloAsignados.value) return g
  const jobIds = trabajosDelProyecto.value
  const filteredFilas = g.filas.filter((fila) =>
    fila.celdas.some((c) => c.trabajoId && jobIds.has(c.trabajoId)),
  )
  return {
    ...g,
    filas: filteredFilas,
  }
})

async function cargarProyectos(): Promise<void> {
  try {
    opcionesProyecto.value = await proyectosStore.lookup(undefined, undefined, 200)
  } catch {
    opcionesProyecto.value = []
  }
}

// ------------------------------------------------------------------ bulk entry

const rangoOpen = ref(false)
const modalEmpleadoId = ref<string | undefined>(undefined)

const empleadosDeLaGrilla = computed(
  () => grilla.value?.filas.map((f) => ({ id: f.empleadoId, label: f.empleadoNombre })) ?? [],
)

function abrirRango(empleadoId?: string): void {
  modalEmpleadoId.value = empleadoId ?? empleadosDeLaGrilla.value[0]?.id
  rangoOpen.value = true
}

useShortcuts({ 'ctrl+n': () => abrirRango() })

onMounted(() => {
  void cargar()
  void cargarProyectos()
})
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
      <label class="flex flex-col gap-1 min-w-[180px]">
        <span class="text-xs text-muted-foreground">{{ $t('Menu.Proyectos') }}</span>
        <Select
          v-model="filtroProyectoId"
          :options="opcionesProyecto"
          option-label="label"
          option-value="id"
          filter
          show-clear
          :placeholder="$t('General.All')"
        />
      </label>

      <label
        v-if="filtroProyectoId"
        class="flex items-center gap-2 pb-2 text-xs text-muted-foreground cursor-pointer select-none"
      >
        <ToggleSwitch v-model="soloAsignados" />
        <span>{{ $t('Asistencia.SoloAsignados') }}</span>
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
      <AsistenciaTablaGrid
        v-if="grilla"
        :grilla="grilla"
        :clases="CLASES"
        :abreviaturas="ABREVIATURAS"
        @abrir-rango="abrirRango"
        @ciclar="ciclar"
      />
    </ListState>

    <AsistenciaCargaMasivaModal
      v-model:visible="rangoOpen"
      :empleados-opciones="empleadosDeLaGrilla"
      :opciones-proyecto="opcionesProyecto"
      :initial-empleado-id="modalEmpleadoId"
      :initial-proyecto-id="filtroProyectoId"
      :initial-desde="desde"
      :initial-hasta="hasta"
      @saved="cargar()"
    />
  </section>
</template>
