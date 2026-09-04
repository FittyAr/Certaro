<script setup lang="ts">
import Column from 'primevue/column'
import DataTable from 'primevue/datatable'
import Textarea from 'primevue/textarea'
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import DateText from '@/components/domain/DateText.vue'
import ExportMenu from '@/components/domain/ExportMenu.vue'
import ListState from '@/components/domain/ListState.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
import PercentBar from '@/components/domain/PercentBar.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import HelpButton from '@/components/ui/HelpButton.vue'
import { Button } from '@/components/ui/button'
import { useApiError, type ApiError } from '@/composables/useApiError'
import { useConfirmDelete } from '@/composables/useConfirmDelete'
import { useCertificadosStore, type CertificadoDetalle } from '@/stores/useCertificadosStore'
import { useReportesStore } from '@/stores/useReportesStore'

/**
 * One issued certificate. See `docs/09-modulos-funcionales.md` �3.7.
 *
 * Everything on this screen except the notes is frozen: the quantities, the prices and the
 * percentages are the copies taken when it was issued, so editing the quote afterwards cannot
 * rewrite what was certified.
 */

const route = useRoute()
const router = useRouter()
const { notify } = useApiError()
const { confirmDelete } = useConfirmDelete()
const store = useCertificadosStore()
const reportes = useReportesStore()

const certificadoId = computed(() => String(route.params.certificadoId ?? ''))

const certificado = ref<CertificadoDetalle | null>(null)
const loading = ref(false)
const firstLoad = ref(true)
const error = ref<ApiError | null>(null)
const observaciones = ref<string | null>(null)
const guardando = ref(false)

async function cargar(): Promise<void> {
  loading.value = true
  error.value = null
  try {
    certificado.value = await store.fetchOne(certificadoId.value)
  } catch (e) {
    error.value = notify(e)
  } finally {
    loading.value = false
    firstLoad.value = false
  }
}

watch(certificado, (value) => {
  observaciones.value = value?.observaciones ?? null
})

const observacionesCambiaron = computed(
  () => (observaciones.value ?? '') !== (certificado.value?.observaciones ?? ''),
)

async function guardarObservaciones(): Promise<void> {
  if (!certificado.value || guardando.value) return
  guardando.value = true
  try {
    certificado.value = await store.updateObservaciones(
      certificado.value.id,
      observaciones.value?.trim() ? observaciones.value : null,
      certificado.value.audit.rowVersion,
    )
  } catch (e) {
    notify(e)
  } finally {
    guardando.value = false
  }
}

function anular(): void {
  const actual = certificado.value
  if (!actual) return
  confirmDelete({
    entityKey: 'Entity.Certificado',
    label: String(actual.numero),
    action: () => store.remove(actual.id, actual.audit.rowVersion),
    onDone: () => router.push({ name: 'certificados' }),
  })
}

function verOrden(): void {
  if (!certificado.value) return
  void router.push({
    name: 'orden-detalle',
    params: { ordenId: certificado.value.ordenTrabajoId },
  })
}

const facturado = ref(false)

function verificarFacturado(id: string): void {
  try {
    facturado.value = Boolean(localStorage.getItem(`certaro:cert-facturado:${id}`))
  } catch {
    facturado.value = false
  }
}

function facturarCertificado(): void {
  if (!certificado.value) return
  if (facturado.value) {
    const continuar = window.confirm(
      'Este certificado de avance ya fue enviado a facturar previamente. ¿Deseas emitir otra factura para este mismo certificado?',
    )
    if (!continuar) return
  }

  try {
    localStorage.setItem(
      `certaro:cert-facturado:${certificado.value.id}`,
      JSON.stringify({
        fecha: new Date().toISOString(),
        numero: certificado.value.numero,
        total: certificado.value.totalNeto,
      }),
    )
    facturado.value = true
  } catch {
    // ignore
  }

  void router.push({
    path: '/facturas',
    query: {
      certificadoId: certificado.value.id,
      clienteId: certificado.value.clienteId,
      proyectoId: certificado.value.proyectoId,
      trabajoId: certificado.value.trabajoId,
      subtotal: certificado.value.totalNeto,
      iva: '0.0000',
      total: certificado.value.totalNeto,
      observaciones: `Certificado N.º ${certificado.value.numero} · ${certificado.value.ordenTitulo}`,
    },
  })
}

function onExportarCertificado(destino: string) {
  if (!certificado.value) return Promise.reject(new Error('No hay certificado'))
  return reportes.exportCertificado(certificado.value.id, destino)
}

onMounted(async () => {
  await cargar()
  if (certificadoId.value) {
    verificarFacturado(certificadoId.value)
  }
})
</script>

