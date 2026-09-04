<script setup lang="ts">
import Column from 'primevue/column'
import DataTable from 'primevue/datatable'
import Dialog from 'primevue/dialog'
import InputText from 'primevue/inputtext'
import Textarea from 'primevue/textarea'
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import DateInput from '@/components/domain/DateInput.vue'
import DateText from '@/components/domain/DateText.vue'
import DecimalInput from '@/components/domain/DecimalInput.vue'
import FieldError from '@/components/domain/FieldError.vue'
import ListState from '@/components/domain/ListState.vue'
import MoneyInput from '@/components/domain/MoneyInput.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import HelpButton from '@/components/ui/HelpButton.vue'
import { Button } from '@/components/ui/button'
import { useApiError, type ApiError } from '@/composables/useApiError'
import { useConfirmDelete } from '@/composables/useConfirmDelete'
import { useShortcuts } from '@/composables/useShortcuts'
import {
  useOrdenesTrabajoStore,
  type OrdenTrabajoItemInput,
  type OrdenTrabajoListItem,
} from '@/stores/useOrdenesTrabajoStore'
import { useTrabajosStore } from '@/stores/useTrabajosStore'

/**
 * Work orders of one job: the itemised quote certificates are issued against.
 * See `docs/09-modulos-funcionales.md` §3.6.
 *
 * The editor is a full-width dialog rather than the usual side drawer because the sheet is a grid
 * of lines, and a 480-pixel panel cannot show a line and its subtotal at the same time.
 */

const route = useRoute()
const router = useRouter()
const { notify, fieldErrors } = useApiError()
const { confirmDelete } = useConfirmDelete()
const store = useOrdenesTrabajoStore()
const trabajos = useTrabajosStore()

const trabajoId = computed(() => String(route.params.trabajoId ?? ''))

const rows = ref<OrdenTrabajoListItem[]>([])
const loading = ref(false)
const firstLoad = ref(true)
const error = ref<ApiError | null>(null)
const trabajoDescripcion = ref('')

async function cargar(): Promise<void> {
  loading.value = true
  error.value = null
  try {
    rows.value = await store.fetchDeTrabajo(trabajoId.value)
  } catch (e) {
    error.value = notify(e)
  } finally {
    loading.value = false
    firstLoad.value = false
  }
}

watch(trabajoId, cargar)

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

const editorOpen = ref(false)
const saving = ref(false)
const errores = ref<Record<string, string>>({})
const editor = ref<Editor>(vacio())

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

/** Lines that already carry certification cannot be removed, so the button is not offered. */
const certificados = ref<Set<string>>(new Set())

function abrirNuevo(): void {
  editor.value = vacio()
  certificados.value = new Set()
  errores.value = {}
  editorOpen.value = true
}

async function abrirEdicion(id: string): Promise<void> {
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
    editorOpen.value = true
  } catch (e) {
    notify(e)
  }
}

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

/**
 * Live value of a line, for the editor only. The figure that is stored comes from the backend:
 * this one exists so the user sees the row add up while typing.
 */
function baseDe(item: OrdenTrabajoItemInput): string {
  return (Number(item.cantidad) * Number(item.precioUnitario)).toFixed(4)
}

const totalPresupuestado = computed(() =>
  editor.value.items.reduce((acc, i) => acc + Number(baseDe(i)), 0).toFixed(4),
)

async function guardar(): Promise<void> {
  if (saving.value) return
  saving.value = true
  errores.value = {}
  try {
    const dto = {
      trabajoId: trabajoId.value,
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
    editorOpen.value = false
    await cargar()
  } catch (e) {
    const api = notify(e)
    if (api.code === 'VALIDATION') errores.value = fieldErrors(api)
  } finally {
    saving.value = false
  }
}

function onDelete(row: OrdenTrabajoListItem): void {
  confirmDelete({
    entityKey: 'Entity.OrdenTrabajo',
    label: row.titulo,
    action: () => store.remove(row.id, row.rowVersion),
    onDone: () => cargar(),
  })
}

function abrirDetalle(row: OrdenTrabajoListItem): void {
  void router.push({ name: 'orden-detalle', params: { ordenId: row.id } })
}

useShortcuts({ 'ctrl+n': abrirNuevo })

onMounted(async () => {
  await cargar()
  try {
    const trabajo = await trabajos.fetchOne(trabajoId.value)
    trabajoDescripcion.value = trabajo.descripcion
  } catch (e) {
    notify(e)
  }
})
</script>

<template>
  <section class="flex h-full flex-col gap-4 p-6">
    <PageHeader :title="$t('Ordenes.Title')" :subtitle="trabajoDescripcion">
      <template #actions>
        <Button variant="outline" @click="router.back()">
          <AppIcon name="arrow-left" :size="16" />
          {{ $t('General.Back') }}
        </Button>
        <Button @click="abrirNuevo()">
          <AppIcon name="plus" :size="16" />
          {{ $t('General.New') }}
        </Button>
        <HelpButton topic-id="ordenes-overview" title="Ayuda sobre Órdenes de Trabajo" />
      </template>
    </PageHeader>

    <ListState
      :loading="loading"
      :first-load="firstLoad"
      :error="error"
      :is-empty="(rows?.length ?? 0) === 0"
      :is-filtered="false"
      empty-key="Ordenes.Empty"
      class="flex-1"
      @retry="cargar()"
    >
      <DataTable
        :value="rows"
        data-key="id"
        size="small"
        class="text-sm"
        @row-dblclick="abrirDetalle($event.data as OrdenTrabajoListItem)"
      >
        <Column field="fecha" :header="$t('Ordenes.Fecha')">
          <template #body="{ data }"><DateText :value="data.fecha" /></template>
        </Column>
        <Column field="titulo" :header="$t('Ordenes.Titulo')" />
        <Column field="itemsCount" :header="$t('Ordenes.Items')" />
        <Column field="totalPresupuestado" :header="$t('Ordenes.TotalPresupuestado')">
          <template #body="{ data }"><MoneyText :value="data.totalPresupuestado" /></template>
        </Column>
        <Column field="certificadosCount" :header="$t('Ordenes.Certificados')" />
        <Column :header="$t('General.Actions')" :style="{ width: '8rem' }">
          <template #body="{ data }">
            <div class="flex gap-1">
              <Button
                variant="ghost"
                size="sm"
                :title="$t('Ordenes.VerDetalle')"
                @click="abrirDetalle(data)"
              >
                <AppIcon name="eye" :size="14" />
              </Button>
              <Button variant="ghost" size="sm" @click="abrirEdicion(data.id)">
                <AppIcon name="pencil" :size="14" />
              </Button>
              <Button
                v-if="data.certificadosCount === 0"
                variant="ghost"
                size="sm"
                @click="onDelete(data)"
              >
                <AppIcon name="trash-2" :size="14" />
              </Button>
            </div>
          </template>
        </Column>
      </DataTable>
    </ListState>

    <Dialog
      v-model:visible="editorOpen"
      modal
      maximizable
      :header="editor.id ? $t('Ordenes.Editar') : $t('Ordenes.Nueva')"
      class="w-full max-w-5xl"
    >
      <div class="space-y-4">
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
        <Button variant="outline" :disabled="saving" @click="editorOpen = false">
          {{ $t('General.Cancel') }}
        </Button>
        <Button :disabled="saving" @click="guardar()">{{ $t('General.Save') }}</Button>
      </template>
    </Dialog>
  </section>
</template>
