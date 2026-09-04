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
import {
  DIAS_SEMANA,
  HORAS_DIA,
  pad,
  formatearFechaIso,
  fechaLocalIsoDe,
  formatearHoraLocal,
  coincideHora,
  useCalendarioPeriodo,
} from './composables/useCalendarioPeriodo'
import { getBadgeClass } from './composables/useBadgeClass'

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

function eventosDeRecurso(recursoId: string) {
  const diaIso = formatearFechaIso(store.fechaSeleccionada)
  return store.eventos.filter((e) => {
    const eInicio = fechaLocalIsoDe(e.inicio)
    const eFin = fechaLocalIsoDe(e.fin)
    const coincideDia = diaIso >= eInicio && diaIso <= eFin
    const tieneRecurso = e.recursos.some((r) => r.id === recursoId)
    return coincideDia && tieneRecurso
  })
}

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
      <div v-if="store.vistaActual === 'mes'" class="flex flex-col h-full border border-border rounded-lg bg-surface-card overflow-hidden">
        <!-- Weekday headers -->
        <div class="grid grid-cols-7 border-b border-border bg-muted/30 text-center text-xs font-semibold py-2">
          <div v-for="dia in DIAS_SEMANA" :key="dia">{{ dia }}</div>
        </div>

        <!-- Month Day Cells -->
        <div class="grid grid-cols-7 flex-1 auto-rows-fr">
          <div
            v-for="dia in diasCuadriculaMes"
            :key="dia.fechaIso"
            :class="[
              'border-b border-r border-border p-1.5 flex flex-col min-h-24 transition-colors cursor-pointer hover:bg-muted/20',
              !dia.esMesActual ? 'opacity-40 bg-muted/10' : '',
              dia.esHoy ? 'bg-primary/5' : ''
            ]"
            @click="abrirCrearEvento(dia.fechaIso)"
          >
            <div class="flex items-center justify-between mb-1">
              <span
                :class="[
                  'text-xs font-medium px-1.5 py-0.5 rounded-full',
                  dia.esHoy ? 'bg-primary text-primary-foreground font-bold' : 'text-muted-foreground'
                ]"
              >
                {{ dia.numeroDia }}
              </span>
              <span v-if="dia.eventos.length > 3" class="text-[10px] text-muted-foreground font-medium">
                +{{ dia.eventos.length - 3 }}
              </span>
            </div>

            <!-- Event Pills (max 3 displayed) -->
            <div class="flex flex-col gap-1 overflow-hidden">
              <div
                v-for="ev in dia.eventos.slice(0, 3)"
                :key="ev.id"
                :class="[
                  'text-[11px] px-1.5 py-0.5 rounded-md border truncate font-medium flex items-center justify-between cursor-pointer hover:opacity-80',
                  getBadgeClass(ev.tipo, ev.esVirtual)
                ]"
                @click.stop="abrirEditarEvento(ev)"
              >
                <span class="truncate">{{ ev.titulo }}</span>
                <span v-if="ev.esVirtual" class="text-[9px] uppercase tracking-wider font-semibold opacity-70">
                  Virtual
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 2. WEEK VIEW -->
      <div v-else-if="store.vistaActual === 'semana'" class="flex flex-col h-full border border-border rounded-lg bg-surface-card overflow-hidden">
        <!-- Week header -->
        <div class="grid grid-cols-8 border-b border-border bg-muted/30 text-xs font-semibold py-2">
          <div class="text-center text-muted-foreground">Hora</div>
          <div
            v-for="dia in diasSemanaView"
            :key="dia.fechaIso"
            :class="['text-center', dia.esHoy ? 'text-primary font-bold' : '']"
          >
            {{ dia.diaNombre }} {{ dia.numeroDia }}
          </div>
        </div>

        <!-- Hourly rows -->
        <div class="flex-1 overflow-y-auto">
          <div
            v-for="hora in HORAS_DIA"
            :key="hora"
            class="grid grid-cols-8 border-b border-border min-h-12 text-xs"
          >
            <div class="border-r border-border p-1 text-center text-muted-foreground text-[11px] font-mono">
              {{ pad(hora) }}:00
            </div>
            <div
              v-for="dia in diasSemanaView"
              :key="dia.fechaIso"
              class="border-r border-border p-1 flex flex-col gap-1 hover:bg-muted/10 cursor-pointer"
              @click="abrirCrearEvento(`${dia.fechaIso}T${pad(hora)}:00`)"
            >
              <div
                v-for="ev in dia.eventos.filter(e => coincideHora(e.inicio, hora))"
                :key="ev.id"
                :class="[
                  'text-[11px] px-1.5 py-0.5 rounded-md border font-medium truncate hover:opacity-80',
                  getBadgeClass(ev.tipo, ev.esVirtual)
                ]"
                @click.stop="abrirEditarEvento(ev)"
              >
                {{ ev.titulo }}
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 3. DAY VIEW -->
      <div v-else-if="store.vistaActual === 'dia'" class="flex flex-col h-full border border-border rounded-lg bg-surface-card overflow-hidden">
        <div class="border-b border-border bg-muted/30 p-3 text-sm font-semibold flex items-center justify-between">
          <span>Detalle de Agenda: {{ tituloPeriodo }}</span>
          <span class="text-xs text-muted-foreground font-normal">
            {{ store.eventos.length }} eventos programados
          </span>
        </div>
        <div class="flex-1 overflow-y-auto divide-y divide-border">
          <div
            v-for="hora in HORAS_DIA"
            :key="hora"
            class="flex items-start min-h-14 p-2 hover:bg-muted/10 cursor-pointer"
            @click="abrirCrearEvento(`${formatearFechaIso(store.fechaSeleccionada)}T${pad(hora)}:00`)"
          >
            <span class="w-16 text-xs text-muted-foreground font-mono">{{ pad(hora) }}:00</span>
            <div class="flex-1 flex flex-wrap gap-2 pl-4">
              <div
                v-for="ev in store.eventos.filter(e => coincideHora(e.inicio, hora) && fechaLocalIsoDe(e.inicio) === formatearFechaIso(store.fechaSeleccionada))"
                :key="ev.id"
                :class="[
                  'px-3 py-1.5 rounded-md border text-xs font-medium hover:opacity-80 flex items-center gap-2',
                  getBadgeClass(ev.tipo, ev.esVirtual)
                ]"
                @click.stop="abrirEditarEvento(ev)"
              >
                <span>{{ ev.titulo }}</span>
                <span v-if="ev.recursos.length > 0" class="text-[10px] opacity-75 font-normal">
                  ({{ ev.recursos.map(r => r.nombre).join(', ') }})
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 4. RESOURCE DAY VIEW -->
      <div v-else-if="store.vistaActual === 'recursos'" class="flex flex-col h-full border border-border rounded-lg bg-surface-card overflow-hidden">
        <div class="border-b border-border bg-muted/30 p-3 flex items-center justify-between">
          <span class="text-sm font-semibold">Vista de Recursos: {{ tituloPeriodo }}</span>
          <div class="flex items-center gap-2">
            <span class="text-xs text-muted-foreground">
              {{ recursosActivos.length }} recursos activos
            </span>
            <button
              v-if="auth.hasPermission('calendario:gestionar_recursos')"
              type="button"
              class="px-2.5 py-1 text-xs border border-border rounded-md hover:bg-muted"
              @click="ejecutarSincronizacionEmpleados"
            >
              Sincronizar Empleados
            </button>
          </div>
        </div>

        <div class="flex-1 overflow-x-auto flex">
          <div
            v-for="recurso in recursosActivos"
            :key="recurso.id"
            class="flex-1 min-w-56 border-r border-border flex flex-col"
          >
            <!-- Column header -->
            <div class="p-2 border-b border-border bg-muted/20 text-center">
              <div class="text-xs font-bold truncate">{{ recurso.nombre }}</div>
              <div class="text-[10px] text-muted-foreground uppercase tracking-wider">
                {{ recurso.tipo }}
              </div>
            </div>

            <!-- Event list for resource -->
            <div class="flex-1 p-2 flex flex-col gap-2 overflow-y-auto">
              <div
                v-for="ev in eventosDeRecurso(recurso.id)"
                :key="ev.id"
                :class="[
                  'p-2 rounded-md border text-xs cursor-pointer hover:opacity-80 transition-opacity',
                  getBadgeClass(ev.tipo, ev.esVirtual)
                ]"
                @click="abrirEditarEvento(ev)"
              >
                <div class="font-semibold truncate">{{ ev.titulo }}</div>
                <div class="text-[10px] opacity-75 mt-0.5">
                  {{ formatearHoraLocal(ev.inicio) }} - {{ formatearHoraLocal(ev.fin) }}
                </div>
                <div v-if="ev.descripcion" class="text-[10px] line-clamp-2 opacity-85 mt-1">
                  {{ ev.descripcion }}
                </div>
              </div>
              <div
                v-if="eventosDeRecurso(recurso.id).length === 0"
                class="text-[11px] text-muted-foreground text-center py-8 italic"
              >
                Sin asignaciones hoy
              </div>
            </div>
          </div>
        </div>
      </div>
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
