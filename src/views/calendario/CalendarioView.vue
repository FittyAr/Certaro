<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useToast } from 'primevue/usetoast'
import { useApiError } from '@/composables/useApiError'
import {
  useCalendarioStore,
  type CalendarioEventoDto,
} from '@/stores/useCalendarioStore'
import { useAuthStore } from '@/stores/useAuthStore'
import { useProyectosStore } from '@/stores/useProyectosStore'
import type { LookupItem } from '@/stores/useCatalogStore'
import CalendarioEventoModal from './components/CalendarioEventoModal.vue'
import CalendarioRecursosModal from './components/CalendarioRecursosModal.vue'
import CalendarioMesGrid from './components/CalendarioMesGrid.vue'
import CalendarioSemanaGrid from './components/CalendarioSemanaGrid.vue'
import CalendarioDiaGrid from './components/CalendarioDiaGrid.vue'
import CalendarioRecursosGrid from './components/CalendarioRecursosGrid.vue'
import {
  DIAS_SEMANA,
  formatearFechaIso,
  fechaLocalIsoDe,
  useCalendarioPeriodo,
} from './composables/useCalendarioPeriodo'

const { notify } = useApiError()
const toast = useToast()

const store = useCalendarioStore()
const auth = useAuthStore()
const proyectosStore = useProyectosStore()

const {
  rangoActual,
  tituloPeriodo,
  irAnterior,
  irSiguiente,
  irHoy,
} = useCalendarioPeriodo(store)

const modalEventoAbierto = ref(false)
const modalRecursosAbierto = ref(false)
const eventoEditando = ref<CalendarioEventoDto | null>(null)
const fechaPredeterminadaModal = ref<string | undefined>(undefined)
const opcionesProyectos = ref<LookupItem[]>([])

// Load data when date or view changes
async function recargarDatos() {
  const { desde, hasta } = rangoActual.value
  await Promise.all([
    store.cargarEventos(desde, hasta),
    store.cargarRecursos(),
    store.cargarGrupos(),
  ])
}

onMounted(() => {
  recargarDatos()
  proyectosStore
    .lookup(undefined, undefined, 200)
    .then((res) => {
      opcionesProyectos.value = res
    })
    .catch(() => {
      opcionesProyectos.value = []
    })
})

watch([() => store.fechaSeleccionada, () => store.vistaActual], () => {
  recargarDatos()
})

// Days in current Month view grid
const diasCuadriculaMes = computed(() => {
  const { inicio, fin } = rangoActual.value
  const dias = []
  const actual = new Date(inicio)
  const mesActual = store.fechaSeleccionada.getMonth()
  const hoyStr = formatearFechaIso(new Date())

  while (actual <= fin) {
    const fechaIso = formatearFechaIso(actual)
    const eventosDelDia = store.eventos.filter((e) => {
      const eInicio = fechaLocalIsoDe(e.inicio)
      const eFin = fechaLocalIsoDe(e.fin)
      return fechaIso >= eInicio && fechaIso <= eFin
    })

    dias.push({
      fecha: new Date(actual),
      fechaIso,
      numeroDia: actual.getDate(),
      esMesActual: actual.getMonth() === mesActual,
      esHoy: fechaIso === hoyStr,
      eventos: eventosDelDia,
    })
    actual.setDate(actual.getDate() + 1)
  }
  return dias
})

// Days in Week view
const diasSemanaView = computed(() => {
  const { inicio } = rangoActual.value
  const dias = []
  const actual = new Date(inicio)
  const hoyStr = formatearFechaIso(new Date())

  for (let i = 0; i < 7; i++) {
    const fechaIso = formatearFechaIso(actual)
    const evs = store.eventos.filter((e) => {
      const eInicio = fechaLocalIsoDe(e.inicio)
      const eFin = fechaLocalIsoDe(e.fin)
      return fechaIso >= eInicio && fechaIso <= eFin
    })

    dias.push({
      fecha: new Date(actual),
      fechaIso,
      diaNombre: DIAS_SEMANA[i],
      numeroDia: actual.getDate(),
      esHoy: fechaIso === hoyStr,
      eventos: evs,
    })
    actual.setDate(actual.getDate() + 1)
  }
  return dias
})

// Resources for Resource Day view
const recursosActivos = computed(() => {
  return store.recursos.filter((r) => r.activo)
})

// Modal actions
function abrirCrearEvento(fechaPredeterminada?: string) {
  eventoEditando.value = null
  fechaPredeterminadaModal.value = fechaPredeterminada
  modalEventoAbierto.value = true
}

function abrirEditarEvento(ev: CalendarioEventoDto) {
  if (ev.esVirtual) {
    // Virtual items (holidays/jobs) cannot be edited directly from calendar
    return
  }
  eventoEditando.value = ev
  fechaPredeterminadaModal.value = undefined
  modalEventoAbierto.value = true
}

function abrirModalRecursos() {
  modalRecursosAbierto.value = true
}

async function ejecutarSincronizacionEmpleados() {
  try {
    await store.sincronizarEmpleados()
    toast.add({
      severity: 'success',
      summary: 'Sincronización',
      detail: 'Empleados sincronizados como recursos correctamente.',
      life: 3000,
    })
  } catch (err: unknown) {
    notify(err)
  }
}
</script>

