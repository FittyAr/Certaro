<script setup lang="ts">
import Checkbox from 'primevue/checkbox'
import Dialog from 'primevue/dialog'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import Textarea from 'primevue/textarea'
import { ref, computed, onMounted, watch } from 'vue'
import { useToast } from 'primevue/usetoast'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import { useApiError } from '@/composables/useApiError'
import { useConfirmDelete } from '@/composables/useConfirmDelete'
import {
  useCalendarioStore,
  type CalendarioEventoDto,
  type CrearEventoInput,
  type ActualizarEventoInput,
  type TipoEvento,
  type TipoRecurso,
} from '@/stores/useCalendarioStore'
import { useAuthStore } from '@/stores/useAuthStore'
import { useProyectosStore } from '@/stores/useProyectosStore'
import { useTrabajosStore } from '@/stores/useTrabajosStore'
import type { LookupItem } from '@/stores/useCatalogStore'

const tipoEventoOptions = [
  { label: 'Trabajo', value: 'Trabajo' },
  { label: 'Reunión', value: 'Reunion' },
  { label: 'Mantenimiento', value: 'Mantenimiento' },
  { label: 'Entrega', value: 'Entrega' },
  { label: 'Otro', value: 'Otro' },
]

const tipoRecursoOptions = [
  { label: 'Empleado', value: 'Empleado' },
  { label: 'Vehículo', value: 'Vehiculo' },
  { label: 'Herramienta', value: 'Herramienta' },
  { label: 'Proyecto', value: 'Proyecto' },
]

const { notify } = useApiError()
const { confirmDelete } = useConfirmDelete()
const toast = useToast()

const store = useCalendarioStore()
const auth = useAuthStore()
const proyectosStore = useProyectosStore()
const trabajosStore = useTrabajosStore()

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
const formProyectoId = ref<string | null>(null)
const formTrabajoId = ref<string | null>(null)
const opcionesProyectos = ref<LookupItem[]>([])
const opcionesTrabajos = ref<LookupItem[]>([])

async function onProyectoChange(): Promise<void> {
  formTrabajoId.value = null
  if (!formProyectoId.value) {
    opcionesTrabajos.value = []
    return
  }
  try {
    opcionesTrabajos.value = await trabajosStore.lookup(formProyectoId.value)
  } catch {
    opcionesTrabajos.value = []
  }
}

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

function fechaLocalIsoDe(isoUtc: string): string {
  return formatearFechaIso(new Date(isoUtc))
}

function formatearHoraLocal(isoUtc: string): string {
  const d = new Date(isoUtc)
  return `${pad(d.getHours())}:${pad(d.getMinutes())}`
}

