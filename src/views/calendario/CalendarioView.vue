<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import {
  useCalendarioStore,
  type CalendarioEventoDto,
  type CrearEventoInput,
  type ActualizarEventoInput,
  type TipoEvento,
  type TipoRecurso,
} from '@/stores/useCalendarioStore'
import { useAuthStore } from '@/stores/useAuthStore'

const store = useCalendarioStore()
const auth = useAuthStore()

const modalEventoAbierto = ref(false)
const modalRecursosAbierto = ref(false)
const eventoEditando = ref<CalendarioEventoDto | null>(null)

// Form state for Event modal
const formTitulo = ref('')
const formDescripcion = ref('')
const formTipo = ref<TipoEvento>('Trabajo')
const formInicio = ref('')
const formFin = ref('')
const formTodoElDia = ref(false)
const formRecursosIds = ref<string[]>([])

// Form state for Resource modal
const formRecursoNombre = ref('')
const formRecursoTipo = ref<TipoRecurso>('Empleado')
const formRecursoGrupoId = ref<string | null>(null)
const editandoRecursoId = ref<string | null>(null)

const DIAS_SEMANA = ['Lun', 'Mar', 'Mié', 'Jue', 'Vie', 'Sáb', 'Dom']
const HORAS_DIA = Array.from({ length: 14 }, (_, i) => i + 7) // 07:00 to 20:00

// Date helpers
function pad(n: number) {
  return n.toString().padStart(2, '0')
}

