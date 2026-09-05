<script setup lang="ts">
import Column from 'primevue/column'
import DataTable from 'primevue/datatable'
import Divider from 'primevue/divider'
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import DateText from '@/components/domain/DateText.vue'
import ListState from '@/components/domain/ListState.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
import StatePill from '@/components/domain/StatePill.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import HelpButton from '@/components/ui/HelpButton.vue'
import { Button } from '@/components/ui/button'
import { useApiError, type ApiError } from '@/composables/useApiError'
import {
  useOrdenesTrabajoStore,
  type OrdenTrabajoListItem,
} from '@/stores/useOrdenesTrabajoStore'
import { useTrabajosStore, type TrabajoDetalle } from '@/stores/useTrabajosStore'
import OrdenFormModal from '@/views/ordenes/components/OrdenFormModal.vue'

const route = useRoute()
const router = useRouter()
const { notify } = useApiError()
const store = useTrabajosStore()
const ordenesStore = useOrdenesTrabajoStore()

const trabajoId = computed(() => String(route.params.trabajoId ?? ''))
const trabajo = ref<TrabajoDetalle | null>(null)
const ordenes = ref<OrdenTrabajoListItem[]>([])
const loading = ref(false)
const firstLoad = ref(true)
const error = ref<ApiError | null>(null)
const nuevaOrdenModal = ref(false)

async function cargar(): Promise<void> {
  loading.value = true
  error.value = null
  try {
    const [t, ords] = await Promise.all([
      store.fetchOne(trabajoId.value),
      ordenesStore.fetchDeTrabajo(trabajoId.value).catch(() => []),
    ])
    trabajo.value = t
    ordenes.value = ords
  } catch (e) {
    error.value = notify(e)
  } finally {
    loading.value = false
    firstLoad.value = false
  }
}

function abrirOrden(ordenId: string): void {
  void router.push({ name: 'orden-detalle', params: { ordenId } })
}

function irACrearOrden(): void {
  nuevaOrdenModal.value = true
}

onMounted(cargar)
</script>

<template>
  <section class="flex h-full flex-col gap-4 p-6">
    <PageHeader
      :title="trabajo?.descripcion ?? $t('Menu.Trabajos')"
      :subtitle="trabajo?.proyectoNombre"
    >
      <template #actions>
        <Button variant="outline" @click="router.back()">
          <AppIcon name="arrow-left" :size="16" />{{ $t('General.Back') }}
        </Button>
        <Button
          v-if="trabajo"
          @click="router.push({ name: 'trabajo-ordenes', params: { trabajoId: trabajo.id } })"
        >
          <AppIcon name="list" :size="16" />
          {{ $t('Ordenes.Title') }}
        </Button>
        <HelpButton topic-id="trabajos-detalle" title="Ayuda sobre Detalle de Trabajo" />
      </template>
    </PageHeader>

    <Divider />

    <ListState
      :loading="loading"
      :first-load="firstLoad"
      :error="error"
      :is-empty="!trabajo"
      :is-filtered="false"
      empty-key="Trabajos.Empty"
      class="flex-1"
      @retry="cargar()"
    >
      <div v-if="trabajo" class="space-y-6">
        <div class="grid gap-4 rounded-md border border-border p-4 text-sm md:grid-cols-4">
          <div>
            <span class="text-xs text-muted-foreground">{{ $t('Proyectos.Nombre') }}</span>
            <p class="font-medium text-foreground">{{ trabajo.proyectoNombre }}</p>
          </div>
          <div>
            <span class="text-xs text-muted-foreground">{{ $t('Clientes.Nombre') }}</span>
            <p class="font-medium text-foreground">{{ trabajo.clienteNombre }}</p>
          </div>
          <div>
            <span class="text-xs text-muted-foreground">{{ $t('Trabajos.Estado') }}</span>
            <p><StatePill entity="Trabajo" :value="trabajo.estado.actual" /></p>
          </div>
          <div>
            <span class="text-xs text-muted-foreground">{{ $t('Trabajos.Presupuesto') }}</span>
            <p class="font-medium text-foreground"><MoneyText :value="trabajo.presupuesto" /></p>
          </div>
        </div>

        <div class="space-y-3">
          <div class="flex items-center justify-between">
            <h3 class="text-base font-semibold">{{ $t('Ordenes.Title') }}</h3>
            <Button size="sm" variant="outline" @click="irACrearOrden()">
              <AppIcon name="plus" :size="14" />
              {{ $t('General.New') }}
            </Button>
          </div>

          <div v-if="ordenes.length === 0" class="rounded-md border border-dashed border-border p-6 text-center text-sm text-muted-foreground">
            {{ $t('Ordenes.Empty') }}
          </div>

          <DataTable
            v-else
            :value="ordenes"
            data-key="id"
            size="small"
            class="text-sm border border-border rounded-md overflow-hidden"
            @row-dblclick="abrirOrden(($event.data as OrdenTrabajoListItem).id)"
          >
            <Column field="fecha" :header="$t('Ordenes.Fecha')">
              <template #body="{ data }"><DateText :value="data.fecha" /></template>
            </Column>
            <Column field="titulo" :header="$t('Ordenes.Titulo')" />
            <Column field="itemsCount" :header="$t('Ordenes.Items')" />
            <Column field="totalPresupuestado" :header="$t('Ordenes.TotalPresupuestado')">
              <template #body="{ data }"><MoneyText :value="data.totalPresupuestado" /></template>
            </Column>
            <Column field="totalCertificado" :header="$t('Ordenes.Certificado')">
              <template #body="{ data }"><MoneyText :value="data.totalCertificado" /></template>
            </Column>
            <Column field="certificadosCount" :header="$t('Ordenes.Certificados')" />
            <Column :header="$t('General.Actions')" :style="{ width: '6rem' }">
              <template #body="{ data }">
                <Button
                  variant="ghost"
                  size="sm"
                  :title="$t('Ordenes.VerDetalle')"
                  @click="abrirOrden(data.id)"
                >
                  <AppIcon name="eye" :size="14" />
                </Button>
              </template>
            </Column>
          </DataTable>
        </div>
      </div>
    </ListState>
    <OrdenFormModal
      v-model:visible="nuevaOrdenModal"
      :trabajo-id="trabajoId"
      :orden-id="null"
      @saved="cargar()"
    />
  </section>
</template>
