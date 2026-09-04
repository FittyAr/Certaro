<script setup lang="ts">
import Checkbox from 'primevue/checkbox'
import Column from 'primevue/column'
import DataTable from 'primevue/datatable'
import Dialog from 'primevue/dialog'
import Select from 'primevue/select'
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import DateInput from '@/components/domain/DateInput.vue'
import DateText from '@/components/domain/DateText.vue'
import FieldError from '@/components/domain/FieldError.vue'
import MoneyInput from '@/components/domain/MoneyInput.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import StatePill from '@/components/domain/StatePill.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import { createCategoria } from '@/api/categorias'
import { useApiError } from '@/composables/useApiError'
import { useConfirmDelete } from '@/composables/useConfirmDelete'
import { useCatalogStore, type LookupItem } from '@/stores/useCatalogStore'
import { useMovimientosStore } from '@/stores/useMovimientosStore'
import { useProyectosStore } from '@/stores/useProyectosStore'
import { useTrabajosStore } from '@/stores/useTrabajosStore'
import {
  MEDIOS_PAGO,
  useFacturasStore,
  type FacturaDetalle,
  type PagoFacturaInput,
} from '@/stores/useFacturasStore'

const props = defineProps<{
  visible: boolean
  facturaId: string | null
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
  (e: 'pagoModificado'): void
}>()

const { t } = useI18n()
const { fieldErrors, notify } = useApiError()
const { confirmDelete } = useConfirmDelete()
const store = useFacturasStore()
const catalogStore = useCatalogStore()
const movimientosStore = useMovimientosStore()
const proyectosStore = useProyectosStore()
const trabajosStore = useTrabajosStore()

const factura = ref<FacturaDetalle | null>(null)
const registrarEnCaja = ref(true)
const pagoProyectoId = ref<string | null>(null)
const pagoTrabajoId = ref<string | null>(null)
const opcionesProyectos = ref<LookupItem[]>([])
const pagoOpcionesTrabajos = ref<LookupItem[]>([])
const pagoErrores = ref<Record<string, string>>({})

function hoy(): string {
  return new Date().toISOString().slice(0, 10)
}

const nuevoPago = ref<PagoFacturaInput>({
  facturaId: '',
  fecha: hoy(),
  monto: '0.0000',
  medioPago: 'Efectivo',
})

const medioPagoOptions = computed(() =>
  MEDIOS_PAGO.map((value) => ({ label: t(`MedioPago.${value}`), value })),
)

async function onPagoProyectoChange(): Promise<void> {
  pagoTrabajoId.value = null
  if (!pagoProyectoId.value) {
    pagoOpcionesTrabajos.value = []
    return
  }
  try {
    pagoOpcionesTrabajos.value = await trabajosStore.lookup(pagoProyectoId.value)
    if (pagoOpcionesTrabajos.value.length === 1 && pagoOpcionesTrabajos.value[0]) {
      pagoTrabajoId.value = pagoOpcionesTrabajos.value[0].id
    }
  } catch {
    pagoOpcionesTrabajos.value = []
  }
}

watch(
  () => [props.visible, props.facturaId],
  async ([vis, id]) => {
    if (!vis || !id) {
      factura.value = null
      return
    }
    try {
      factura.value = await store.fetchOne(id as string)
      nuevoPago.value = {
        facturaId: id as string,
        fecha: hoy(),
        monto: factura.value.saldo,
        medioPago: 'Efectivo',
      }
      pagoErrores.value = {}
      pagoProyectoId.value = null
      pagoTrabajoId.value = null
      pagoOpcionesTrabajos.value = []

      try {
        opcionesProyectos.value = await proyectosStore.lookup(factura.value.clienteId)
      } catch {
        opcionesProyectos.value = []
      }

      const cachedObra = localStorage.getItem(`certaro:factura-obra:${id}`)
      if (cachedObra) {
        try {
          const parsed = JSON.parse(cachedObra)
          if (parsed.proyectoId) {
            pagoProyectoId.value = parsed.proyectoId
            await onPagoProyectoChange()
          }
          if (parsed.trabajoId) {
            pagoTrabajoId.value = parsed.trabajoId
          }
        } catch {
          // ignore
        }
      } else if (opcionesProyectos.value.length === 1 && opcionesProyectos.value[0]) {
        pagoProyectoId.value = opcionesProyectos.value[0].id
        await onPagoProyectoChange()
      }
    } catch (e) {
      notify(e)
    }
  }
)

