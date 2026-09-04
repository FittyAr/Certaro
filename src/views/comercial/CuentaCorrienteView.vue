<script setup lang="ts">
import Column from 'primevue/column'
import DataTable from 'primevue/datatable'
import ToggleSwitch from 'primevue/toggleswitch'
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'

import Dialog from 'primevue/dialog'
import Select from 'primevue/select'
import DateInput from '@/components/domain/DateInput.vue'
import DateText from '@/components/domain/DateText.vue'
import FieldError from '@/components/domain/FieldError.vue'
import ListState from '@/components/domain/ListState.vue'
import MoneyInput from '@/components/domain/MoneyInput.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
import StatePill from '@/components/domain/StatePill.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import HelpButton from '@/components/ui/HelpButton.vue'
import { Button } from '@/components/ui/button'
import { useApiError, type ApiError } from '@/composables/useApiError'
import { useMovimientosStore } from '@/stores/useMovimientosStore'
import {
  MEDIOS_PAGO,
  useFacturasStore,
  type FacturaDetalle,
  type PagoFacturaInput,
} from '@/stores/useFacturasStore'
import {
  useComercialStore,
  type AntiguedadDeuda,
  type CuentaCorriente,
  type CuentaCorrienteFactura,
} from '@/stores/useComercialStore'

/**
 * A customer's account statement with the ageing of their debt. See `docs/09` §3.3.
 *
 * An unknown customer yields an empty statement rather than an error (doc 06 §4.5): this screen is
 * reached from links that outlive the record they point at.
 */

const { t } = useI18n()
const route = useRoute()
const router = useRouter()
const { notify, fieldErrors } = useApiError()
const store = useComercialStore()
const facturasStore = useFacturasStore()
const movimientosStore = useMovimientosStore()

const clienteId = computed(() => String(route.params.clienteId ?? ''))

const cuenta = ref<CuentaCorriente | null>(null)
const antiguedad = ref<AntiguedadDeuda | null>(null)
const incluirPagadas = ref(false)
const loading = ref(false)
const firstLoad = ref(true)
const error = ref<ApiError | null>(null)

async function cargar(): Promise<void> {
  if (!clienteId.value) return
  loading.value = true
  error.value = null
  try {
    const [statement, aging] = await Promise.all([
      store.fetchCuentaCorriente({
        clienteId: clienteId.value,
        incluirPagadas: incluirPagadas.value,
      }),
      store.fetchAntiguedad({ clienteId: clienteId.value }),
    ])
    cuenta.value = statement
    antiguedad.value = aging
  } catch (e) {
    error.value = notify(e)
  } finally {
    loading.value = false
    firstLoad.value = false
  }
}

watch(incluirPagadas, () => void cargar())

/**
 * The bucket bounds come with the report so the columns are labelled from configuration and never
 * from a hardcoded 30/60/90 (doc 06 §4.6).
 */
const buckets = computed(() => {
  const a = antiguedad.value
  if (!a) return []
  const [primero = 30, segundo = 60, tercero = 90] = a.limites
  return [
    { key: 'b1', label: `0-${primero}`, value: a.bucket0a30 },
    { key: 'b2', label: `${primero + 1}-${segundo}`, value: a.bucket31a60 },
    { key: 'b3', label: `${segundo + 1}-${tercero}`, value: a.bucket61a90 },
    { key: 'b4', label: `+${tercero}`, value: a.bucketMas90 },
  ]
})

/** Days in arrears are coloured by the bucket they fall in, as the screen spec asks. */
function claseMora(dias: number): string {
  const [primero = 30, segundo = 60, tercero = 90] = antiguedad.value?.limites ?? []
  if (dias <= 0) return 'text-muted-foreground'
  if (dias <= primero) return 'text-money-neutral'
  if (dias <= segundo) return 'text-warning'
  return dias <= tercero ? 'text-money-negative' : 'text-destructive'
}

function verFactura(factura: CuentaCorrienteFactura): void {
  void router.push({ name: 'facturas', query: { id: factura.id } })
}