function formatearFechaIso(d: Date): string {
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`
}

function formatearFechaHoraIso(d: Date): string {
  return `${formatearFechaIso(d)}T${pad(d.getHours())}:${pad(d.getMinutes())}:00.000Z`
}

// Current period range calculation based on selected view
const rangoActual = computed(() => {
  const base = new Date(store.fechaSeleccionada)
  const y = base.getFullYear()
  const m = base.getMonth()

  if (store.vistaActual === 'mes') {
    const primerDiaMes = new Date(y, m, 1)
    const ultimoDiaMes = new Date(y, m + 1, 0)
    // Expand to full grid (from Monday of first week to Sunday of last week)
    const diaSemana = (primerDiaMes.getDay() + 6) % 7
    const inicio = new Date(primerDiaMes)
    inicio.setDate(inicio.getDate() - diaSemana)

    const fin = new Date(ultimoDiaMes)
    const extraDias = (7 - ((ultimoDiaMes.getDay() + 6) % 7) - 1) % 7
    fin.setDate(fin.getDate() + extraDias)
    fin.setHours(23, 59, 59, 999)

    return {
      desde: formatearFechaHoraIso(inicio),
      hasta: formatearFechaHoraIso(fin),
      inicio,
      fin,
    }
  }

  if (store.vistaActual === 'semana') {
    const diaSemana = (base.getDay() + 6) % 7
    const inicio = new Date(base)
    inicio.setDate(inicio.getDate() - diaSemana)
    inicio.setHours(0, 0, 0, 0)

    const fin = new Date(inicio)
    fin.setDate(fin.getDate() + 6)
    fin.setHours(23, 59, 59, 999)

    return {
      desde: formatearFechaHoraIso(inicio),
      hasta: formatearFechaHoraIso(fin),
      inicio,
      fin,
    }
  }

  // 'dia' or 'recursos' (single day)
  const inicio = new Date(base)
  inicio.setHours(0, 0, 0, 0)
  const fin = new Date(base)
  fin.setHours(23, 59, 59, 999)

  return {
    desde: formatearFechaHoraIso(inicio),
    hasta: formatearFechaHoraIso(fin),
    inicio,
    fin,
  }
})

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
})

watch([() => store.fechaSeleccionada, () => store.vistaActual], () => {
  recargarDatos()
})

// Period Title
const tituloPeriodo = computed(() => {
  const d = store.fechaSeleccionada
  const meses = [
    'Enero', 'Febrero', 'Marzo', 'Abril', 'Mayo', 'Junio',
    'Julio', 'Agosto', 'Septiembre', 'Octubre', 'Noviembre', 'Diciembre',
  ]
  if (store.vistaActual === 'mes') {
    return `${meses[d.getMonth()]} ${d.getFullYear()}`
  }
  if (store.vistaActual === 'semana') {
    const { inicio, fin } = rangoActual.value
    const mInicio = meses[inicio.getMonth()] ?? ''
    const mFin = meses[fin.getMonth()] ?? ''
    return `${inicio.getDate()} ${mInicio.substring(0, 3)} - ${fin.getDate()} ${mFin.substring(0, 3)} ${fin.getFullYear()}`
  }
  return `${d.getDate()} de ${meses[d.getMonth()] ?? ''} de ${d.getFullYear()}`
})

// Navigation
function irAnterior() {
  const d = new Date(store.fechaSeleccionada)
  if (store.vistaActual === 'mes') {
    d.setMonth(d.getMonth() - 1)
  } else if (store.vistaActual === 'semana') {
    d.setDate(d.getDate() - 7)
  } else {
    d.setDate(d.getDate() - 1)
  }
  store.fechaSeleccionada = d
}

function irSiguiente() {
  const d = new Date(store.fechaSeleccionada)
  if (store.vistaActual === 'mes') {
    d.setMonth(d.getMonth() + 1)
  } else if (store.vistaActual === 'semana') {
    d.setDate(d.getDate() + 7)
  } else {
    d.setDate(d.getDate() + 1)
  }
  store.fechaSeleccionada = d
}

function irHoy() {
  store.fechaSeleccionada = new Date()
}

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
      const eInicio = e.inicio.substring(0, 10)
      const eFin = e.fin.substring(0, 10)
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
      const eInicio = e.inicio.substring(0, 10)
      const eFin = e.fin.substring(0, 10)
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
    const eInicio = e.inicio.substring(0, 10)
    const eFin = e.fin.substring(0, 10)
    const coincideDia = diaIso >= eInicio && diaIso <= eFin
    const tieneRecurso = e.recursos.some((r) => r.id === recursoId)
    return coincideDia && tieneRecurso
  })
}

// Modal actions
function abrirCrearEvento(fechaPredeterminada?: string) {
  eventoEditando.value = null
  formTitulo.value = ''
  formDescripcion.value = ''
  formTipo.value = 'Trabajo'

  const base = fechaPredeterminada ? new Date(fechaPredeterminada) : new Date(store.fechaSeleccionada)
  const anio = base.getFullYear()
  const mes = pad(base.getMonth() + 1)
  const dia = pad(base.getDate())

  formInicio.value = `${anio}-${mes}-${dia}T09:00`
  formFin.value = `${anio}-${mes}-${dia}T10:00`
  formTodoElDia.value = false
  formRecursosIds.value = []

  modalEventoAbierto.value = true
}

function abrirEditarEvento(ev: CalendarioEventoDto) {
  if (ev.esVirtual) {
    // Virtual items (holidays/jobs) cannot be edited directly from calendar
    return
  }
  eventoEditando.value = ev
  formTitulo.value = ev.titulo
  formDescripcion.value = ev.descripcion || ''
  formTipo.value = ev.tipo
  formInicio.value = ev.inicio.substring(0, 16)
  formFin.value = ev.fin.substring(0, 16)
  formTodoElDia.value = ev.todoElDia
  formRecursosIds.value = ev.recursos.map((r) => r.id)

  modalEventoAbierto.value = true
}

async function guardarEvento() {
  if (!formTitulo.value.trim()) return

  const inicioUtc = `${formInicio.value}:00.000Z`
  const finUtc = `${formFin.value}:00.000Z`

  try {
    if (eventoEditando.value) {
      const input: ActualizarEventoInput = {
        titulo: formTitulo.value.trim(),
        descripcion: formDescripcion.value.trim() || null,
        tipo: formTipo.value,
        inicio: inicioUtc,
        fin: finUtc,
        todoElDia: formTodoElDia.value,
        recursoIds: formRecursosIds.value,
        rowVersion: eventoEditando.value.rowVersion,
      }
      await store.actualizarEvento(eventoEditando.value.id, input)
    } else {
      const input: CrearEventoInput = {
        titulo: formTitulo.value.trim(),
        descripcion: formDescripcion.value.trim() || null,
        tipo: formTipo.value,
        inicio: inicioUtc,
        fin: finUtc,
        todoElDia: formTodoElDia.value,
        recursoIds: formRecursosIds.value,
      }
      await store.crearEvento(input)
    }
    modalEventoAbierto.value = false
  } catch (err: unknown) {
    alert(err instanceof Error ? err.message : 'Error al guardar el evento')
  }
}

async function borrarEvento() {
  if (!eventoEditando.value) return
  if (!confirm('¿Eliminar este evento del calendario?')) return

  try {
    await store.eliminarEvento(eventoEditando.value.id, eventoEditando.value.rowVersion)
    modalEventoAbierto.value = false
  } catch (err: unknown) {
    alert(err instanceof Error ? err.message : 'Error al eliminar el evento')
  }
}

// Resource management
function abrirModalRecursos() {
  editandoRecursoId.value = null
  formRecursoNombre.value = ''
  formRecursoTipo.value = 'Empleado'
  formRecursoGrupoId.value = null
  modalRecursosAbierto.value = true
}

async function guardarRecurso() {
  if (!formRecursoNombre.value.trim()) return

  try {
    if (editandoRecursoId.value) {
      const existente = store.recursos.find((r) => r.id === editandoRecursoId.value)
      if (existente) {
        await store.actualizarRecurso(existente.id, {
          nombre: formRecursoNombre.value.trim(),
          tipo: formRecursoTipo.value,
          grupoId: formRecursoGrupoId.value,
          activo: existente.activo,
          rowVersion: existente.rowVersion,
        })
      }
    } else {
      await store.crearRecurso({
        nombre: formRecursoNombre.value.trim(),
        tipo: formRecursoTipo.value,
        grupoId: formRecursoGrupoId.value,
      })
    }
    formRecursoNombre.value = ''
    editandoRecursoId.value = null
  } catch (err: unknown) {
    alert(err instanceof Error ? err.message : 'Error al guardar el recurso')
  }
}

async function ejecutarSincronizacionEmpleados() {
  try {
    await store.sincronizarEmpleados()
    alert('Empleados sincronizados como recursos correctamente.')
  } catch (err: unknown) {
    alert(err instanceof Error ? err.message : 'Error al sincronizar empleados')
  }
}

// Event Type badge helpers
function getBadgeClass(tipo: TipoEvento, esVirtual: boolean) {
  if (esVirtual) {
    return 'bg-warning/20 text-warning border-warning/30'
  }
  switch (tipo) {
    case 'Trabajo':
      return 'bg-primary/20 text-primary border-primary/30'
    case 'Reunion':
      return 'bg-info/20 text-info border-info/30'
    case 'Mantenimiento':
      return 'bg-warning/20 text-warning border-warning/30'
    case 'Entrega':
      return 'bg-success/20 text-success border-success/30'
    default:
      return 'bg-muted text-foreground border-border'
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
                v-for="ev in dia.eventos.filter(e => e.inicio.includes(`T${pad(hora)}:`))"
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
                v-for="ev in store.eventos.filter(e => e.inicio.includes(`T${pad(hora)}:`))"
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
                  {{ ev.inicio.substring(11, 16) }} - {{ ev.fin.substring(11, 16) }}
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
    <div
      v-if="modalEventoAbierto"
      class="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-xs p-4"
    >
      <div class="bg-surface-card border border-border rounded-xl shadow-lg w-full max-w-lg overflow-hidden">
        <div class="p-4 border-b border-border flex items-center justify-between">
          <h2 class="text-base font-bold">
            {{ eventoEditando ? 'Editar Evento' : 'Nuevo Evento' }}
          </h2>
          <button
            type="button"
            class="text-muted-foreground hover:text-foreground text-sm"
            @click="modalEventoAbierto = false"
          >
            ✕
          </button>
        </div>

        <form @submit.prevent="guardarEvento" class="p-4 flex flex-col gap-3 text-xs">
          <div>
            <label class="font-medium text-foreground block mb-1">Título *</label>
            <input
              v-model="formTitulo"
              type="text"
              required
              class="w-full px-3 py-2 bg-background border border-border rounded-md text-foreground focus:outline-hidden focus:ring-1 focus:ring-primary"
              placeholder="Ej. Instalación en planta matriz"
            />
          </div>

          <div class="grid grid-cols-2 gap-3">
            <div>
              <label class="font-medium text-foreground block mb-1">Tipo de Evento</label>
              <select
                v-model="formTipo"
                class="w-full px-3 py-2 bg-background border border-border rounded-md text-foreground"
              >
                <option value="Trabajo">Trabajo</option>
                <option value="Reunion">Reunión</option>
                <option value="Mantenimiento">Mantenimiento</option>
                <option value="Entrega">Entrega</option>
                <option value="Otro">Otro</option>
              </select>
            </div>
            <div class="flex items-center gap-2 pt-6">
              <input
                id="todoElDia"
                v-model="formTodoElDia"
                type="checkbox"
                class="rounded border-border text-primary"
              />
              <label for="todoElDia" class="font-medium text-foreground">Todo el día</label>
            </div>
          </div>

          <div class="grid grid-cols-2 gap-3">
            <div>
              <label class="font-medium text-foreground block mb-1">Inicio</label>
              <input
                v-model="formInicio"
                type="datetime-local"
                required
                class="w-full px-3 py-2 bg-background border border-border rounded-md text-foreground"
              />
            </div>
            <div>
              <label class="font-medium text-foreground block mb-1">Fin</label>
              <input
                v-model="formFin"
                type="datetime-local"
                required
                class="w-full px-3 py-2 bg-background border border-border rounded-md text-foreground"
              />
            </div>
          </div>

          <div>
            <label class="font-medium text-foreground block mb-1">Recursos Asignados</label>
            <div class="max-h-28 overflow-y-auto border border-border rounded-md p-2 flex flex-col gap-1.5 bg-background">
              <label
                v-for="rec in store.recursos.filter(r => r.activo)"
                :key="rec.id"
                class="flex items-center gap-2 cursor-pointer text-xs"
              >
                <input
                  type="checkbox"
                  :value="rec.id"
                  v-model="formRecursosIds"
                  class="rounded border-border text-primary"
                />
                <span>{{ rec.nombre }}</span>
                <span class="text-[10px] text-muted-foreground font-mono">({{ rec.tipo }})</span>
              </label>
            </div>
          </div>

          <div>
            <label class="font-medium text-foreground block mb-1">Descripción</label>
            <textarea
              v-model="formDescripcion"
              rows="2"
              class="w-full px-3 py-2 bg-background border border-border rounded-md text-foreground"
              placeholder="Detalles adicionales..."
            ></textarea>
          </div>

          <div class="flex items-center justify-between pt-3 border-t border-border mt-2">
            <div>
              <button
                v-if="eventoEditando && auth.hasPermission('calendario:editar_evento')"
                type="button"
                class="px-3 py-1.5 text-destructive hover:bg-destructive/10 rounded-md font-medium"
                @click="borrarEvento"
              >
                Eliminar
              </button>
            </div>
            <div class="flex items-center gap-2">
              <button
                type="button"
                class="px-3 py-1.5 border border-border rounded-md hover:bg-muted"
                @click="modalEventoAbierto = false"
              >
                Cancelar
              </button>
              <button
                type="submit"
                class="px-4 py-1.5 bg-primary text-primary-foreground font-medium rounded-md hover:opacity-90"
              >
                Guardar
              </button>
            </div>
          </div>
        </form>
      </div>
    </div>

    <!-- MODAL: Gestión de Recursos -->
    <div
      v-if="modalRecursosAbierto"
      class="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-xs p-4"
    >
      <div class="bg-surface-card border border-border rounded-xl shadow-lg w-full max-w-xl overflow-hidden">
        <div class="p-4 border-b border-border flex items-center justify-between">
          <h2 class="text-base font-bold">Gestión de Recursos</h2>
          <button
            type="button"
            class="text-muted-foreground hover:text-foreground text-sm"
            @click="modalRecursosAbierto = false"
          >
            ✕
          </button>
        </div>

        <div class="p-4 flex flex-col gap-4 text-xs">
          <!-- Form new resource -->
          <form @submit.prevent="guardarRecurso" class="p-3 border border-border rounded-lg bg-muted/20 flex flex-col gap-3">
            <div class="font-semibold">
              {{ editandoRecursoId ? 'Editar Recurso' : 'Nuevo Recurso' }}
            </div>
            <div class="grid grid-cols-3 gap-2">
              <div class="col-span-2">
                <input
                  v-model="formRecursoNombre"
                  type="text"
                  required
                  class="w-full px-3 py-1.5 bg-background border border-border rounded-md text-foreground"
                  placeholder="Nombre (ej. Camioneta 01, Grua)"
                />
              </div>
              <div>
                <select
                  v-model="formRecursoTipo"
                  class="w-full px-3 py-1.5 bg-background border border-border rounded-md text-foreground"
                >
                  <option value="Empleado">Empleado</option>
                  <option value="Vehiculo">Vehículo</option>
                  <option value="Herramienta">Herramienta</option>
                  <option value="Proyecto">Proyecto</option>
                </select>
              </div>
            </div>
            <div class="flex justify-end gap-2">
              <button
                type="submit"
                class="px-3 py-1 bg-primary text-primary-foreground rounded-md font-medium hover:opacity-90"
              >
                Guardar Recurso
              </button>
            </div>
          </form>

          <!-- List of active resources -->
          <div class="max-h-60 overflow-y-auto border border-border rounded-lg divide-y divide-border">
            <div
              v-for="rec in store.recursos"
              :key="rec.id"
              class="p-2.5 flex items-center justify-between hover:bg-muted/10"
            >
              <div>
                <span class="font-medium">{{ rec.nombre }}</span>
                <span class="text-muted-foreground text-[10px] ml-2">({{ rec.tipo }})</span>
              </div>
              <div class="flex items-center gap-2">
                <button
                  type="button"
                  class="text-primary hover:underline"
                  @click="
                    editandoRecursoId = rec.id;
                    formRecursoNombre = rec.nombre;
                    formRecursoTipo = rec.tipo;
                    formRecursoGrupoId = rec.grupoId;
                  "
                >
                  Editar
                </button>
                <button
                  type="button"
                  class="text-destructive hover:underline"
                  @click="store.eliminarRecurso(rec.id, rec.rowVersion)"
                >
                  Eliminar
                </button>
              </div>
            </div>
          </div>
        </div>

        <div class="p-3 border-t border-border flex justify-end">
          <button
            type="button"
            class="px-4 py-1.5 border border-border rounded-md hover:bg-muted text-xs font-medium"
            @click="modalRecursosAbierto = false"
          >
            Cerrar
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