function formatearLocalParaInput(isoUtc: string): string {
  const d = new Date(isoUtc)
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`
}

function coincideHora(isoUtc: string, hora: number): boolean {
  const d = new Date(isoUtc)
  return d.getHours() === hora
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
      desde: inicio.toISOString(),
      hasta: fin.toISOString(),
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
      desde: inicio.toISOString(),
      hasta: fin.toISOString(),
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
    desde: inicio.toISOString(),
    hasta: fin.toISOString(),
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
  formTitulo.value = ''
  formDescripcion.value = ''
  formTipo.value = 'Trabajo'
  formProyectoId.value = null
  formTrabajoId.value = null
  opcionesTrabajos.value = []

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
  formInicio.value = formatearLocalParaInput(ev.inicio)
  formFin.value = formatearLocalParaInput(ev.fin)
  formTodoElDia.value = ev.todoElDia
  formRecursosIds.value = ev.recursos.map((r) => r.id)
  formProyectoId.value = null
  formTrabajoId.value = ev.trabajoId ?? null
  opcionesTrabajos.value = []

  if (ev.trabajoId) {
    trabajosStore
      .fetchOne(ev.trabajoId)
      .then(async (t) => {
        formProyectoId.value = t.proyectoId
        opcionesTrabajos.value = await trabajosStore.lookup(t.proyectoId)
      })
      .catch(() => {})
  }

  modalEventoAbierto.value = true
}

async function guardarEvento() {
  if (!formTitulo.value.trim()) return

  const inicioUtc = new Date(formInicio.value).toISOString()
  const finUtc = new Date(formFin.value).toISOString()

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
        trabajoId: formTrabajoId.value || null,
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
        trabajoId: formTrabajoId.value || null,
      }
      await store.crearEvento(input)
    }
    modalEventoAbierto.value = false
  } catch (err: unknown) {
    notify(err)
  }
}

async function borrarEvento() {
  if (!eventoEditando.value) return
  const ev = eventoEditando.value
  confirmDelete({
    entityKey: 'Menu.Calendario',
    label: ev.titulo,
    action: async () => {
      await store.eliminarEvento(ev.id, ev.rowVersion)
      modalEventoAbierto.value = false
    },
  })
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
    notify(err)
  }
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
    <Dialog
      v-model:visible="modalEventoAbierto"
      modal
      :header="eventoEditando ? 'Editar Evento' : 'Nuevo Evento'"
      class="w-full max-w-lg"
    >
      <form class="flex flex-col gap-3" @submit.prevent="guardarEvento">
        <label class="flex flex-col gap-1">
          <span class="text-xs font-medium text-foreground">Título *</span>
          <InputText
            v-model="formTitulo"
            required
            placeholder="Ej. Instalación en planta matriz"
          />
        </label>

        <div class="grid grid-cols-2 gap-3">
          <label class="flex flex-col gap-1">
            <span class="text-xs font-medium text-foreground">Tipo de Evento</span>
            <Select
              v-model="formTipo"
              :options="tipoEventoOptions"
              option-label="label"
              option-value="value"
            />
          </label>
          <div class="flex items-center gap-2 pt-5">
            <Checkbox id="todoElDia" v-model="formTodoElDia" :binary="true" />
            <label for="todoElDia" class="text-xs font-medium text-foreground cursor-pointer">
              Todo el día
            </label>
          </div>
        </div>

        <div class="grid grid-cols-2 gap-3 rounded-md border border-border/70 bg-muted/20 p-2.5">
          <label class="flex flex-col gap-1">
            <span class="text-xs text-muted-foreground">Proyecto / Obra (opcional)</span>
            <Select
              v-model="formProyectoId"
              :options="opcionesProyectos"
              option-label="label"
              option-value="id"
              filter
              show-clear
              placeholder="Ninguno"
              @change="onProyectoChange"
            />
          </label>
          <label class="flex flex-col gap-1">
            <span class="text-xs text-muted-foreground">Trabajo / Frente (opcional)</span>
            <Select
              v-model="formTrabajoId"
              :options="opcionesTrabajos"
              option-label="label"
              option-value="id"
              filter
              show-clear
              placeholder="Ninguno"
              :disabled="!formProyectoId && opcionesTrabajos.length === 0"
            />
          </label>
        </div>

        <div class="grid grid-cols-2 gap-3">
          <label class="flex flex-col gap-1">
            <span class="text-xs font-medium text-foreground">Inicio</span>
            <InputText
              v-model="formInicio"
              type="datetime-local"
              required
            />
          </label>
          <label class="flex flex-col gap-1">
            <span class="text-xs font-medium text-foreground">Fin</span>
            <InputText
              v-model="formFin"
              type="datetime-local"
              required
            />
          </label>
        </div>

        <div>
          <span class="text-xs font-medium text-foreground block mb-1">Recursos Asignados</span>
          <div class="max-h-28 overflow-y-auto border border-border rounded-md p-2 flex flex-col gap-1.5 bg-background">
            <label
              v-for="rec in store.recursos.filter((r) => r.activo)"
              :key="rec.id"
              class="flex items-center gap-2 cursor-pointer text-xs"
            >
              <Checkbox
                v-model="formRecursosIds"
                :value="rec.id"
              />
              <span>{{ rec.nombre }}</span>
              <span class="text-[10px] text-muted-foreground font-mono">({{ rec.tipo }})</span>
            </label>
          </div>
        </div>

        <label class="flex flex-col gap-1">
          <span class="text-xs font-medium text-foreground">Descripción</span>
          <Textarea
            v-model="formDescripcion"
            rows="2"
            auto-resize
            placeholder="Detalles adicionales..."
          />
        </label>

        <div class="flex items-center justify-between pt-3 border-t border-border mt-2">
          <div>
            <Button
              v-if="eventoEditando && auth.hasPermission('calendario:editar_evento')"
              type="button"
              variant="destructive"
              size="sm"
              @click="borrarEvento"
            >
              <AppIcon name="trash-2" :size="14" />
              Eliminar
            </Button>
          </div>
          <div class="flex items-center gap-2">
            <Button
              type="button"
              variant="outline"
              size="sm"
              @click="modalEventoAbierto = false"
            >
              {{ $t('General.Cancel') }}
            </Button>
            <Button
              type="submit"
              size="sm"
            >
              {{ $t('General.Save') }}
            </Button>
          </div>
        </div>
      </form>
    </Dialog>

    <!-- MODAL: Gestión de Recursos -->
    <Dialog
      v-model:visible="modalRecursosAbierto"
      modal
      header="Gestión de Recursos"
      class="w-full max-w-xl"
    >
      <div class="flex flex-col gap-4">
        <!-- Form new resource -->
        <form class="p-3 border border-border rounded-lg bg-muted/20 flex flex-col gap-3" @submit.prevent="guardarRecurso">
          <div class="font-semibold text-xs">
            {{ editandoRecursoId ? 'Editar Recurso' : 'Nuevo Recurso' }}
          </div>
          <div class="grid grid-cols-3 gap-2">
            <div class="col-span-2">
              <InputText
                v-model="formRecursoNombre"
                required
                placeholder="Nombre (ej. Camioneta 01, Cuadrilla A)"
                class="w-full"
              />
            </div>
            <div>
              <Select
                v-model="formRecursoTipo"
                :options="tipoRecursoOptions"
                option-label="label"
                option-value="value"
                class="w-full"
              />
            </div>
          </div>
          <div class="flex justify-end gap-2">
            <Button
              v-if="editandoRecursoId"
              type="button"
              variant="ghost"
              size="sm"
              @click="editandoRecursoId = null; formRecursoNombre = ''"
            >
              {{ $t('General.Cancel') }}
            </Button>
            <Button type="submit" size="sm">
              {{ editandoRecursoId ? 'Actualizar' : 'Guardar Recurso' }}
            </Button>
          </div>
        </form>

        <!-- List of active resources -->
        <div class="max-h-60 overflow-y-auto border border-border rounded-lg divide-y divide-border">
          <div
            v-for="rec in store.recursos"
            :key="rec.id"
            class="p-2.5 flex items-center justify-between hover:bg-muted/10 text-xs"
          >
            <div>
              <span class="font-medium">{{ rec.nombre }}</span>
              <span class="text-muted-foreground text-[10px] ml-2">({{ rec.tipo }})</span>
            </div>
            <div class="flex items-center gap-2">
              <Button
                variant="ghost"
                size="sm"
                @click="
                  editandoRecursoId = rec.id;
                  formRecursoNombre = rec.nombre;
                  formRecursoTipo = rec.tipo;
                  formRecursoGrupoId = rec.grupoId;
                "
              >
                <AppIcon name="pencil" :size="14" />
              </Button>
              <Button
                variant="ghost"
                size="sm"
                @click="store.eliminarRecurso(rec.id, rec.rowVersion)"
              >
                <AppIcon name="trash-2" :size="14" />
              </Button>
            </div>
          </div>
        </div>
      </div>
      <template #footer>
        <Button
          type="button"
          variant="outline"
          size="sm"
          @click="modalRecursosAbierto = false"
        >
          {{ $t('General.Close') }}
        </Button>
      </template>
    </Dialog>
  </div>
</template>
