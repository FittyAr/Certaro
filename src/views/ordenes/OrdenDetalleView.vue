<script setup lang="ts">
import Column from 'primevue/column'
import DataTable from 'primevue/datatable'
import Dialog from 'primevue/dialog'
import Textarea from 'primevue/textarea'
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import DateInput from '@/components/domain/DateInput.vue'
import DateText from '@/components/domain/DateText.vue'
import DecimalInput from '@/components/domain/DecimalInput.vue'
import FieldError from '@/components/domain/FieldError.vue'
import ListState from '@/components/domain/ListState.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
import PercentBar from '@/components/domain/PercentBar.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import { useApiError, type ApiError } from '@/composables/useApiError'
import { useCertificadosStore, type CertificadoBorradorItem } from '@/stores/useCertificadosStore'
import { useOrdenesTrabajoStore, type OrdenTrabajoDetalle } from '@/stores/useOrdenesTrabajoStore'

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

const ordenId = computed(() => String(route.params.ordenId ?? ''))

const orden = ref<OrdenTrabajoDetalle | null>(null)
const loading = ref(false)
const firstLoad = ref(true)
const error = ref<ApiError | null>(null)

async function cargar(): Promise<void> {
  loading.value = true
  error.value = null
  try {
    orden.value = await store.fetchOne(ordenId.value)
  } catch (e) {
    error.value = notify(e)
  } finally {
    loading.value = false
    firstLoad.value = false
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

const totalAEmitir = computed(() => {
  const draft = borrador.value
  if (!draft) return '0.0000'
  return draft.items.reduce((acc, i) => acc + Number(subtotalDe(i)), 0).toFixed(4)
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
        <Button :disabled="!orden" @click="abrirEmision()">
          <AppIcon name="file-badge" :size="16" />
          {{ $t('Certificados.Emitir') }}
        </Button>
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
            <dt class="text-xs text-muted-foreground">{{ $t('Ordenes.Obra') }}</dt>
            <dd>{{ orden.obraNumero }} · {{ orden.obraNombre }}</dd>
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

        <div class="flex justify-end gap-2 border-t border-border pt-3 text-sm">
          <span class="text-muted-foreground">{{ $t('Certificados.TotalAEmitir') }}</span>
          <MoneyText :value="totalAEmitir" />
        </div>
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
  </section>
</template>
