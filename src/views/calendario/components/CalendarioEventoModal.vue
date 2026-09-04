<script setup lang="ts">
import Checkbox from 'primevue/checkbox'
import Dialog from 'primevue/dialog'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import Textarea from 'primevue/textarea'
import { ref, watch } from 'vue'
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
} from '@/stores/useCalendarioStore'
import { useAuthStore } from '@/stores/useAuthStore'
import { useTrabajosStore } from '@/stores/useTrabajosStore'
import type { LookupItem } from '@/stores/useCatalogStore'
import { formatearLocalParaInput, pad } from '../composables/useCalendarioPeriodo'

const props = defineProps<{
  visible: boolean
  evento: CalendarioEventoDto | null
  fechaPredeterminada?: string
  opcionesProyectos: LookupItem[]
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
  (e: 'guardado'): void
}>()

const tipoEventoOptions = [
  { label: 'Trabajo', value: 'Trabajo' },
  { label: 'Reunión', value: 'Reunion' },
  { label: 'Mantenimiento', value: 'Mantenimiento' },
  { label: 'Entrega', value: 'Entrega' },
  { label: 'Otro', value: 'Otro' },
]

const { notify } = useApiError()
const { confirmDelete } = useConfirmDelete()
const store = useCalendarioStore()
const auth = useAuthStore()
const trabajosStore = useTrabajosStore()

const formTitulo = ref('')
const formDescripcion = ref('')
const formTipo = ref<TipoEvento>('Trabajo')
const formInicio = ref('')
const formFin = ref('')
const formTodoElDia = ref(false)
const formRecursosIds = ref<string[]>([])
const formProyectoId = ref<string | null>(null)
const formTrabajoId = ref<string | null>(null)
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

watch(
  () => props.visible,
  (val) => {
    if (!val) return

    if (props.evento) {
      const ev = props.evento
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
    } else {
      formTitulo.value = ''
      formDescripcion.value = ''
      formTipo.value = 'Trabajo'
      formProyectoId.value = null
      formTrabajoId.value = null
      opcionesTrabajos.value = []

      const base = props.fechaPredeterminada
        ? new Date(props.fechaPredeterminada)
        : new Date(store.fechaSeleccionada)
      const anio = base.getFullYear()
      const mes = pad(base.getMonth() + 1)
      const dia = pad(base.getDate())

      formInicio.value = `${anio}-${mes}-${dia}T09:00`
      formFin.value = `${anio}-${mes}-${dia}T10:00`
      formTodoElDia.value = false
      formRecursosIds.value = []
    }
  }
)

async function guardarEvento() {
  if (!formTitulo.value.trim()) return

  const inicioUtc = new Date(formInicio.value).toISOString()
  const finUtc = new Date(formFin.value).toISOString()

  try {
    if (props.evento) {
      const input: ActualizarEventoInput = {
        titulo: formTitulo.value.trim(),
        descripcion: formDescripcion.value.trim() || null,
        tipo: formTipo.value,
        inicio: inicioUtc,
        fin: finUtc,
        todoElDia: formTodoElDia.value,
        recursoIds: formRecursosIds.value,
        trabajoId: formTrabajoId.value || null,
        rowVersion: props.evento.rowVersion,
      }
      await store.actualizarEvento(props.evento.id, input)
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
    emit('update:visible', false)
    emit('guardado')
  } catch (err: unknown) {
    notify(err)
  }
}

async function borrarEvento() {
  if (!props.evento) return
  const ev = props.evento
  confirmDelete({
    entityKey: 'Menu.Calendario',
    label: ev.titulo,
    action: async () => {
      await store.eliminarEvento(ev.id, ev.rowVersion)
      emit('update:visible', false)
      emit('guardado')
    },
  })
}
</script>

<template>
  <Dialog
    :visible="visible"
    modal
    :header="evento ? 'Editar Evento' : 'Nuevo Evento'"
    class="w-full max-w-lg"
    @update:visible="emit('update:visible', $event)"
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
            v-if="evento && auth.hasPermission('calendario:editar_evento')"
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
            @click="emit('update:visible', false)"
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
</template>
