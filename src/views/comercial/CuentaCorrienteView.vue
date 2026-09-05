<script setup lang="ts">
import ToggleSwitch from 'primevue/toggleswitch'
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'

import DateText from '@/components/domain/DateText.vue'
import ListState from '@/components/domain/ListState.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import HelpButton from '@/components/ui/HelpButton.vue'
import { Button } from '@/components/ui/button'
import { useApiError, type ApiError } from '@/composables/useApiError'
import { useCategoriasStore } from '@/stores/useCategoriasStore'
import { useMovimientosStore } from '@/stores/useMovimientosStore'
import { useProyectosStore } from '@/stores/useProyectosStore'
import { useTrabajosStore } from '@/stores/useTrabajosStore'
import { useCatalogStore, type LookupItem } from '@/stores/useCatalogStore'
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
import CuentaCorrienteCobroModal from './components/CuentaCorrienteCobroModal.vue'
import CuentaCorrienteFacturasTable from './components/CuentaCorrienteFacturasTable.vue'

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
const categoriasStore = useCategoriasStore()
const movimientosStore = useMovimientosStore()
const proyectosStore = useProyectosStore()
const trabajosStore = useTrabajosStore()
const catalogStore = useCatalogStore()

const clienteId = computed(() => String(route.params.clienteId ?? ''))

const cuenta = ref<CuentaCorriente | null>(null)
const antiguedad = ref<AntiguedadDeuda | null>(null)
const incluirPagadas = ref(false)
const loading = ref(false)
const firstLoad = ref(true)
const error = ref<ApiError | null>(null)

const anticiposCliente = ref<import('@/stores/useMovimientosStore').MovimientoListItem[]>([])
const modoCobro = ref<'nuevo' | 'anticipo'>('nuevo')
const anticipoSeleccionadoId = ref<string | null>(null)

const totalAnticipos = computed(() => {
  return anticiposCliente.value.reduce((acc, a) => acc + Number(a.total), 0).toFixed(4)
})

const saldoNetoReal = computed(() => {
  if (!cuenta.value) return '0.0000'
  const saldoFacturado = Number(cuenta.value.saldo)
  const anticipos = Number(totalAnticipos.value)
  return (saldoFacturado - anticipos).toFixed(4)
})

async function cargar(): Promise<void> {
  if (!clienteId.value) return
  loading.value = true
  error.value = null
  try {
    const [statement, aging, movsRes] = await Promise.all([
      store.fetchCuentaCorriente({
        clienteId: clienteId.value,
        incluirPagadas: incluirPagadas.value,
      }),
      store.fetchAntiguedad({ clienteId: clienteId.value }),
      movimientosStore.fetchPaged({
        filtro: { clienteId: clienteId.value },
        page: 1,
        pageSize: 0,
      }),
    ])
    cuenta.value = statement
    antiguedad.value = aging
    anticiposCliente.value = movsRes.items.filter((m) => m.esIngreso && !m.facturaId)
  } catch (e) {
    error.value = notify(e)
  } finally {
    loading.value = false
    firstLoad.value = false
  }
}