<template>
  <div class="flex flex-col h-full bg-background text-foreground select-none">
    <!-- Header Controls -->
    <header class="flex flex-wrap items-center justify-between gap-4 p-4 border-b border-border bg-surface-card">
      <div class="flex items-center gap-3">
        <h1 class="text-xl font-bold tracking-tight">Calendario</h1>

        <!-- View Selector -->
        <div class="flex items-center rounded-lg border border-border bg-muted/40 p-0.5 text-xs font-medium">
          <button
            type="button"
            :class="[
              'px-3 py-1.5 rounded-md transition-colors',
              store.vistaActual === 'mes' ? 'bg-background shadow-xs text-foreground font-semibold' : 'text-muted-foreground hover:text-foreground'
            ]"
            @click="store.vistaActual = 'mes'"
          >
            Mes
          </button>
          <button
            type="button"
            :class="[
              'px-3 py-1.5 rounded-md transition-colors',
              store.vistaActual === 'semana' ? 'bg-background shadow-xs text-foreground font-semibold' : 'text-muted-foreground hover:text-foreground'
            ]"
            @click="store.vistaActual = 'semana'"
          >
            Semana
          </button>
          <button
            type="button"
            :class="[
              'px-3 py-1.5 rounded-md transition-colors',
              store.vistaActual === 'dia' ? 'bg-background shadow-xs text-foreground font-semibold' : 'text-muted-foreground hover:text-foreground'
            ]"
            @click="store.vistaActual = 'dia'"
          >
            Día
          </button>
          <button
            type="button"
            :class="[
              'px-3 py-1.5 rounded-md transition-colors',
              store.vistaActual === 'recursos' ? 'bg-background shadow-xs text-foreground font-semibold' : 'text-muted-foreground hover:text-foreground'
            ]"
            @click="store.vistaActual = 'recursos'"
          >
            Recursos
          </button>
        </div>
      </div>

      <!-- Navigation buttons and Title -->
      <div class="flex items-center gap-3">
        <button
          type="button"
          class="px-2.5 py-1 text-xs font-medium border border-border rounded-md hover:bg-muted transition-colors"
          @click="irHoy"
        >
          Hoy
        </button>
        <div class="flex items-center rounded-md border border-border">
          <button
            type="button"
            class="px-2.5 py-1 text-xs hover:bg-muted transition-colors rounded-l-md"
            @click="irAnterior"
          >
            &lt;
          </button>
          <button
            type="button"
            class="px-2.5 py-1 text-xs hover:bg-muted transition-colors rounded-r-md border-l border-border"
            @click="irSiguiente"
          >
            &gt;
          </button>
        </div>
        <span class="text-sm font-semibold min-w-44 text-center">{{ tituloPeriodo }}</span>
      </div>

      <!-- Actions -->
      <div class="flex items-center gap-2">
        <button
          v-if="auth.hasPermission('calendario:gestionar_recursos')"
          type="button"
          class="px-3 py-1.5 text-xs font-medium border border-border rounded-md hover:bg-muted transition-colors"
          @click="abrirModalRecursos"
        >
          Gestionar Recursos
        </button>
        <button
          v-if="auth.hasPermission('calendario:crear_evento')"
          type="button"
          class="px-3 py-1.5 text-xs font-medium bg-primary text-primary-foreground rounded-md shadow-xs hover:opacity-90 transition-opacity"
          @click="abrirCrearEvento()"
        >
          + Nuevo Evento
        </button>
      </div>
    </header>

    <!-- Content views -->
    <main class="flex-1 overflow-auto p-4">
      <!-- 1. MONTH VIEW -->
      <CalendarioMesGrid
        v-if="store.vistaActual === 'mes'"
        :dias="diasCuadriculaMes"
        @crear="abrirCrearEvento"
        @editar="abrirEditarEvento"
      />

      <!-- 2. WEEK VIEW -->
      <CalendarioSemanaGrid
        v-else-if="store.vistaActual === 'semana'"
        :dias="diasSemanaView"
        @crear="abrirCrearEvento"
        @editar="abrirEditarEvento"
      />

      <!-- 3. DAY VIEW -->
      <CalendarioDiaGrid
        v-else-if="store.vistaActual === 'dia'"
        :titulo-periodo="tituloPeriodo"
        :eventos="store.eventos"
        :fecha-seleccionada-iso="formatearFechaIso(store.fechaSeleccionada)"
        @crear="abrirCrearEvento"
        @editar="abrirEditarEvento"
      />

      <!-- 4. RESOURCE DAY VIEW -->
      <CalendarioRecursosGrid
        v-else-if="store.vistaActual === 'recursos'"
        :titulo-periodo="tituloPeriodo"
        :recursos="recursosActivos"
        :eventos="store.eventos"
        :fecha-seleccionada="store.fechaSeleccionada"
        :puede-gestionar-recursos="auth.hasPermission('calendario:gestionar_recursos')"
        @editar="abrirEditarEvento"
        @sincronizar="ejecutarSincronizacionEmpleados"
      />
    </main>

    <!-- MODAL: Crear/Editar Evento -->
    <CalendarioEventoModal
      v-model:visible="modalEventoAbierto"
      :evento="eventoEditando"
      :fecha-predeterminada="fechaPredeterminadaModal"
      :opciones-proyectos="opcionesProyectos"
    />

    <!-- MODAL: Gestión de Recursos -->
    <CalendarioRecursosModal
      v-model:visible="modalRecursosAbierto"
    />
  </div>
</template>
