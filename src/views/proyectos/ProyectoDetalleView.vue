<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import Column from 'primevue/column'
import DataTable from 'primevue/datatable'
import Tabs from 'primevue/tabs'
import TabList from 'primevue/tablist'
import Tab from 'primevue/tab'
import TabPanels from 'primevue/tabpanels'
import TabPanel from 'primevue/tabpanel'

import DateText from '@/components/domain/DateText.vue'
import ListState from '@/components/domain/ListState.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
import StatePill from '@/components/domain/StatePill.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import HelpButton from '@/components/ui/HelpButton.vue'
import { Button } from '@/components/ui/button'
import { useApiError, type ApiError } from '@/composables/useApiError'
import { useMovimientosStore, type MovimientoListItem } from '@/stores/useMovimientosStore'
import { useProyectosStore, type ProyectoDetalle } from '@/stores/useProyectosStore'
import { useTrabajosStore, type TrabajoListItem } from '@/stores/useTrabajosStore'
import NuevoTrabajoModal from './components/NuevoTrabajoModal.vue'

/**
 * Project Detail Hub: Integrated workspace for a single site / project.
 * Consolidates general information, works/jobs breakdown, and site cash ledger in tabs.
 */

const route = useRoute()
const router = useRouter()
const { notify } = useApiError()
const store = useProyectosStore()
const trabajosStore = useTrabajosStore()
const movimientosStore = useMovimientosStore()

const proyectoId = computed(() => String(route.params.proyectoId ?? ''))
const activeTab = ref('general')

const proyecto = ref<ProyectoDetalle | null>(null)
const loading = ref(false)
const firstLoad = ref(true)
const error = ref<ApiError | null>(null)

// Works (Trabajos) tab state
const trabajosItems = ref<TrabajoListItem[]>([])
const loadingTrabajos = ref(false)
const showNuevoTrabajoModal = ref(false)

// Cash ledger tab state
const movimientosItems = ref<MovimientoListItem[]>([])
const loadingMovimientos = ref(false)
const resumenMovimientos = ref<{ totalIngresos: string; totalGastos: string; balance: string } | null>(null)

async function cargarGeneral(): Promise<void> {
  loading.value = true
  error.value = null
  try {
    proyecto.value = await store.fetchOne(proyectoId.value)
  } catch (e) {
    error.value = notify(e)
  } finally {
    loading.value = false
    firstLoad.value = false
  }
}

async function cargarTrabajos(): Promise<void> {
  loadingTrabajos.value = true
  try {
    const res = await trabajosStore.fetchPaged({
      page: 1,
      pageSize: 100,
      filtro: { proyectoId: proyectoId.value } as unknown as Record<string, unknown>,
      sortDir: 'Desc',
    })
    trabajosItems.value = res.items
  } catch (e) {
    notify(e)
  } finally {
    loadingTrabajos.value = false
  }
}

async function cargarMovimientos(): Promise<void> {
  loadingMovimientos.value = true
  try {
    const res = await movimientosStore.fetchPaged({
      page: 1,
      pageSize: 100,
      filtro: { proyectoId: proyectoId.value },
      sortDir: 'Desc',
    })
    movimientosItems.value = res.items
    if (res.resumen) {
      resumenMovimientos.value = {
        totalIngresos: res.resumen.totalIngresos,
        totalGastos: res.resumen.totalGastos,
        balance: res.resumen.balance,
      }
    }
  } catch (e) {
    notify(e)
  } finally {
    loadingMovimientos.value = false
  }
}

async function cargarTodo(): Promise<void> {
  await cargarGeneral()
  if (proyecto.value) {
    void cargarTrabajos()
    void cargarMovimientos()
  }
}

const totalIngresos = computed(() =>
  resumenMovimientos.value
    ? resumenMovimientos.value.totalIngresos
    : movimientosItems.value
        .filter((m) => m.esIngreso)
        .reduce((acc, m) => acc + Number(m.total), 0)
        .toFixed(4),
)