function fechaPagoToIso(fechaStr: string): string {
  if (!fechaStr) return new Date().toISOString()
  const partes = fechaStr.split('-').map(Number)
  if (partes.length === 3 && partes[0] && partes[1] && partes[2]) {
    const now = new Date()
    return new Date(partes[0], partes[1] - 1, partes[2], now.getHours(), now.getMinutes(), now.getSeconds()).toISOString()
  }
  return new Date().toISOString()
}

async function resolveCategoriaCobranza(): Promise<string | null> {
  const cats = await catalogStore.loadCategorias()
  const catCobranza = cats.find((c) => {
    const l = c.label.toLowerCase()
    return l.includes('cobranza') || l.includes('venta') || l.includes('ingreso') || l.includes('factura')
  })
  if (catCobranza) return catCobranza.id
  if (cats.length > 0 && cats[0]) return cats[0].id

  try {
    const created = await createCategoria({
      nombre: 'Cobranzas',
      descripcion: 'Categoría automática para cobranzas de facturas',
      colorHex: '#10B981',
      icono: 'wallet',
      categoriaPadreId: null,
    })
    catalogStore.invalidateCategorias()
    return created.id
  } catch {
    return null
  }
}

async function registrarPago(): Promise<void> {
  pagoErrores.value = {}
  try {
    const pagoMonto = nuevoPago.value.monto
    const pagoMedio = nuevoPago.value.medioPago
    const pagoFecha = nuevoPago.value.fecha
    factura.value = await store.crearPago(nuevoPago.value)

    if (registrarEnCaja.value && factura.value) {
      try {
        const catId = await resolveCategoriaCobranza()
        if (catId) {
          await movimientosStore.create({
            fecha: fechaPagoToIso(pagoFecha),
            concepto: `Cobranza Factura ${factura.value.numero} (${pagoMedio})`,
            monto: pagoMonto,
            cantidad: '1.0000',
            tipoMovimientoId: '00000000-0000-0000-0000-000000000001',
            moneda: 'Ars',
            cotizacionAplicada: null,
            tipoConceptoPagoId: null,
            categoriaId: catId,
            clienteId: factura.value.clienteId,
            trabajoId: pagoTrabajoId.value || null,
            empleadoId: null,
            facturaId: factura.value.id,
          })
        } else {
          console.warn('No se pudo determinar una categoría para asentar el cobro en caja.')
        }
      } catch (err) {
        notify(err)
      }
    }

    nuevoPago.value = {
      facturaId: factura.value.id,
      fecha: hoy(),
      monto: factura.value.saldo,
      medioPago: 'Efectivo',
    }
    emit('pagoModificado')
  } catch (e) {
    const error = notify(e)
    if (error.code === 'VALIDATION') pagoErrores.value = fieldErrors(error)
  }
}

function borrarPago(pago: { id: string; rowVersion: string; fecha: string; medioPago: string; monto: string }): void {
  confirmDelete({
    entityKey: 'Facturas.Pagos',
    label: `${pago.fecha} · ${pago.medioPago} ($${pago.monto})`,
    action: async () => {
      factura.value = await store.borrarPago(pago.id, pago.rowVersion)
      emit('pagoModificado')
    },
  })
}
</script>

