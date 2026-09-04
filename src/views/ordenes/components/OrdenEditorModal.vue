<script setup lang="ts">
import Dialog from 'primevue/dialog'
import InputText from 'primevue/inputtext'
import Textarea from 'primevue/textarea'
import { computed, ref, watch } from 'vue'
import DecimalInput from '@/components/domain/DecimalInput.vue'
import FieldError from '@/components/domain/FieldError.vue'
import MoneyInput from '@/components/domain/MoneyInput.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import { useApiError } from '@/composables/useApiError'
import {
  useOrdenesTrabajoStore,
  type OrdenTrabajoDetalle,
  type OrdenTrabajoItemInput,
} from '@/stores/useOrdenesTrabajoStore'

interface EditorOrden {
  id: string
  rowVersion: string
  titulo: string
  fecha: string
  observaciones: string | null
  ajusteUocraPorcentaje: string
  otrosDescuentos: string
  items: OrdenTrabajoItemInput[]
}

const props = defineProps<{
  visible: boolean
  orden: OrdenTrabajoDetalle | null
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
  (e: 'saved'): void
}>()

const { notify, fieldErrors } = useApiError()
const store = useOrdenesTrabajoStore()

const savingEditor = ref(false)
const erroresEditor = ref<Record<string, string>>({})
const itemsCertificados = ref<Set<string>>(new Set())

const editor = ref<EditorOrden>({
  id: '',
  rowVersion: '',
  titulo: '',
  fecha: '',
  observaciones: null,
  ajusteUocraPorcentaje: '0.0000',
  otrosDescuentos: '0.0000',
  items: [],
})

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

