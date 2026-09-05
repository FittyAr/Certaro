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
import { useApiError } from '@/composables/useApiError'
import { useConfirmDelete } from '@/composables/useConfirmDelete'
import { useCatalogStore, type LookupItem } from '@/stores/useCatalogStore'
import { useCategoriasStore } from '@/stores/useCategoriasStore'
import { useMovimientosStore, type MovimientoListItem } from '@/stores/useMovimientosStore'
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
const categoriasStore = useCategoriasStore()
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

const modoCobro = ref<'nuevo' | 'anticipo'>('nuevo')
const anticiposDisponibles = ref<MovimientoListItem[]>([])
const anticipoSeleccionadoId = ref<string | null>(null)

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
      } else if (factura.value.observaciones && /\[proy:([a-f0-9-]+)\]/i.test(factura.value.observaciones)) {
        const match = factura.value.observaciones.match(/\[proy:([a-f0-9-]+)\]/i)
        if (match && match[1]) {
          pagoProyectoId.value = match[1]
          await onPagoProyectoChange()
        }
      } else if (opcionesProyectos.value.length === 1 && opcionesProyectos.value[0]) {
        pagoProyectoId.value = opcionesProyectos.value[0].id
        await onPagoProyectoChange()
      }

      await cargarAnticipos(factura.value.clienteId)
    } catch (e) {
      notify(e)
    }
  }
)

async function cargarAnticipos(clienteId: string): Promise<void> {
  anticiposDisponibles.value = []
  anticipoSeleccionadoId.value = null
  modoCobro.value = 'nuevo'
  try {
    const res = await movimientosStore.fetchPaged({
      filtro: {
        clienteId,
      },
      page: 1,
      pageSize: 0,
    })
    // Unapplied advances: income movements for this client that are not tied to any invoice yet
    anticiposDisponibles.value = res.items.filter((m) => m.esIngreso && !m.facturaId)
  } catch (e) {
    console.warn('No se pudieron cargar los anticipos del cliente:', e)
  }
}