<template>
  <Dialog
    :visible="visible"
    modal
    :header="$t('Facturas.Pagos')"
    class="w-[42rem]"
    :dismissable-mask="true"
    @update:visible="emit('update:visible', $event)"
  >
    <div v-if="factura" class="flex flex-col gap-4">
      <div class="flex flex-wrap items-center gap-6 text-sm">
        <span class="font-medium">{{ factura.numero }}</span>
        <span>{{ factura.clienteNombre }}</span>
        <StatePill entity="Factura" :value="factura.estado.actual" />
        <span class="flex items-center gap-2">
          <span class="text-muted-foreground">{{ $t('Facturas.Total') }}</span>
          <MoneyText :value="factura.total" />
        </span>
        <span class="flex items-center gap-2">
          <span class="text-muted-foreground">{{ $t('Facturas.Saldo') }}</span>
          <MoneyText :value="factura.saldo" colored />
        </span>
      </div>

      <DataTable :value="factura.pagos" size="small" class="text-sm">
        <Column field="fecha" :header="$t('Facturas.Fecha')">
          <template #body="{ data }"><DateText :value="data.fecha" /></template>
        </Column>
        <Column field="medioPago" :header="$t('Facturas.MedioPago')" />
        <Column field="monto" :header="$t('Facturas.Monto')">
          <template #body="{ data }"><MoneyText :value="data.monto" /></template>
        </Column>
        <Column>
          <template #body="{ data }">
            <Button variant="ghost" size="sm" @click="borrarPago(data)">
              <AppIcon name="trash-2" :size="14" />
            </Button>
          </template>
        </Column>
        <template #empty>
          <span class="text-muted-foreground">{{ $t('Facturas.SinPagos') }}</span>
        </template>
      </DataTable>

      <div
        v-if="factura.admitePagos"
        class="grid grid-cols-4 items-end gap-3 border-t border-border pt-3"
      >
        <label class="flex flex-col gap-1">
          <span class="text-xs text-muted-foreground">{{ $t('Facturas.Fecha') }}</span>
          <DateInput v-model="nuevoPago.fecha" :invalid="Boolean(pagoErrores.fecha)" />
          <FieldError id="pago-fecha-error" :message="pagoErrores.fecha" />
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-xs text-muted-foreground">{{ $t('Facturas.Monto') }}</span>
          <MoneyInput v-model="nuevoPago.monto" :min="0" :invalid="Boolean(pagoErrores.monto)" />
          <FieldError id="pago-monto-error" :message="pagoErrores.monto" />
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-xs text-muted-foreground">{{ $t('Facturas.MedioPago') }}</span>
          <Select
            v-model="nuevoPago.medioPago"
            :options="medioPagoOptions"
            option-label="label"
            option-value="value"
            editable
          />
        </label>
        <div v-if="registrarEnCaja" class="col-span-4 grid grid-cols-2 gap-3 rounded border border-border/70 bg-muted/20 p-2.5">
          <label class="flex flex-col gap-1">
            <span class="text-xs text-muted-foreground">Imputar a Proyecto / Obra</span>
            <Select
              v-model="pagoProyectoId"
              :options="opcionesProyectos"
              option-label="label"
              option-value="id"
              filter
              show-clear
              placeholder="General (Sin proyecto)"
              @change="onPagoProyectoChange()"
            />
          </label>
          <label class="flex flex-col gap-1">
            <span class="text-xs text-muted-foreground">Trabajo / Frente de Obra</span>
            <Select
              v-model="pagoTrabajoId"
              :options="pagoOpcionesTrabajos"
              option-label="label"
              option-value="id"
              filter
              show-clear
              placeholder="General"
              :disabled="!pagoProyectoId && pagoOpcionesTrabajos.length === 0"
            />
          </label>
        </div>

        <div class="col-span-4 flex items-center justify-between pt-2">
          <label class="flex items-center gap-2 text-xs text-muted-foreground cursor-pointer select-none">
            <Checkbox v-model="registrarEnCaja" :binary="true" />
            <span>{{ $t('Facturas.RegistrarEnCaja') }}</span>
          </label>
          <Button @click="registrarPago()">
            <AppIcon name="plus" :size="16" />
            {{ $t('Facturas.RegistrarPago') }}
          </Button>
        </div>
      </div>
      <p v-else class="text-xs text-muted-foreground">{{ $t('Facturas.NoAdmitePagos') }}</p>
    </div>
  </Dialog>
</template>