// ------------------------------------------------------------------- direct payment

function hoy(): string {
  return new Date().toISOString().slice(0, 10)
}

const pagosVisible = ref(false)
const factura = ref<FacturaDetalle | null>(null)
const registrarEnCaja = ref(true)
const guardandoPago = ref(false)
const nuevoPago = ref<PagoFacturaInput>({
  facturaId: '',
  fecha: hoy(),
  monto: '0.0000',
  medioPago: 'Efectivo',
})
const pagoErrores = ref<Record<string, string>>({})

const medioPagoOptions = computed(() =>
  MEDIOS_PAGO.map((value) => ({ label: t(`MedioPago.${value}`), value })),
)

async function abrirCobro(f: CuentaCorrienteFactura): Promise<void> {
  try {
    factura.value = await facturasStore.fetchOne(f.id)
    nuevoPago.value = {
      facturaId: f.id,
      fecha: hoy(),
      monto: factura.value.saldo,
      medioPago: 'Efectivo',
    }
    pagoErrores.value = {}
    pagosVisible.value = true
  } catch (e) {
    notify(e)
  }
}

async function registrarPago(): Promise<void> {
  pagoErrores.value = {}
  guardandoPago.value = true
  try {
    const pagoMonto = nuevoPago.value.monto
    const pagoMedio = nuevoPago.value.medioPago
    const pagoFecha = nuevoPago.value.fecha
    factura.value = await facturasStore.crearPago(nuevoPago.value)

function fechaPagoToIso(fechaStr: string): string {
  if (!fechaStr) return new Date().toISOString()
  const partes = fechaStr.split('-').map(Number)
  if (partes.length === 3 && partes[0] && partes[1] && partes[2]) {
    const now = new Date()
    return new Date(partes[0], partes[1] - 1, partes[2], now.getHours(), now.getMinutes(), now.getSeconds()).toISOString()
  }
  return new Date().toISOString()
}

    if (registrarEnCaja.value && factura.value) {
      let imputacionTrabajoId: string | null = null
      try {
        const cachedObra = localStorage.getItem(`certaro:factura-obra:${factura.value.id}`)
        if (cachedObra) {
          const parsed = JSON.parse(cachedObra)
          if (parsed.trabajoId) imputacionTrabajoId = parsed.trabajoId
        }
      } catch {
        // ignore
      }

      try {
        await movimientosStore.create({
          fecha: fechaPagoToIso(pagoFecha),
          concepto: `Cobranza Factura ${factura.value.numero} (${pagoMedio})`,
          monto: pagoMonto,
          cantidad: '1.0000',
          tipoMovimientoId: '00000000-0000-0000-0000-000000000001',
          moneda: 'Ars',
          cotizacionAplicada: null,
          tipoConceptoPagoId: null,
          categoriaId: null,
          clienteId: factura.value.clienteId,
          trabajoId: imputacionTrabajoId,
          empleadoId: null,
          facturaId: factura.value.id,
        })
      } catch (err) {
        console.warn('No se pudo registrar automáticamente el ingreso en caja:', err)
      }
    }

    pagosVisible.value = false
    await cargar()
  } catch (e) {
    const err = notify(e)
    if (err.code === 'VALIDATION') pagoErrores.value = fieldErrors(err)
  } finally {
    guardandoPago.value = false
  }
}

onMounted(cargar)
</script>