function onAnticipoChange(): void {
  if (!anticipoSeleccionadoId.value) return
  const anticipo = anticiposDisponibles.value.find((a) => a.id === anticipoSeleccionadoId.value)
  if (!anticipo || !factura.value) return

  // Auto-set the date from the advance and cap the amount to min(anticipo.total, factura.saldo)
  nuevoPago.value.fecha = anticipo.fecha.slice(0, 10)
  const montoAnticipo = Number(anticipo.total)
  const saldoFactura = Number(factura.value.saldo)
  const montoAImputar = Math.min(montoAnticipo, saldoFactura).toFixed(4)
  nuevoPago.value.monto = montoAImputar
  nuevoPago.value.medioPago = 'Transferencia'
}

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
    const created = await categoriasStore.create({
      nombre: 'Cobranzas',
      descripcion: 'Categoría automática para cobranzas de facturas',
      colorHex: null,
      icono: 'wallet',
      categoriaPadreId: null,
    })
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

    if (modoCobro.value === 'anticipo' && anticipoSeleccionadoId.value) {
      // Link the existing advance to this invoice so it cannot be reused, WITHOUT duplicating cash movement
      try {
        const anticipoItem = anticiposDisponibles.value.find((a) => a.id === anticipoSeleccionadoId.value)
        if (anticipoItem && factura.value) {
          await movimientosStore.update(
            anticipoItem.id,
            {
              fecha: anticipoItem.fecha,
              concepto: `${anticipoItem.concepto} (Imputado a Factura ${factura.value.numero})`,
              monto: anticipoItem.monto,
              cantidad: anticipoItem.cantidad,
              tipoMovimientoId: anticipoItem.tipoMovimientoId,
              moneda: anticipoItem.moneda,
              cotizacionAplicada: anticipoItem.cotizacionAplicada,
              tipoConceptoPagoId: anticipoItem.tipoConceptoPagoId,
              categoriaId: anticipoItem.categoriaId,
              clienteId: anticipoItem.clienteId,
              trabajoId: anticipoItem.trabajoId,
              empleadoId: anticipoItem.empleadoId,
              facturaId: factura.value.id,
            },
            anticipoItem.rowVersion,
          )
        }
      } catch (err) {
        console.warn('No se pudo vincular el anticipo con la factura:', err)
      }
    } else if (registrarEnCaja.value && factura.value) {
      try {
        const catId = await resolveCategoriaCobranza()
        if (catId) {
          const pagoCreado = factura.value.pagos.find(
            (p) => p.fecha === pagoFecha && p.medioPago === pagoMedio && p.monto === pagoMonto,
          ) ?? factura.value.pagos[factura.value.pagos.length - 1]
          const refTexto = pagoCreado ? ` · ref:${pagoCreado.id.slice(0, 8)}` : ''

          await movimientosStore.create({
            fecha: fechaPagoToIso(pagoFecha),
            concepto: `Cobranza Factura ${factura.value.numero} (${pagoMedio}${refTexto})`,
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

    if (factura.value) {
      await cargarAnticipos(factura.value.clienteId)
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
        class="border-t border-border pt-3 flex flex-col gap-3"
      >
        <!-- Mode switcher if advances exist -->
        <div v-if="anticiposDisponibles.length > 0" class="flex items-center gap-4 rounded border border-info/30 bg-info/10 p-2.5 text-xs">
          <span class="font-medium text-info-foreground">Modo de registro:</span>
          <label class="flex items-center gap-1.5 cursor-pointer select-none">
            <input
              type="radio"
              value="nuevo"
              v-model="modoCobro"
              class="text-primary focus:ring-0"
            />
            <span>Nuevo cobro / pago directo</span>
          </label>
          <label class="flex items-center gap-1.5 cursor-pointer select-none">
            <input
              type="radio"
              value="anticipo"
              v-model="modoCobro"
              class="text-primary focus:ring-0"
            />
            <span class="font-semibold text-primary">
              Imputar anticipo / seña previa ({{ anticiposDisponibles.length }} disponible{{ anticiposDisponibles.length > 1 ? 's' : '' }})
            </span>
          </label>
        </div>

        <!-- Advance selector -->
        <div v-if="modoCobro === 'anticipo'" class="rounded border border-border/80 bg-surface-raised p-3 flex flex-col gap-2">
          <label class="flex flex-col gap-1">
            <span class="text-xs font-medium text-foreground">Seleccionar anticipo a imputar:</span>
            <Select
              v-model="anticipoSeleccionadoId"
              :options="anticiposDisponibles"
              option-label="concepto"
              option-value="id"
              placeholder="Elija un anticipo existente..."
              @change="onAnticipoChange"
            >
              <template #option="{ option }">
                <div class="flex justify-between items-center w-full gap-4 text-xs">
                  <span>{{ option.fecha.slice(0, 10) }} · {{ option.concepto }}</span>
                  <span class="font-semibold text-money-positive">${{ option.total }}</span>
                </div>
              </template>
            </Select>
          </label>
          <p class="text-[11px] text-muted-foreground">
            Al imputar un anticipo, se asocia el movimiento de caja existente a esta factura sin duplicar el ingreso en el libro diario.
          </p>
        </div>

        <div class="grid grid-cols-4 items-end gap-3">
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
          <div v-if="modoCobro === 'nuevo' && registrarEnCaja" class="col-span-4 grid grid-cols-2 gap-3 rounded border border-border/70 bg-muted/20 p-2.5">
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
            <label v-if="modoCobro === 'nuevo'" class="flex items-center gap-2 text-xs text-muted-foreground cursor-pointer select-none">
              <Checkbox v-model="registrarEnCaja" :binary="true" />
              <span>{{ $t('Facturas.RegistrarEnCaja') }}</span>
            </label>
            <span v-else class="text-xs text-muted-foreground italic">
              El movimiento ya existe en caja (se vinculará a la factura).
            </span>
            <Button
              :disabled="modoCobro === 'anticipo' && !anticipoSeleccionadoId"
              @click="registrarPago()"
            >
              <AppIcon name="plus" :size="16" />
              {{ modoCobro === 'anticipo' ? 'Imputar Anticipo' : $t('Facturas.RegistrarPago') }}
            </Button>
          </div>
        </div>
      </div>
      <p v-else class="text-xs text-muted-foreground">{{ $t('Facturas.NoAdmitePagos') }}</p>
    </div>
  </Dialog>
</template>
