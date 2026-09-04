<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import Tabs from 'primevue/tabs'
import TabList from 'primevue/tablist'
import Tab from 'primevue/tab'
import TabPanels from 'primevue/tabpanels'
import TabPanel from 'primevue/tabpanel'

import ListState from '@/components/domain/ListState.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import HelpButton from '@/components/ui/HelpButton.vue'
import { Button } from '@/components/ui/button'
import { useApiError, type ApiError } from '@/composables/useApiError'
import { useMovimientosStore, type MovimientoListItem } from '@/stores/useMovimientosStore'
import { useProyectosStore, type ProyectoDetalle } from '@/stores/useProyectosStore'
import { useTrabajosStore, type TrabajoListItem } from '@/stores/useTrabajosStore'
import NuevoTrabajoModal from './components/NuevoTrabajoModal.vue'
import ProyectoGeneralTab from './components/ProyectoGeneralTab.vue'
import ProyectoTrabajosTab from './components/ProyectoTrabajosTab.vue'
import ProyectoCajaTab from './components/ProyectoCajaTab.vue'

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
      tipoMovimientoId: '00000000-0000-0000-0000-000000000002',
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
              <ProyectoGeneralTab
                :proyecto="proyecto"
                :trabajos-count="trabajosItems.length"
                :total-ingresos="totalIngresos"
                :balance-neto="balanceNeto"
              />
            </TabPanel>

            <!-- TAB 2: Works / Jobs -->
            <TabPanel value="trabajos">
              <ProyectoTrabajosTab
                :items="trabajosItems"
                @nuevo="showNuevoTrabajoModal = true"
                @ver-ordenes="verOrdenesDeTrabajo"
                @ver-detalle="router.push({ name: 'trabajo-detalle', params: { trabajoId: $event } })"
              />
            </TabPanel>

            <!-- TAB 3: Cash & Profitability -->
            <TabPanel value="caja">
              <ProyectoCajaTab
                :items="movimientosItems"
                :total-ingresos="totalIngresos"
                :total-gastos="totalGastos"
                :balance-neto="balanceNeto"
                @registrar-movimiento="irARegistrarMovimiento()"
              />
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
