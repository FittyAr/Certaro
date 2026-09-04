<script setup lang="ts">
import Column from 'primevue/column'
import DataTable from 'primevue/datatable'
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import DateText from '@/components/domain/DateText.vue'
import ListState from '@/components/domain/ListState.vue'
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
  type CertificadoListItem,
} from '@/stores/useCertificadosStore'
import {
  useOrdenesTrabajoStore,
  type OrdenTrabajoDetalle,
} from '@/stores/useOrdenesTrabajoStore'

import OrdenEditorModal from './components/OrdenEditorModal.vue'
import OrdenEmisionCertificadoModal from './components/OrdenEmisionCertificadoModal.vue'

const route = useRoute()
const router = useRouter()
const { notify } = useApiError()
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

const editorOpen = ref(false)
const emisionOpen = ref(false)

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
        <Button v-if="orden" variant="outline" @click="editorOpen = true">
          <AppIcon name="pencil" :size="16" />
          {{ $t('General.Edit') }}
        </Button>
        <Button :disabled="!orden" @click="emisionOpen = true">
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
            <Button size="sm" @click="emisionOpen = true">
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

    <!-- Dialog for issuing advance certificates -->
    <OrdenEmisionCertificadoModal
      v-model:visible="emisionOpen"
      :orden-id="ordenId"
    />

    <!-- Dialog for editing the work order directly from its detail view -->
    <OrdenEditorModal
      v-model:visible="editorOpen"
      :orden="orden"
      @saved="cargar"
    />
  </section>
</template>
