<script setup lang="ts">
import Dialog from 'primevue/dialog'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import Textarea from 'primevue/textarea'
import { computed, ref, watch } from 'vue'
import DateInput from '@/components/domain/DateInput.vue'
import DecimalInput from '@/components/domain/DecimalInput.vue'
import FieldError from '@/components/domain/FieldError.vue'
import MoneyInput from '@/components/domain/MoneyInput.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import { useApiError } from '@/composables/useApiError'
import type { LookupItem } from '@/stores/useCatalogStore'
import { useProyectosStore } from '@/stores/useProyectosStore'
import { useTrabajosStore } from '@/stores/useTrabajosStore'
import {
  useOrdenesTrabajoStore,
  type OrdenTrabajoItemInput,
} from '@/stores/useOrdenesTrabajoStore'

interface Editor {
  id: string | null
  rowVersion: string
  titulo: string
  fecha: string
  observaciones: string | null
  ajusteUocraPorcentaje: string
  otrosDescuentos: string
  items: OrdenTrabajoItemInput[]
}

const props = withDefaults(
  defineProps<{
    visible: boolean
    trabajoId?: string
    ordenId: string | null
  }>(),
  {
    trabajoId: '',
  },
)

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
  (e: 'saved'): void
}>()

const { notify, fieldErrors } = useApiError()
const store = useOrdenesTrabajoStore()
const proyectosStore = useProyectosStore()
const trabajosStore = useTrabajosStore()

const opcionesProyecto = ref<LookupItem[]>([])
const opcionesTrabajo = ref<LookupItem[]>([])
const proyectoIdSeleccionado = ref<string | null>(null)
const trabajoIdSeleccionado = ref<string>('')
const trabajoOriginalId = ref<string | null>(null)

const saving = ref(false)
const errores = ref<Record<string, string>>({})
const certificados = ref<Set<string>>(new Set())

function hoy(): string {
  return new Date().toISOString().slice(0, 10)
}

function lineaVacia(): OrdenTrabajoItemInput {
  return {
    id: null,
    descripcion: '',
    unidad: 'u',
    cantidad: '0.0000',
    precioUnitario: '0.0000',
    porcentajeActual: '0.0000',
    ejecutado: false,
    nota: null,
  }
}

function vacio(): Editor {
  return {
    id: null,
    rowVersion: '',
    titulo: '',
    fecha: hoy(),
    observaciones: null,
    ajusteUocraPorcentaje: '0.0000',
    otrosDescuentos: '0.0000',
    items: [lineaVacia()],
  }
}

const editor = ref<Editor>(vacio())

async function cargarParaEdicion(id: string): Promise<void> {
  errores.value = {}
  try {
    const d = await store.fetchOne(id)
    editor.value = {
      id: d.id,
      rowVersion: d.audit.rowVersion,
      titulo: d.titulo,
      fecha: d.fecha,
      observaciones: d.observaciones,
      ajusteUocraPorcentaje: d.ajusteUocraPorcentaje,
      otrosDescuentos: d.otrosDescuentos,
      items: d.items.map((i) => ({
        id: i.id,
        descripcion: i.descripcion,
        unidad: i.unidad,
        cantidad: i.cantidad,
        precioUnitario: i.precioUnitario,
        porcentajeActual: i.porcentajeActual,
        ejecutado: i.ejecutado,
        nota: i.nota,
      })),
    }
    certificados.value = new Set(d.items.filter((i) => i.certificado).map((i) => i.id))
    trabajoOriginalId.value = d.trabajoId
  } catch (e) {
    notify(e)
  }
}

async function onProyectoChange(): Promise<void> {
  trabajoIdSeleccionado.value = ''
  if (!proyectoIdSeleccionado.value) {
    opcionesTrabajo.value = []
    return
  }
  try {
    opcionesTrabajo.value = await trabajosStore.lookup(proyectoIdSeleccionado.value)
    if (opcionesTrabajo.value.length === 1 && opcionesTrabajo.value[0]) {
      trabajoIdSeleccionado.value = opcionesTrabajo.value[0].id
    }
  } catch {
    opcionesTrabajo.value = []
  }
}

watch(
  () => props.visible,
  async (abierto) => {
    if (!abierto) return
    errores.value = {}
    proyectoIdSeleccionado.value = null
    trabajoIdSeleccionado.value = props.trabajoId || ''
    trabajoOriginalId.value = null
    if (!props.trabajoId && !props.ordenId) {
      try {
        opcionesProyecto.value = await proyectosStore.lookup(undefined, undefined, 200)
      } catch {
        opcionesProyecto.value = []
      }
    }
    if (props.ordenId) {
      await cargarParaEdicion(props.ordenId)
    } else {
      editor.value = vacio()
      certificados.value = new Set()
    }
  },
  { immediate: true },
)

