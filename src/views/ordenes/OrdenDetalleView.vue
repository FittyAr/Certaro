<script setup lang="ts">
import Column from 'primevue/column'
import DataTable from 'primevue/datatable'
import Dialog from 'primevue/dialog'
import InputText from 'primevue/inputtext'
import Textarea from 'primevue/textarea'
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import DateInput from '@/components/domain/DateInput.vue'
import DateText from '@/components/domain/DateText.vue'
import DecimalInput from '@/components/domain/DecimalInput.vue'
import FieldError from '@/components/domain/FieldError.vue'
import ListState from '@/components/domain/ListState.vue'
import MoneyInput from '@/components/domain/MoneyInput.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
import PercentBar from '@/components/domain/PercentBar.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import HelpButton from '@/components/ui/HelpButton.vue'
import { Button } from '@/components/ui/button'
import { useApiError, type ApiError } from '@/composables/useApiError'
import { useExport } from '@/composables/useExport'
import { useReportesStore } from '@/stores/useReportesStore'
import {
  useCertificadosStore,
  type CertificadoBorradorItem,
  type CertificadoListItem,
} from '@/stores/useCertificadosStore'
import {
  useOrdenesTrabajoStore,
  type OrdenTrabajoDetalle,
  type OrdenTrabajoItemInput,
} from '@/stores/useOrdenesTrabajoStore'

/**
 * One work order: its sheet, its totals and the certificates issued against it.
 * See `docs/09-modulos-funcionales.md` §3.6 and §3.7.
 *
 * The issuing form is prefilled by the backend (`certificados_preparar`) rather than by reading the
 * items here: the ceiling of each line comes from the certificates already issued, and the check
 * that guards the write reads the same source.
 */

const route = useRoute()
const router = useRouter()
const { notify, fieldErrors } = useApiError()
const store = useOrdenesTrabajoStore()
const certificados = useCertificadosStore()
const reportes = useReportesStore()
const { exportar } = useExport()

const ordenId = computed(() => String(route.params.ordenId ?? ''))

const orden = ref<OrdenTrabajoDetalle | null>(null)
const certificadosEmitidos = ref<CertificadoListItem[]>([])
const loading = ref(false)
const firstLoad = ref(true)
const error = ref<ApiError | null>(null)

async function cargar(): Promise<void> {
  loading.value = true
  error.value = null
  try {
    const [ord, certs] = await Promise.all([
      store.fetchOne(ordenId.value),
      certificados.fetchPaged({
        page: 1,
        pageSize: 50,
        filtro: { ordenTrabajoId: ordenId.value },
        sortBy: 'numero',
        sortDir: 'Desc',
      }),
    ])
    orden.value = ord
    certificadosEmitidos.value = certs.items
  } catch (e) {
    error.value = notify(e)
  } finally {
    loading.value = false
    firstLoad.value = false
  }
}

function exportarPdfCertificado(cert: CertificadoListItem): void {
  void exportar({
    reporte: 'certificado',
    formato: 'Pdf',
    detalle: `${cert.proyectoNombre} - #${cert.numero}`,
    run: (destino) => reportes.exportCertificado(cert.id, destino),
  })
}

// ------------------------------------------------------------------- Order Editor
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

const editorOpen = ref(false)
const savingEditor = ref(false)
const erroresEditor = ref<Record<string, string>>({})
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
const itemsCertificados = ref<Set<string>>(new Set())

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