const totalGastos = computed(() =>
  resumenMovimientos.value
    ? resumenMovimientos.value.totalGastos
    : movimientosItems.value
        .filter((m) => !m.esIngreso)
        .reduce((acc, m) => acc + Number(m.total), 0)
        .toFixed(4),
)

const balanceNeto = computed(() =>
  resumenMovimientos.value
    ? resumenMovimientos.value.balance
    : (Number(totalIngresos.value) - Number(totalGastos.value)).toFixed(4),
)

function irARegistrarMovimiento(): void {
  void router.push({
    path: '/movimientos',
    query: {
      proyectoId: proyectoId.value,
      clienteId: proyecto.value?.clienteId ?? undefined,
    },
  })
}

function verOrdenesDeTrabajo(trabajoId: string): void {
  void router.push({ name: 'trabajo-ordenes', params: { trabajoId } })
}

function onTrabajoGuardado(): void {
  showNuevoTrabajoModal.value = false
  void cargarTrabajos()
  void cargarGeneral()
}

onMounted(cargarTodo)
</script>

<template>
  <section class="flex h-full flex-col gap-4 p-6">
    <PageHeader
      :title="proyecto ? `${proyecto.nombre} · #${proyecto.numero}` : $t('Menu.Proyectos')"
      :subtitle="proyecto?.clienteNombre"
    >
      <template #actions>
        <Button variant="outline" @click="router.back()">
          <AppIcon name="arrow-left" :size="16" />{{ $t('General.Back') }}
        </Button>
        <Button
          v-if="proyecto"
          variant="outline"
          @click="irARegistrarMovimiento()"
        >
          <AppIcon name="plus" :size="16" />
          {{ $t('Movimientos.RegistrarGasto') }}
        </Button>
        <Button
          v-if="proyecto"
          variant="outline"
          @click="router.push({ path: '/kanban', query: { proyectoId: proyecto.id } })"
        >
          <AppIcon name="kanban" :size="16" />
          {{ $t('Proyectos.VerKanban') }}
        </Button>
        <HelpButton topic-id="proyectos-detalle" title="Ayuda sobre la Ficha Integral de Obra" />
      </template>
    </PageHeader>

    <ListState
      :loading="loading"
      :first-load="firstLoad"
      :error="error"
      :is-empty="!proyecto"
      :is-filtered="false"
      empty-key="Proyectos.Empty"
      class="flex-1 overflow-auto"
      @retry="cargarTodo()"
    >
      <div v-if="proyecto" class="flex flex-col gap-4">
        <!-- Tabs Navigation -->
        <Tabs v-model:value="activeTab" class="w-full">
          <TabList class="border-b border-border">
            <Tab value="general" class="flex items-center gap-2">
              <AppIcon name="building-2" :size="16" />
              <span>Información General</span>
            </Tab>
            <Tab value="trabajos" class="flex items-center gap-2">
              <AppIcon name="hammer" :size="16" />
              <span>{{ $t('Menu.Trabajos') }}</span>
              <span class="ml-1 rounded-full bg-muted px-2 py-0.5 text-xs">
                {{ trabajosItems.length }}
              </span>
            </Tab>
            <Tab value="caja" class="flex items-center gap-2">
              <AppIcon name="wallet" :size="16" />
              <span>Caja y Rentabilidad</span>
              <span class="ml-1 rounded-full bg-muted px-2 py-0.5 text-xs">
                {{ movimientosItems.length }}
              </span>
            </Tab>
          </TabList>

          <TabPanels class="bg-transparent px-0 py-4">
            <!-- TAB 1: General Info -->
            <TabPanel value="general">
              <div class="space-y-4">
                <div class="grid gap-4 rounded-lg border border-border bg-surface-card p-5 text-sm md:grid-cols-2">
                  <div>
                    <span class="text-xs text-muted-foreground">{{ $t('Proyectos.Numero') }}</span>
                    <p class="font-semibold text-foreground">#{{ proyecto.numero }}</p>
                  </div>
                  <div>
                    <span class="text-xs text-muted-foreground">{{ $t('Proyectos.Estado') }}</span>
                    <p class="mt-0.5"><StatePill entity="Proyecto" :value="proyecto.estado.actual" /></p>
                  </div>
                  <div>
                    <span class="text-xs text-muted-foreground">{{ $t('Clientes.Nombre') }}</span>
                    <p class="font-medium text-foreground">{{ proyecto.clienteNombre }}</p>
                  </div>
                  <div>
                    <span class="text-xs text-muted-foreground">{{ $t('Proyectos.Localidad') }}</span>
                    <p>{{ proyecto.localidad ?? '—' }}</p>
                  </div>
                  <div class="md:col-span-2">
                    <span class="text-xs text-muted-foreground">{{ $t('Clientes.Direccion') }}</span>
                    <p>{{ proyecto.direccion ?? '—' }}</p>
                  </div>
                </div>

                <!-- Resumen rápido de métricas -->
                <div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
                  <div class="rounded-lg border border-border bg-card p-4 shadow-xs">
                    <span class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                      {{ $t('Menu.Trabajos') }}
                    </span>
                    <div class="mt-1 text-2xl font-bold">{{ trabajosItems.length }}</div>
                  </div>
                  <div class="rounded-lg border border-border bg-card p-4 shadow-xs">
                    <span class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                      {{ $t('Movimientos.Ingresos') }}
                    </span>
                    <div class="mt-1 text-2xl font-bold">
                      <MoneyText :value="totalIngresos" colored />
                    </div>
                  </div>
                  <div class="rounded-lg border border-border bg-card p-4 shadow-xs">
                    <span class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                      {{ $t('Movimientos.Balance') }}
                    </span>
                    <div class="mt-1 text-2xl font-bold">
                      <MoneyText :value="balanceNeto" colored show-sign />
                    </div>
                  </div>
                </div>
              </div>
            </TabPanel>

            <!-- TAB 2: Works / Jobs -->
            <TabPanel value="trabajos">
              <div class="space-y-3">
                <div class="flex items-center justify-between">
                  <span class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                    Trabajos y Cómputo de la Obra
                  </span>
                  <Button size="sm" @click="showNuevoTrabajoModal = true">
                    <AppIcon name="plus" :size="14" />
                    {{ $t('General.New') }} {{ $t('Entity.Trabajo') }}
                  </Button>
                </div>

                <div v-if="trabajosItems.length === 0" class="rounded-lg border border-dashed border-border p-8 text-center text-sm text-muted-foreground">
                  {{ $t('Trabajos.Empty') }}
                </div>

                <DataTable
                  v-else
                  :value="trabajosItems"
                  data-key="id"
                  size="small"
                  class="text-sm"
                >
                  <Column field="descripcion" :header="$t('Trabajos.Descripcion')">
                    <template #body="{ data }">
                      <span class="font-medium text-foreground">{{ data.descripcion }}</span>
                    </template>
                  </Column>
                  <Column field="estado" :header="$t('Trabajos.Estado')">
                    <template #body="{ data }">
                      <StatePill entity="Trabajo" :value="data.estado" />
                    </template>
                  </Column>
                  <Column field="presupuesto" :header="$t('Trabajos.Presupuesto')">
                    <template #body="{ data }">
                      <MoneyText :value="data.presupuesto" />
                    </template>
                  </Column>
                  <Column :header="$t('General.Actions')" class="w-40 text-right">
                    <template #body="{ data }">
                      <div class="flex items-center justify-end gap-1">
                        <Button
                          variant="ghost"
                          size="sm"
                          :title="$t('Ordenes.Title')"
                          @click="verOrdenesDeTrabajo(data.id)"
                        >
                          <AppIcon name="file-text" :size="14" />
                          <span class="ml-1 text-xs">{{ $t('Ordenes.Title') }}</span>
                        </Button>
                        <Button
                          variant="ghost"
                          size="sm"
                          :title="$t('General.View')"
                          @click="router.push({ name: 'trabajo-detalle', params: { trabajoId: data.id } })"
                        >
                          <AppIcon name="eye" :size="14" />
                        </Button>
                      </div>
                    </template>
                  </Column>
                </DataTable>
              </div>
            </TabPanel>

            <!-- TAB 3: Cash & Profitability -->
            <TabPanel value="caja">
              <div class="space-y-4">
                <!-- Financial KPI Cards -->
                <div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
                  <div class="rounded-lg border border-border bg-card p-4 shadow-xs">
                    <span class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                      {{ $t('Movimientos.Ingresos') }}
                    </span>
                    <div class="mt-1 text-xl font-bold">
                      <MoneyText :value="totalIngresos" colored />
                    </div>
                  </div>
                  <div class="rounded-lg border border-border bg-card p-4 shadow-xs">
                    <span class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                      {{ $t('Movimientos.Gastos') }}
                    </span>
                    <div class="mt-1 text-xl font-bold">
                      <MoneyText :value="`-${totalGastos}`" colored />
                    </div>
                  </div>
                  <div class="rounded-lg border border-border bg-card p-4 shadow-xs">
                    <span class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                      {{ $t('Movimientos.Balance') }}
                    </span>
                    <div class="mt-1 text-xl font-bold">
                      <MoneyText :value="balanceNeto" colored show-sign />
                    </div>
                  </div>
                </div>

                <div class="flex items-center justify-between">
                  <span class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                    Movimientos Imputados
                  </span>
                  <Button size="sm" @click="irARegistrarMovimiento()">
                    <AppIcon name="plus" :size="14" />
                    {{ $t('Movimientos.RegistrarGasto') }}
                  </Button>
                </div>

                <div v-if="movimientosItems.length === 0" class="rounded-lg border border-dashed border-border p-8 text-center text-sm text-muted-foreground">
                  {{ $t('Movimientos.Empty') }}
                </div>

                <DataTable
                  v-else
                  :value="movimientosItems"
                  data-key="id"
                  size="small"
                  class="text-sm"
                  paginator
                  :rows="20"
                >
                  <Column field="fecha" :header="$t('Movimientos.Fecha')">
                    <template #body="{ data }">
                      <DateText :value="data.fecha" />
                    </template>
                  </Column>
                  <Column field="concepto" :header="$t('Movimientos.Concepto')" />
                  <Column field="tipoMovimientoNombre" :header="$t('Movimientos.Tipo')">
                    <template #body="{ data }">
                      <span
                        class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium"
                        :class="
                          data.esIngreso
                            ? 'bg-success/10 text-success'
                            : 'bg-destructive/10 text-destructive'
                        "
                      >
                        {{ data.tipoMovimientoNombre }}
                      </span>
                    </template>
                  </Column>
                  <Column field="total" :header="$t('Movimientos.Total')">
                    <template #body="{ data }">
                      <MoneyText
                        :value="data.esIngreso ? data.total : `-${data.total}`"
                        colored
                      />
                    </template>
                  </Column>
                </DataTable>
              </div>
            </TabPanel>
          </TabPanels>
        </Tabs>
      </div>
    </ListState>

    <!-- Modal para agregar trabajo al proyecto -->
    <NuevoTrabajoModal
      :show="showNuevoTrabajoModal"
      :proyecto-id="proyecto?.id ?? null"
      :proyectos="proyecto ? [{ id: proyecto.id, label: `${proyecto.numero} · ${proyecto.nombre}` }] : []"
      @close="showNuevoTrabajoModal = false"
      @saved="onTrabajoGuardado"
    />
  </section>
</template>