function agregarLinea(): void {
  editor.value.items.push(lineaVacia())
}

function quitarLinea(index: number): void {
  editor.value.items.splice(index, 1)
  if (editor.value.items.length === 0) agregarLinea()
}

function mover(index: number, delta: number): void {
  const destino = index + delta
  if (destino < 0 || destino >= editor.value.items.length) return
  const items = editor.value.items
  const linea = items.splice(index, 1)[0]
  if (linea) items.splice(destino, 0, linea)
}

function baseDe(item: OrdenTrabajoItemInput): string {
  return (Number(item.cantidad) * Number(item.precioUnitario)).toFixed(4)
}

const totalPresupuestado = computed(() =>
  editor.value.items.reduce((acc, i) => acc + Number(baseDe(i)), 0).toFixed(4),
)

async function guardar(): Promise<void> {
  if (saving.value) return
  errores.value = {}
  const targetTrabajoId = props.trabajoId || trabajoOriginalId.value || trabajoIdSeleccionado.value
  if (!targetTrabajoId) {
    errores.value = { trabajoId: 'Debe seleccionar un proyecto y un trabajo para la orden' }
    return
  }
  saving.value = true
  try {
    const dto = {
      trabajoId: targetTrabajoId,
      titulo: editor.value.titulo,
      fecha: editor.value.fecha,
      observaciones: editor.value.observaciones,
      ajusteUocraPorcentaje: editor.value.ajusteUocraPorcentaje,
      otrosDescuentos: editor.value.otrosDescuentos,
      items: editor.value.items,
    }
    if (editor.value.id) {
      await store.update(editor.value.id, dto, editor.value.rowVersion)
    } else {
      await store.create(dto)
    }
    emit('update:visible', false)
    emit('saved')
  } catch (e) {
    const api = notify(e)
    if (api.code === 'VALIDATION') errores.value = fieldErrors(api)
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <Dialog
    :visible="visible"
    modal
    maximizable
    :header="editor.id ? $t('Ordenes.Editar') : $t('Ordenes.Nueva')"
    class="w-full max-w-5xl"
    @update:visible="emit('update:visible', $event)"
  >
    <div class="space-y-4">
      <!-- Selector de Proyecto y Trabajo cuando se crea desde la vista global -->
      <div
        v-if="!props.trabajoId && !editor.id"
        class="grid grid-cols-1 gap-3 rounded-md border border-border/80 bg-muted/20 p-3 md:grid-cols-2"
      >
        <label class="flex flex-col gap-1">
          <span class="text-sm font-medium">
            {{ $t('Proyectos.Title') || 'Proyecto / Obra' }} <span class="text-destructive">*</span>
          </span>
          <Select
            v-model="proyectoIdSeleccionado"
            :options="opcionesProyecto"
            option-label="label"
            option-value="id"
            filter
            :placeholder="$t('General.Select') || 'Seleccionar Proyecto'"
            @update:model-value="onProyectoChange()"
          />
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-sm font-medium">
            {{ $t('Trabajos.Title') || 'Trabajo' }} <span class="text-destructive">*</span>
          </span>
          <Select
            v-model="trabajoIdSeleccionado"
            :options="opcionesTrabajo"
            option-label="label"
            option-value="id"
            filter
            :disabled="!proyectoIdSeleccionado"
            :placeholder="!proyectoIdSeleccionado ? 'Primero seleccione un proyecto' : ($t('General.Select') || 'Seleccionar Trabajo')"
            :invalid="Boolean(errores.trabajoId)"
          />
          <FieldError id="orden-trabajo-error" :message="errores.trabajoId" />
        </label>
      </div>

      <div class="grid grid-cols-1 gap-3 md:grid-cols-3">
        <label class="flex flex-col gap-1 md:col-span-2">
          <span class="text-sm">{{ $t('Ordenes.Titulo') }}</span>
          <InputText v-model="editor.titulo" :invalid="Boolean(errores.titulo)" />
          <FieldError id="orden-titulo-error" :message="errores.titulo" />
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('Ordenes.Fecha') }}</span>
          <DateInput v-model="editor.fecha" :invalid="Boolean(errores.fecha)" />
          <FieldError id="orden-fecha-error" :message="errores.fecha" />
        </label>
      </div>

      <div class="grid grid-cols-1 gap-3 md:grid-cols-2">
        <label class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('Ordenes.AjusteUocra') }}</span>
          <DecimalInput
            v-model="editor.ajusteUocraPorcentaje"
            :min="0"
            :max="100"
            suffix=" %"
            :invalid="Boolean(errores.ajusteUocraPorcentaje)"
          />
          <FieldError id="orden-uocra-error" :message="errores.ajusteUocraPorcentaje" />
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('Ordenes.OtrosDescuentos') }}</span>
          <MoneyInput
            v-model="editor.otrosDescuentos"
            :min="0"
            :invalid="Boolean(errores.otrosDescuentos)"
          />
          <FieldError id="orden-descuentos-error" :message="errores.otrosDescuentos" />
        </label>
      </div>

      <div class="space-y-2">
        <div class="flex items-center justify-between">
          <h4 class="text-sm font-semibold">{{ $t('Ordenes.Items') }}</h4>
          <Button variant="outline" size="sm" @click="agregarLinea()">
            <AppIcon name="plus" :size="14" />
            {{ $t('General.Add') }}
          </Button>
        </div>
        <FieldError id="orden-items-error" :message="errores.items" />

        <div
          v-for="(item, index) in editor.items"
          :key="index"
          class="grid grid-cols-12 items-end gap-2 rounded-md border border-border p-2"
        >
          <label class="col-span-12 flex flex-col gap-1 md:col-span-4">
            <span class="text-xs text-muted-foreground">{{ $t('Ordenes.Descripcion') }}</span>
            <InputText
              v-model="item.descripcion"
              :invalid="Boolean(errores[`items[${index}].descripcion`])"
            />
            <FieldError
              :id="`orden-item-${index}-descripcion-error`"
              :message="errores[`items[${index}].descripcion`]"
            />
          </label>
          <label class="col-span-4 flex flex-col gap-1 md:col-span-1">
            <span class="text-xs text-muted-foreground">{{ $t('Ordenes.Unidad') }}</span>
            <InputText
              v-model="item.unidad"
              :invalid="Boolean(errores[`items[${index}].unidad`])"
            />
          </label>
          <label class="col-span-4 flex flex-col gap-1 md:col-span-2">
            <span class="text-xs text-muted-foreground">{{ $t('Ordenes.Cantidad') }}</span>
            <DecimalInput
              v-model="item.cantidad"
              :min="0"
              :invalid="Boolean(errores[`items[${index}].cantidad`])"
            />
          </label>
          <label class="col-span-4 flex flex-col gap-1 md:col-span-2">
            <span class="text-xs text-muted-foreground">{{ $t('Ordenes.PrecioUnitario') }}</span>
            <MoneyInput
              v-model="item.precioUnitario"
              :min="0"
              :invalid="Boolean(errores[`items[${index}].precioUnitario`])"
            />
          </label>
          <div class="col-span-8 flex flex-col gap-1 md:col-span-2">
            <span class="text-xs text-muted-foreground">{{ $t('Ordenes.Subtotal') }}</span>
            <span class="py-2 text-right text-sm">
              <MoneyText :value="baseDe(item)" />
            </span>
          </div>
          <div class="col-span-4 flex justify-end gap-1 md:col-span-1">
            <Button
              variant="ghost"
              size="sm"
              :title="$t('Ordenes.SubirLinea')"
              @click="mover(index, -1)"
            >
              <AppIcon name="chevron-up" :size="14" />
            </Button>
            <Button
              variant="ghost"
              size="sm"
              :title="$t('Ordenes.BajarLinea')"
              @click="mover(index, 1)"
            >
              <AppIcon name="chevron-down" :size="14" />
            </Button>
            <Button
              v-if="!item.id || !certificados.has(item.id)"
              variant="ghost"
              size="sm"
              :title="$t('General.Delete')"
              @click="quitarLinea(index)"
            >
              <AppIcon name="trash-2" :size="14" />
            </Button>
          </div>
        </div>
      </div>

      <label class="flex flex-col gap-1">
        <span class="text-sm">{{ $t('Ordenes.Observaciones') }}</span>
        <Textarea v-model="editor.observaciones" rows="2" auto-resize />
      </label>

      <div class="flex justify-end gap-2 border-t border-border pt-3 text-sm">
        <span class="text-muted-foreground">{{ $t('Ordenes.TotalPresupuestado') }}</span>
        <MoneyText :value="totalPresupuestado" />
      </div>
    </div>

    <template #footer>
      <Button variant="outline" :disabled="saving" @click="emit('update:visible', false)">
        {{ $t('General.Cancel') }}
      </Button>
      <Button :disabled="saving" @click="guardar()">{{ $t('General.Save') }}</Button>
    </template>
  </Dialog>
</template>