<template>
  <section class="flex h-full flex-col gap-4 p-6">
    <PageHeader
      :title="$t('Comercial.CuentaCorriente.Title')"
      :subtitle="cuenta?.clienteNombre || undefined"
    >
      <template #actions>
        <label class="flex items-center gap-2 text-sm cursor-pointer select-none">
          <ToggleSwitch v-model="incluirPagadas" />
          <span class="font-medium text-foreground/90">{{ $t('Comercial.CuentaCorriente.IncluirPagadas') }}</span>
        </label>
        <Button variant="outline" @click="router.back()">
          <AppIcon name="arrow-left" :size="16" />
          {{ $t('General.Back') }}
        </Button>
        <HelpButton topic-id="clientes-cuenta-corriente" title="Ayuda sobre Cuentas Corrientes" />
      </template>
    </PageHeader>

    <ListState
      :loading="loading"
      :first-load="firstLoad"
      :error="error"
      :is-empty="!cuenta"
      :is-filtered="false"
      empty-key="Comercial.CuentaCorriente.Empty"
      class="flex-1"
      @retry="cargar()"
    >
      <div v-if="cuenta" class="space-y-4">
        <dl class="grid grid-cols-1 gap-3 sm:grid-cols-3">
          <div class="rounded-lg border border-border bg-surface-card p-4">
            <dt class="text-sm text-muted-foreground">
              {{ $t('Comercial.CuentaCorriente.TotalFacturado') }}
            </dt>
            <dd class="text-xl font-semibold">
              <MoneyText :value="cuenta.totalFacturado" />
            </dd>
          </div>
          <div class="rounded-lg border border-border bg-surface-card p-4">
            <dt class="text-sm text-muted-foreground">
              {{ $t('Comercial.CuentaCorriente.TotalPagado') }}
            </dt>
            <dd class="text-xl font-semibold"><MoneyText :value="cuenta.totalPagado" /></dd>
          </div>
          <div class="rounded-lg border border-border bg-surface-card p-4">
            <dt class="text-sm text-muted-foreground">
              {{ $t('Comercial.CuentaCorriente.Saldo') }}
            </dt>
            <dd class="text-xl font-semibold"><MoneyText :value="cuenta.saldo" colored /></dd>
          </div>
        </dl>

        <section v-if="antiguedad" class="rounded-lg border border-border bg-surface-card p-4">
          <h3 class="mb-3 text-sm font-semibold">
            {{ $t('Comercial.Antiguedad.Title') }}
            <span class="font-normal text-muted-foreground">
              {{ $t('Comercial.Antiguedad.AlCorte') }}
              <DateText :value="antiguedad.fechaCorte" />
            </span>
          </h3>
          <dl class="grid grid-cols-2 gap-3 md:grid-cols-4">
            <div v-for="bucket in buckets" :key="bucket.key">
              <dt class="text-xs text-muted-foreground">
                {{ $t('Comercial.Antiguedad.Dias', { rango: bucket.label }) }}
              </dt>
              <dd class="text-sm font-medium"><MoneyText :value="bucket.value" /></dd>
            </div>
          </dl>
        </section>

        <DataTable
          :value="cuenta.facturas"
          data-key="id"
          size="small"
          scrollable
          scroll-height="flex"
          class="flex-1 text-sm"
          @row-dblclick="verFactura($event.data as CuentaCorrienteFactura)"
        >
          <template #empty>
            <p class="p-4 text-center text-sm text-muted-foreground">
              {{ $t('Comercial.CuentaCorriente.SinDeuda') }}
            </p>
          </template>
          <Column field="numero" :header="$t('Facturas.Numero')" sortable />
          <Column field="fecha" :header="$t('Facturas.Fecha')" sortable>
            <template #body="{ data }"><DateText :value="data.fecha" /></template>
          </Column>
          <Column field="fechaVencimiento" :header="$t('Facturas.Vencimiento')" sortable>
            <template #body="{ data }"><DateText :value="data.fechaVencimiento" /></template>
          </Column>
          <Column field="estado" :header="$t('Facturas.Estado')">
            <template #body="{ data }"><StatePill entity="Factura" :value="data.estado" /></template>
          </Column>
          <Column field="total" :header="$t('Facturas.Total')" sortable>
            <template #body="{ data }"><MoneyText :value="data.total" /></template>
          </Column>
          <Column field="pagado" :header="$t('Facturas.Pagado')" sortable>
            <template #body="{ data }"><MoneyText :value="data.pagado" /></template>
          </Column>
          <Column field="saldo" :header="$t('Facturas.Saldo')" sortable>
            <template #body="{ data }"><MoneyText :value="data.saldo" colored /></template>
          </Column>
          <Column field="diasMora" :header="$t('Comercial.CuentaCorriente.DiasMora')" sortable>
            <template #body="{ data }">
              <span class="tabular-nums" :class="claseMora(data.diasMora)">{{ data.diasMora }}</span>
            </template>
          </Column>
          <Column :header="$t('General.Actions')" class="w-24 text-right">
            <template #body="{ data }">
              <div class="flex items-center justify-end gap-1">
                <Button
                  v-if="Number(data.saldo) > 0"
                  size="sm"
                  variant="outline"
                  title="Registrar Cobro"
                  class="h-7 px-2 text-xs"
                  @click="abrirCobro(data)"
                >
                  <AppIcon name="wallet" :size="12" />
                  <span class="ml-1">Cobrar</span>
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  :title="$t('General.View')"
                  class="h-7 w-7 p-0"
                  @click="verFactura(data)"
                >
                  <AppIcon name="eye" :size="13" />
                </Button>
              </div>
            </template>
          </Column>
        </DataTable>
      </div>
    </ListState>

    <!-- Dialog para registrar pago directo -->
    <Dialog
      v-model:visible="pagosVisible"
      modal
      :header="$t('Facturas.RegistrarPago')"
      class="w-[32rem]"
      :dismissable-mask="true"
    >
      <div v-if="factura" class="flex flex-col gap-4">
        <div class="flex items-center justify-between rounded-md border border-border bg-muted/20 p-3 text-sm">
          <div>
            <span class="text-xs text-muted-foreground">{{ $t('Facturas.Numero') }}</span>
            <p class="font-semibold text-foreground">{{ factura.numero }}</p>
          </div>
          <div class="text-right">
            <span class="text-xs text-muted-foreground">{{ $t('Facturas.Saldo') }}</span>
            <p><MoneyText :value="factura.saldo" colored /></p>
          </div>
        </div>

        <div class="grid grid-cols-2 gap-3">
          <label class="flex flex-col gap-1">
            <span class="text-xs text-muted-foreground">{{ $t('Facturas.Fecha') }}</span>
            <DateInput v-model="nuevoPago.fecha" :invalid="Boolean(pagoErrores.fecha)" />
            <FieldError id="cc-pago-fecha-error" :message="pagoErrores.fecha" />
          </label>
          <label class="flex flex-col gap-1">
            <span class="text-xs text-muted-foreground">{{ $t('Facturas.Monto') }}</span>
            <MoneyInput v-model="nuevoPago.monto" :min="0" :invalid="Boolean(pagoErrores.monto)" />
            <FieldError id="cc-pago-monto-error" :message="pagoErrores.monto" />
          </label>
        </div>

        <label class="flex flex-col gap-1">
          <span class="text-xs text-muted-foreground">{{ $t('Facturas.MedioPago') }}</span>
          <Select
            v-model="nuevoPago.medioPago"
            :options="medioPagoOptions"
            option-label="label"
            option-value="value"
          />
        </label>

        <label class="flex items-center gap-2 cursor-pointer select-none rounded-md border border-border/80 bg-surface-raised p-2.5">
          <ToggleSwitch v-model="registrarEnCaja" />
          <div class="text-xs">
            <span class="font-medium text-foreground block">Registrar movimiento en caja</span>
            <span class="text-muted-foreground block">Ingresa como cobranza en el libro de caja</span>
          </div>
        </label>
      </div>

      <template #footer>
        <Button variant="outline" :disabled="guardandoPago" @click="pagosVisible = false">
          {{ $t('General.Cancel') }}
        </Button>
        <Button :disabled="guardandoPago || Number(nuevoPago.monto) <= 0" @click="registrarPago()">
          {{ $t('Facturas.RegistrarPago') }}
        </Button>
      </template>
    </Dialog>
  </section>
</template>