function abrirEdicion(): void {
  if (!orden.value) return
  erroresEditor.value = {}
  editor.value = {
    id: orden.value.id,
    rowVersion: orden.value.audit.rowVersion,
    titulo: orden.value.titulo,
    fecha: orden.value.fecha,
    observaciones: orden.value.observaciones,
    ajusteUocraPorcentaje: orden.value.ajusteUocraPorcentaje,
    otrosDescuentos: orden.value.otrosDescuentos,
    items: orden.value.items.map((i) => ({
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
    orden.value.items.filter((i) => Number(i.porcentajeAcumulado) > 0).map((i) => i.id),
  )
  editorOpen.value = true
}

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
  if (savingEditor.value || !orden.value) return
  savingEditor.value = true
  erroresEditor.value = {}
  try {
    const dto = {
      trabajoId: orden.value.trabajoId,
      titulo: editor.value.titulo,
      fecha: editor.value.fecha,
      observaciones: editor.value.observaciones,
      ajusteUocraPorcentaje: editor.value.ajusteUocraPorcentaje,
      otrosDescuentos: editor.value.otrosDescuentos,
      items: editor.value.items,
    }
    await store.update(editor.value.id, dto, editor.value.rowVersion)
    editorOpen.value = false
    await cargar()
  } catch (e) {
    const api = notify(e)
    if (api.code === 'VALIDATION') erroresEditor.value = fieldErrors(api)
  } finally {
    savingEditor.value = false
  }
}

const emisionOpen = ref(false)
const emitiendo = ref(false)
const erroresEmision = ref<Record<string, string>>({})
const fechaEmision = ref(new Date().toISOString().slice(0, 10))
const observacionesEmision = ref<string | null>(null)
/** Percentage typed for each line of the draft, keyed by the order item. */
const avances = ref<Record<string, string>>({})

const borrador = computed(() => certificados.borrador)

async function abrirEmision(): Promise<void> {
  erroresEmision.value = {}
  fechaEmision.value = new Date().toISOString().slice(0, 10)
  observacionesEmision.value = null
  try {
    const draft = await certificados.preparar(ordenId.value)
    avances.value = Object.fromEntries(
      draft.items.map((i) => [i.ordenTrabajoItemId, i.porcentajeActual]),
    )
    emisionOpen.value = true
  } catch (e) {
    notify(e)
  }
}

function acumuladoDe(item: CertificadoBorradorItem): string {
  const pedido = Number(avances.value[item.ordenTrabajoItemId] ?? '0')
  return (Number(item.porcentajeAcumuladoAnterior) + pedido).toFixed(4)
}

/** Over 100 is flagged before the save, which is what the legacy system never did. */
function excede(item: CertificadoBorradorItem): boolean {
  return Number(acumuladoDe(item)) > 100
}

function subtotalDe(item: CertificadoBorradorItem): string {
  const pedido = Number(avances.value[item.ordenTrabajoItemId] ?? '0')
  return ((Number(item.base) * pedido) / 100).toFixed(4)
}

const totalCertificadoAEmitir = computed(() => {
  const draft = borrador.value
  if (!draft) return '0.0000'
  return draft.items.reduce((acc, i) => acc + Number(subtotalDe(i)), 0).toFixed(4)
})

const ajusteUocraAEmitir = computed(() => {
  const draft = borrador.value
  if (!draft) return '0.0000'
  const sub = Number(totalCertificadoAEmitir.value)
  const pct = Number(draft.ajusteUocraPorcentaje)
  return ((sub * pct) / 100).toFixed(4)
})

const otrosDescuentosAEmitir = computed(() => {
  const draft = borrador.value
  if (!draft) return '0.0000'
  const sub = Number(totalCertificadoAEmitir.value)
  const totalOrden = draft.items.reduce((acc, i) => acc + Number(i.base), 0)
  const restante = Number(draft.otrosDescuentos)
  if (totalOrden <= 0 || restante <= 0) return '0.0000'
  const prop = (sub / totalOrden) * restante
  return Math.min(prop, restante).toFixed(4)
})

const totalNetoAEmitir = computed(() => {
  const bruto = Number(totalCertificadoAEmitir.value)
  const uocra = Number(ajusteUocraAEmitir.value)
  const otros = Number(otrosDescuentosAEmitir.value)
  return Math.max(0, bruto + uocra - otros).toFixed(4)
})

const hayExcedidos = computed(() => borrador.value?.items.some(excede) ?? false)
const hayAvance = computed(() => Object.values(avances.value).some((v) => Number(v) > 0))

async function emitir(): Promise<void> {
  if (emitiendo.value || !borrador.value) return
  emitiendo.value = true
  erroresEmision.value = {}
  try {
    const emitido = await certificados.create({
      ordenTrabajoId: ordenId.value,
      fecha: fechaEmision.value,
      observaciones: observacionesEmision.value,
      items: borrador.value.items.map((i) => ({
        ordenTrabajoItemId: i.ordenTrabajoItemId,
        porcentajeActual: avances.value[i.ordenTrabajoItemId] ?? '0.0000',
      })),
    })
    emisionOpen.value = false
    await router.push({ name: 'certificado-detalle', params: { certificadoId: emitido.id } })
  } catch (e) {
    const api = notify(e)
    if (api.code === 'VALIDATION') erroresEmision.value = fieldErrors(api)
  } finally {
    emitiendo.value = false
  }
}

onMounted(cargar)
</script>

<template>
  <section class="flex h-full flex-col gap-4 p-6">
    <PageHeader :title="orden?.titulo ?? $t('Ordenes.Title')" :subtitle="orden?.trabajoDescripcion">
      <template #actions>
        <Button variant="outline" @click="router.back()">
          <AppIcon name="arrow-left" :size="16" />
          {{ $t('General.Back') }}
        </Button>
        <Button v-if="orden" variant="outline" @click="abrirEdicion()">
          <AppIcon name="pencil" :size="16" />
          {{ $t('General.Edit') }}
        </Button>
        <Button :disabled="!orden" @click="abrirEmision()">
          <AppIcon name="file-badge" :size="16" />
          {{ $t('Certificados.Emitir') }}
        </Button>
        <HelpButton topic-id="ordenes-detalle" title="Ayuda sobre la Planilla de Ítems y Cómputo" />
      </template>
    </PageHeader>

    <ListState
      :loading="loading"
      :first-load="firstLoad"
      :error="error"
      :is-empty="!orden"
      :is-filtered="false"
      empty-key="Ordenes.Empty"
      class="flex-1 overflow-auto"
      @retry="cargar()"
    >
      <div v-if="orden" class="space-y-4">
        <dl
          class="grid grid-cols-2 gap-3 rounded-md border border-border p-4 text-sm md:grid-cols-4"
        >
          <div>
            <dt class="text-xs text-muted-foreground">{{ $t('Ordenes.Proyecto') }}</dt>
            <dd>{{ orden.proyectoNumero }} · {{ orden.proyectoNombre }}</dd>
          </div>
          <div>
            <dt class="text-xs text-muted-foreground">{{ $t('Ordenes.Cliente') }}</dt>
            <dd>{{ orden.clienteNombre }}</dd>
          </div>
          <div>
            <dt class="text-xs text-muted-foreground">{{ $t('Ordenes.Fecha') }}</dt>
            <dd><DateText :value="orden.fecha" /></dd>
          </div>
          <div>
            <dt class="text-xs text-muted-foreground">{{ $t('Ordenes.UltimoCertificado') }}</dt>
            <dd>{{ orden.numeroCertificado ?? '—' }}</dd>
          </div>
        </dl>

        <DataTable :value="orden.items" data-key="id" size="small" class="text-sm">
          <Column field="descripcion" :header="$t('Ordenes.Descripcion')" />
          <Column field="unidad" :header="$t('Ordenes.Unidad')" />
          <Column field="cantidad" :header="$t('Ordenes.Cantidad')" />
          <Column field="precioUnitario" :header="$t('Ordenes.PrecioUnitario')">
            <template #body="{ data }"><MoneyText :value="data.precioUnitario" /></template>
          </Column>
          <Column field="base" :header="$t('Ordenes.Subtotal')">
            <template #body="{ data }"><MoneyText :value="data.base" /></template>
          </Column>
          <Column field="porcentajeAcumulado" :header="$t('Ordenes.Avance')">
            <template #body="{ data }"><PercentBar :value="data.porcentajeAcumulado" /></template>
          </Column>
          <Column field="subtotalAcumulado" :header="$t('Ordenes.Certificado')">
            <template #body="{ data }"><MoneyText :value="data.subtotalAcumulado" /></template>
          </Column>
          <Column field="nota" :header="$t('Ordenes.Nota')">
            <template #body="{ data }">
              <span v-if="data.nota" :title="data.nota">{{ data.nota }}</span>
              <span v-else class="text-muted-foreground">—</span>
            </template>
          </Column>
        </DataTable>

        <dl class="ml-auto w-full max-w-sm space-y-1 rounded-md border border-border p-4 text-sm">
          <div class="flex justify-between">
            <dt>{{ $t('Ordenes.TotalPresupuestado') }}</dt>
            <dd><MoneyText :value="orden.totalPresupuestado" /></dd>
          </div>
          <div class="flex justify-between">
            <dt>{{ $t('Ordenes.TotalCertificado') }}</dt>
            <dd><MoneyText :value="orden.totalCertificado" /></dd>
          </div>
          <div class="flex justify-between text-muted-foreground">
            <dt>{{ $t('Ordenes.AjusteUocra') }}</dt>
            <dd>− <MoneyText :value="orden.ajusteUocra" /></dd>
          </div>
          <div class="flex justify-between text-muted-foreground">
            <dt>{{ $t('Ordenes.OtrosDescuentos') }}</dt>
            <dd>− <MoneyText :value="orden.otrosDescuentos" /></dd>
          </div>
          <div class="flex justify-between border-t border-border pt-1 font-semibold">
            <dt>{{ $t('Ordenes.TotalNeto') }}</dt>
            <dd><MoneyText :value="orden.totalNeto" /></dd>
          </div>
        </dl>

        <!-- Historial de Certificados de Avance Emitidos para esta Orden -->
        <section class="space-y-3 rounded-lg border border-border bg-surface-card p-4">
          <div class="flex items-center justify-between">
            <div>
              <h2 class="text-sm font-semibold text-foreground">
                {{ $t('Certificados.Subtitle') }}
              </h2>
              <p class="text-xs text-muted-foreground">
                Certificados de avance generados para esta orden de trabajo
              </p>
            </div>
            <Button size="sm" @click="abrirEmision()">
              <AppIcon name="file-badge" :size="14" />
              {{ $t('Certificados.Emitir') }}
            </Button>
          </div>

          <div v-if="certificadosEmitidos.length === 0" class="rounded-md border border-dashed border-border p-6 text-center text-xs text-muted-foreground">
            {{ $t('Certificados.Empty') }}
          </div>
          <DataTable
            v-else
            :value="certificadosEmitidos"
            data-key="id"
            size="small"
            class="text-xs"
          >
            <Column field="numero" :header="$t('Certificados.Numero')">
              <template #body="{ data }">
                <span class="font-semibold text-foreground">#{{ data.numero }}</span>
              </template>
            </Column>
            <Column field="fecha" :header="$t('Certificados.Fecha')">
              <template #body="{ data }">
                <DateText :value="data.fecha" />
              </template>
            </Column>
            <Column field="totalNeto" :header="$t('Certificados.TotalNeto')">
              <template #body="{ data }">
                <MoneyText :value="data.totalNeto" />
              </template>
            </Column>
            <Column :header="$t('General.Actions')" class="w-32 text-right">
              <template #body="{ data }">
                <div class="flex items-center justify-end gap-1">
                  <Button
                    variant="ghost"
                    size="sm"
                    title="Exportar PDF"
                    @click="exportarPdfCertificado(data)"
                  >
                    <AppIcon name="download" :size="14" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    :title="$t('Certificados.VerDetalle')"
                    @click="router.push({ name: 'certificado-detalle', params: { certificadoId: data.id } })"
                  >
                    <AppIcon name="eye" :size="14" />
                  </Button>
                </div>
              </template>
            </Column>
          </DataTable>
        </section>
      </div>
    </ListState>

    <Dialog
      v-model:visible="emisionOpen"
      modal
      maximizable
      :header="$t('Certificados.Emitir')"
      class="w-full max-w-5xl"
    >
      <div v-if="borrador" class="space-y-4">
        <p class="text-sm text-muted-foreground">
          {{
            $t('Certificados.EmitirSubtitulo', {
              numero: borrador.numeroSugerido,
              orden: borrador.ordenTitulo,
            })
          }}
        </p>

        <label class="flex max-w-xs flex-col gap-1">
          <span class="text-sm">{{ $t('Certificados.Fecha') }}</span>
          <DateInput v-model="fechaEmision" :invalid="Boolean(erroresEmision.fecha)" />
          <FieldError id="cert-fecha-error" :message="erroresEmision.fecha" />
        </label>

        <div class="space-y-2">
          <div
            v-for="(item, index) in borrador.items"
            :key="item.ordenTrabajoItemId"
            class="grid grid-cols-12 items-end gap-2 rounded-md border p-2"
            :class="excede(item) ? 'border-state-overdue' : 'border-border'"
          >
            <div class="col-span-12 md:col-span-4">
              <p class="text-sm">{{ item.descripcion }}</p>
              <p class="text-xs text-muted-foreground">
                {{ item.cantidad }} {{ item.unidad }} ·
                <MoneyText :value="item.precioUnitario" />
              </p>
            </div>
            <div class="col-span-6 md:col-span-2">
              <span class="text-xs text-muted-foreground">
                {{ $t('Certificados.AcumuladoAnterior') }}
              </span>
              <PercentBar :value="item.porcentajeAcumuladoAnterior" />
            </div>
            <label class="col-span-6 flex flex-col gap-1 md:col-span-2">
              <span class="text-xs text-muted-foreground">{{
                $t('Certificados.AvanceAhora')
              }}</span>
              <DecimalInput
                :model-value="avances[item.ordenTrabajoItemId] ?? '0.0000'"
                :min="0"
                :max="100"
                suffix=" %"
                :invalid="
                  excede(item) || Boolean(erroresEmision[`items[${index}].porcentajeActual`])
                "
                @update:model-value="avances[item.ordenTrabajoItemId] = $event"
              />
              <FieldError
                :id="`cert-item-${index}-error`"
                :message="erroresEmision[`items[${index}].porcentajeActual`]"
              />
            </label>
            <div class="col-span-6 md:col-span-2">
              <span class="text-xs text-muted-foreground">{{ $t('Certificados.Acumulado') }}</span>
              <PercentBar :value="acumuladoDe(item)" />
            </div>
            <div class="col-span-6 text-right md:col-span-2">
              <span class="text-xs text-muted-foreground">{{ $t('Certificados.Subtotal') }}</span>
              <p><MoneyText :value="subtotalDe(item)" /></p>
            </div>
          </div>
        </div>

        <p v-if="hayExcedidos" class="text-sm text-state-overdue">
          {{ $t('Certificados.AcumuladoExcedidoAviso') }}
        </p>

        <label class="flex flex-col gap-1">
          <span class="text-sm">{{ $t('Certificados.Observaciones') }}</span>
          <Textarea v-model="observacionesEmision" rows="2" auto-resize />
        </label>

        <dl class="space-y-1.5 rounded-md border border-border bg-surface-card p-3 text-sm">
          <div class="flex justify-between">
            <dt class="text-muted-foreground">{{ $t('Certificados.TotalCertificado') }}</dt>
            <dd><MoneyText :value="totalCertificadoAEmitir" /></dd>
          </div>
          <div v-if="Number(ajusteUocraAEmitir) > 0" class="flex justify-between text-muted-foreground">
            <dt>{{ $t('Certificados.AjusteUocra') }} (+{{ borrador.ajusteUocraPorcentaje }}%)</dt>
            <dd>+ <MoneyText :value="ajusteUocraAEmitir" /></dd>
          </div>
          <div v-if="Number(otrosDescuentosAEmitir) > 0" class="flex justify-between text-muted-foreground">
            <dt>{{ $t('Certificados.OtrosDescuentos') }}</dt>
            <dd>- <MoneyText :value="otrosDescuentosAEmitir" /></dd>
          </div>
          <div class="flex justify-between border-t border-border pt-1 font-semibold">
            <dt>{{ $t('Certificados.TotalNeto') }}</dt>
            <dd><MoneyText :value="totalNetoAEmitir" colored /></dd>
          </div>
        </dl>
      </div>

      <template #footer>
        <Button variant="outline" :disabled="emitiendo" @click="emisionOpen = false">
          {{ $t('General.Cancel') }}
        </Button>
        <Button :disabled="emitiendo || hayExcedidos || !hayAvance" @click="emitir()">
          {{ $t('Certificados.Emitir') }}
        </Button>
      </template>
    </Dialog>

    <!-- Dialog for editing the work order directly from its detail view -->
    <Dialog
      v-model:visible="editorOpen"
      modal
      maximizable
      :header="$t('General.Edit') + ' - ' + (editor.titulo || $t('Ordenes.Title'))"
      :style="{ width: '80vw', maxWidth: '1100px' }"
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
        <Button variant="outline" :disabled="savingEditor" @click="editorOpen = false">
          {{ $t('General.Cancel') }}
        </Button>
        <Button :disabled="savingEditor" @click="guardarEdicion()">
          {{ $t('General.Save') }}
        </Button>
      </template>
    </Dialog>
  </section>
</template>
