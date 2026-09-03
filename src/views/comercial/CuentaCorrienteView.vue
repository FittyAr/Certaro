<script setup lang="ts">
import Column from 'primevue/column'
import DataTable from 'primevue/datatable'
import ToggleSwitch from 'primevue/toggleswitch'
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import DateText from '@/components/domain/DateText.vue'
import ListState from '@/components/domain/ListState.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
import StatePill from '@/components/domain/StatePill.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import { useApiError, type ApiError } from '@/composables/useApiError'
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

const route = useRoute()
const router = useRouter()
const { notify } = useApiError()
const store = useComercialStore()

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
        </DataTable>
      </div>
    </ListState>
  </section>
</template>