<template>
  <section class="flex h-full flex-col gap-4 p-6">
    <PageHeader
      :title="
        certificado
          ? $t('Certificados.DetalleTitulo', { numero: certificado.numero })
          : $t('Menu.Certificados')
      "
      :subtitle="certificado?.ordenTitulo"
    >
      <template #actions>
        <span
          v-if="facturado"
          class="inline-flex items-center gap-1.5 rounded-md border border-success/40 bg-success/15 px-2.5 py-1 text-xs font-semibold text-success"
        >
          <AppIcon name="receipt" :size="14" />
          Facturado
        </span>
        <Button variant="outline" @click="router.back()">
          <AppIcon name="arrow-left" :size="16" />
          {{ $t('General.Back') }}
        </Button>
        <ExportMenu
          v-if="certificado"
          reporte="certificado"
          :formatos="['Pdf']"
          :detalle="`${certificado.proyectoNombre} - #${certificado.numero}`"
          :run="(_, destino) => onExportarCertificado(destino)"
        />
        <Button
          v-if="certificado"
          variant="secondary"
          @click="facturarCertificado()"
        >
          <AppIcon name="receipt" :size="16" />
          {{ $t('Certificados.Facturar') }}
        </Button>
        <Button variant="outline" :disabled="!certificado" @click="verOrden()">
          <AppIcon name="file-text" :size="16" />
          {{ $t('Certificados.VerOrden') }}
        </Button>
        <Button v-if="certificado?.esUltimo" variant="destructive" @click="anular()">
          <AppIcon name="trash-2" :size="16" />
          {{ $t('Certificados.Anular') }}
        </Button>
        <HelpButton topic-id="certificados-detalle" title="Ayuda sobre Detalle del Certificado" />
      </template>
    </PageHeader>

    <ListState
      :loading="loading"
      :first-load="firstLoad"
      :error="error"
      :is-empty="!certificado"
      :is-filtered="false"
      empty-key="Certificados.Empty"
      class="flex-1 overflow-auto"
      @retry="cargar()"
    >
      <div v-if="certificado" class="space-y-4">
        <dl
          class="grid grid-cols-2 gap-3 rounded-md border border-border p-4 text-sm md:grid-cols-4"
        >
          <div>
            <dt class="text-xs text-muted-foreground">{{ $t('Certificados.Proyecto') }}</dt>
            <dd>{{ certificado.proyectoNumero }} � {{ certificado.proyectoNombre }}</dd>
          </div>
          <div>
            <dt class="text-xs text-muted-foreground">{{ $t('Certificados.Cliente') }}</dt>
            <dd>{{ certificado.clienteNombre }}</dd>
          </div>
          <div>
            <dt class="text-xs text-muted-foreground">{{ $t('Certificados.Trabajo') }}</dt>
            <dd>{{ certificado.trabajoDescripcion }}</dd>
          </div>
          <div>
            <dt class="text-xs text-muted-foreground">{{ $t('Certificados.Fecha') }}</dt>
            <dd><DateText :value="certificado.fecha" /></dd>
          </div>
        </dl>

        <!-- The nine columns of the sheet, per `docs/12-reportes-y-exportaciones.md` �4. -->
        <DataTable :value="certificado.items" data-key="id" size="small" class="text-sm">
          <Column field="descripcion" :header="$t('Certificados.Descripcion')" />
          <Column field="unidad" :header="$t('Certificados.Unidad')" />
          <Column field="cantidad" :header="$t('Certificados.Cantidad')" />
          <Column field="precioUnitario" :header="$t('Certificados.PrecioUnitario')">
            <template #body="{ data }"><MoneyText :value="data.precioUnitario" /></template>
          </Column>
          <Column field="porcentajeAnterior" :header="$t('Certificados.AcumuladoAnterior')">
            <template #body="{ data }"><PercentBar :value="data.porcentajeAnterior" /></template>
          </Column>
          <Column field="porcentajeActual" :header="$t('Certificados.AvanceCertificado')">
            <template #body="{ data }"><PercentBar :value="data.porcentajeActual" /></template>
          </Column>
          <Column field="porcentajeAcumulado" :header="$t('Certificados.Acumulado')">
            <template #body="{ data }"><PercentBar :value="data.porcentajeAcumulado" /></template>
          </Column>
          <Column field="subtotalActual" :header="$t('Certificados.Subtotal')">
            <template #body="{ data }"><MoneyText :value="data.subtotalActual" /></template>
          </Column>
          <Column field="subtotalAcumulado" :header="$t('Certificados.SubtotalAcumulado')">
            <template #body="{ data }"><MoneyText :value="data.subtotalAcumulado" /></template>
          </Column>
        </DataTable>

        <div class="flex flex-col gap-4 md:flex-row">
          <label class="flex flex-1 flex-col gap-1">
            <span class="text-sm">{{ $t('Certificados.Observaciones') }}</span>
            <Textarea v-model="observaciones" rows="3" auto-resize />
            <Button
              class="self-start"
              variant="outline"
              size="sm"
              :disabled="!observacionesCambiaron || guardando"
              @click="guardarObservaciones()"
            >
              {{ $t('General.Save') }}
            </Button>
          </label>

          <dl class="w-full max-w-sm space-y-1 rounded-md border border-border p-4 text-sm">
            <div class="flex justify-between">
              <dt>{{ $t('Certificados.TotalCertificado') }}</dt>
              <dd><MoneyText :value="certificado.totalCertificado" /></dd>
            </div>
            <div class="flex justify-between text-muted-foreground">
              <dt>{{ $t('Certificados.AjusteUocra') }}</dt>
              <dd>- <MoneyText :value="certificado.ajusteUocra" /></dd>
            </div>
            <div class="flex justify-between text-muted-foreground">
              <dt>{{ $t('Certificados.OtrosDescuentos') }}</dt>
              <dd>- <MoneyText :value="certificado.otrosDescuentos" /></dd>
            </div>
            <div class="flex justify-between border-t border-border pt-1 font-semibold">
              <dt>{{ $t('Certificados.TotalNeto') }}</dt>
              <dd><MoneyText :value="certificado.totalNeto" /></dd>
            </div>
          </dl>
        </div>
      </div>
    </ListState>
  </section>
</template>