watch(
  () => props.visible,
  (val) => {
    if (!val || !props.orden) return
    erroresEditor.value = {}
    editor.value = {
      id: props.orden.id,
      rowVersion: props.orden.audit.rowVersion,
      titulo: props.orden.titulo,
      fecha: props.orden.fecha,
      observaciones: props.orden.observaciones,
      ajusteUocraPorcentaje: props.orden.ajusteUocraPorcentaje,
      otrosDescuentos: props.orden.otrosDescuentos,
      items: props.orden.items.map((i) => ({
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
    itemsCertificados.value = new Set(
      props.orden.items.filter((i) => Number(i.porcentajeAcumulado) > 0).map((i) => i.id),
    )
  }
)

function agregarLinea(): void {
  editor.value.items.push(lineaVacia())
}

function quitarLinea(index: number): void {
  editor.value.items.splice(index, 1)
  if (editor.value.items.length === 0) agregarLinea()
}

function moverLinea(index: number, delta: number): void {
  const destino = index + delta
  if (destino < 0 || destino >= editor.value.items.length) return
  const items = editor.value.items
  const linea = items.splice(index, 1)[0]
  if (linea) items.splice(destino, 0, linea)
}

function baseItemDe(item: OrdenTrabajoItemInput): string {
  return (Number(item.cantidad) * Number(item.precioUnitario)).toFixed(4)
}

const totalPresupuestadoEditor = computed(() =>
  editor.value.items.reduce((acc, i) => acc + Number(baseItemDe(i)), 0).toFixed(4),
)

async function guardarEdicion(): Promise<void> {
  if (savingEditor.value || !props.orden) return
  savingEditor.value = true
  erroresEditor.value = {}
  try {
    const dto = {
      trabajoId: props.orden.trabajoId,
      titulo: editor.value.titulo,
      fecha: editor.value.fecha,
      observaciones: editor.value.observaciones,
      ajusteUocraPorcentaje: editor.value.ajusteUocraPorcentaje,
      otrosDescuentos: editor.value.otrosDescuentos,
      items: editor.value.items,
    }
    await store.update(editor.value.id, dto, editor.value.rowVersion)
    emit('update:visible', false)
    emit('saved')
  } catch (e) {
    const api = notify(e)
    if (api.code === 'VALIDATION') erroresEditor.value = fieldErrors(api)
  } finally {
    savingEditor.value = false
  }
}
</script>

<template>
  <Dialog
    :visible="visible"
    modal
    maximizable
    :header="$t('General.Edit') + ' - ' + (editor.titulo || $t('Ordenes.Title'))"
    :style="{ width: '80vw', maxWidth: '1100px' }"
    @update:visible="emit('update:visible', $event)"
  >
    <div class="space-y-4 pt-2">
      <div class="grid grid-cols-1 gap-3 md:grid-cols-2">
        <label class="flex flex-col gap-1">
          <span class="text-sm font-medium">{{ $t('Ordenes.Titulo') }}</span>
          <InputText v-model="editor.titulo" :invalid="Boolean(erroresEditor.titulo)" />
          <FieldError id="orden-edit-titulo-error" :message="erroresEditor.titulo" />
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-sm font-medium">{{ $t('Ordenes.Fecha') }}</span>
          <InputText v-model="editor.fecha" type="date" :invalid="Boolean(erroresEditor.fecha)" />
          <FieldError id="orden-edit-fecha-error" :message="erroresEditor.fecha" />
        </label>
      </div>

      <div class="grid grid-cols-1 gap-3 md:grid-cols-2">
        <label class="flex flex-col gap-1">
          <span class="text-sm font-medium">{{ $t('Ordenes.AjusteUocraPorcentaje') }}</span>
          <DecimalInput
            v-model="editor.ajusteUocraPorcentaje"
            :min="0"
            :max="100"
            suffix=" %"
            :invalid="Boolean(erroresEditor.ajusteUocraPorcentaje)"
          />
          <FieldError id="orden-edit-uocra-error" :message="erroresEditor.ajusteUocraPorcentaje" />
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-sm font-medium">{{ $t('Ordenes.OtrosDescuentos') }}</span>
          <MoneyInput
            v-model="editor.otrosDescuentos"
            :min="0"
            :invalid="Boolean(erroresEditor.otrosDescuentos)"
          />
          <FieldError id="orden-edit-descuentos-error" :message="erroresEditor.otrosDescuentos" />
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
        <FieldError id="orden-edit-items-error" :message="erroresEditor.items" />

        <div
          v-for="(item, index) in editor.items"
          :key="index"
          class="grid grid-cols-12 items-end gap-2 rounded-md border border-border p-2"
        >
          <label class="col-span-12 flex flex-col gap-1 md:col-span-4">
            <span class="text-xs text-muted-foreground">{{ $t('Ordenes.Descripcion') }}</span>
            <InputText
              v-model="item.descripcion"
              :invalid="Boolean(erroresEditor[`items[${index}].descripcion`])"
            />
            <FieldError
              :id="`orden-edit-item-${index}-descripcion-error`"
              :message="erroresEditor[`items[${index}].descripcion`]"
            />
          </label>
          <label class="col-span-4 flex flex-col gap-1 md:col-span-1">
            <span class="text-xs text-muted-foreground">{{ $t('Ordenes.Unidad') }}</span>
            <InputText
              v-model="item.unidad"
              :invalid="Boolean(erroresEditor[`items[${index}].unidad`])"
            />
          </label>
          <label class="col-span-4 flex flex-col gap-1 md:col-span-2">
            <span class="text-xs text-muted-foreground">{{ $t('Ordenes.Cantidad') }}</span>
            <DecimalInput
              v-model="item.cantidad"
              :min="0"
              :invalid="Boolean(erroresEditor[`items[${index}].cantidad`])"
            />
          </label>
          <label class="col-span-4 flex flex-col gap-1 md:col-span-2">
            <span class="text-xs text-muted-foreground">{{ $t('Ordenes.PrecioUnitario') }}</span>
            <MoneyInput
              v-model="item.precioUnitario"
              :min="0"
              :invalid="Boolean(erroresEditor[`items[${index}].precioUnitario`])"
            />
          </label>
          <div class="col-span-8 flex flex-col gap-1 md:col-span-2">
            <span class="text-xs text-muted-foreground">{{ $t('Ordenes.Subtotal') }}</span>
            <span class="py-2 text-right text-sm">
              <MoneyText :value="baseItemDe(item)" />
            </span>
          </div>
          <div class="col-span-4 flex justify-end gap-1 md:col-span-1">
            <Button
              variant="ghost"
              size="sm"
              :title="$t('Ordenes.SubirLinea')"
              @click="moverLinea(index, -1)"
            >
              <AppIcon name="chevron-up" :size="14" />
            </Button>
            <Button
              variant="ghost"
              size="sm"
              :title="$t('Ordenes.BajarLinea')"
              @click="moverLinea(index, 1)"
            >
              <AppIcon name="chevron-down" :size="14" />
            </Button>
            <Button
              v-if="!item.id || !itemsCertificados.has(item.id)"
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
        <MoneyText :value="totalPresupuestadoEditor" />
      </div>
    </div>

    <template #footer>
      <Button variant="outline" :disabled="savingEditor" @click="emit('update:visible', false)">
        {{ $t('General.Cancel') }}
      </Button>
      <Button :disabled="savingEditor" @click="guardarEdicion()">
        {{ $t('General.Save') }}
      </Button>
    </template>
  </Dialog>
</template>