function onAnticipoChange(): void {
  if (!anticipoSeleccionadoId.value) return
  const anticipo = anticiposCliente.value.find((a) => a.id === anticipoSeleccionadoId.value)
  if (!anticipo || !factura.value) return

  nuevoPago.value.fecha = anticipo.fecha.slice(0, 10)
  const montoAnticipo = Number(anticipo.total)
  const saldoFactura = Number(factura.value.saldo)
  const montoAImputar = Math.min(montoAnticipo, saldoFactura).toFixed(4)
  nuevoPago.value.monto = montoAImputar
  nuevoPago.value.medioPago = 'Transferencia'
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
const pagoProyectoId = ref<string | null>(null)
const pagoTrabajoId = ref<string | null>(null)
const opcionesProyectos = ref<LookupItem[]>([])
const pagoOpcionesTrabajos = ref<LookupItem[]>([])
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

function fechaPagoToIso(fechaStr: string): string {
  if (!fechaStr) return new Date().toISOString()
  const partes = fechaStr.split('-').map(Number)
  if (partes.length === 3 && partes[0] && partes[1] && partes[2]) {
    const now = new Date()
    return new Date(
      partes[0],
      partes[1] - 1,
      partes[2],
      now.getHours(),
      now.getMinutes(),
      now.getSeconds(),
    ).toISOString()
  }
  return new Date().toISOString()
}

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
    pagoProyectoId.value = null
    pagoTrabajoId.value = null
    pagoOpcionesTrabajos.value = []

    try {
      opcionesProyectos.value = await proyectosStore.lookup(factura.value.clienteId || clienteId.value)
    } catch {
      opcionesProyectos.value = []
    }

    const cachedObra = localStorage.getItem(`certaro:factura-obra:${f.id}`)
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

    pagosVisible.value = true
    modoCobro.value = 'nuevo'
    anticipoSeleccionadoId.value = null
  } catch (e) {
    notify(e)
  }
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
  guardandoPago.value = true
  try {
    const pagoMonto = nuevoPago.value.monto
    const pagoMedio = nuevoPago.value.medioPago
    const pagoFecha = nuevoPago.value.fecha
    factura.value = await facturasStore.crearPago(nuevoPago.value)

    if (modoCobro.value === 'anticipo' && anticipoSeleccionadoId.value) {
      try {
        const anticipoItem = anticiposCliente.value.find((a) => a.id === anticipoSeleccionadoId.value)
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
        <dl class="grid grid-cols-1 gap-3 sm:grid-cols-4">
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
              {{ $t('Comercial.CuentaCorriente.Saldo') }} (Facturado)
            </dt>
            <dd class="text-xl font-semibold"><MoneyText :value="cuenta.saldo" colored /></dd>
          </div>
          <div class="rounded-lg border border-border bg-surface-card p-4">
            <dt class="text-sm text-muted-foreground flex items-center justify-between">
              <span>Anticipos sin Imputar</span>
              <span v-if="anticiposCliente.length > 0" class="text-xs bg-primary/10 text-primary px-1.5 py-0.5 rounded font-medium">
                {{ anticiposCliente.length }}
              </span>
            </dt>
            <dd class="text-xl font-semibold text-money-positive">
              <MoneyText :value="totalAnticipos" />
            </dd>
            <p v-if="anticiposCliente.length > 0" class="mt-1 text-xs text-muted-foreground">
              Saldo real neto: <span class="font-medium text-foreground">${{ saldoNetoReal }}</span>
            </p>
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

        <CuentaCorrienteFacturasTable
          :facturas="cuenta.facturas"
          :clase-mora="claseMora"
          @ver-factura="verFactura"
          @abrir-cobro="abrirCobro"
        />
      </div>
    </ListState>

    <CuentaCorrienteCobroModal
      v-model:visible="pagosVisible"
      v-model:registrar-en-caja="registrarEnCaja"
      v-model:pago-proyecto-id="pagoProyectoId"
      v-model:pago-trabajo-id="pagoTrabajoId"
      v-model:modo-cobro="modoCobro"
      v-model:anticipo-seleccionado-id="anticipoSeleccionadoId"
      :anticipos-disponibles="anticiposCliente"
      :factura="factura"
      :nuevo-pago="nuevoPago"
      :pago-errores="pagoErrores"
      :medio-pago-options="medioPagoOptions"
      :guardando-pago="guardandoPago"
      :opciones-proyectos="opcionesProyectos"
      :pago-opciones-trabajos="pagoOpcionesTrabajos"
      @anticipo-change="onAnticipoChange"
      @proyecto-change="onPagoProyectoChange"
      @registrar="registrarPago"
    />
  </section>
</template>
