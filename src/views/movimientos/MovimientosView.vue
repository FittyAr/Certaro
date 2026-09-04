<script setup lang="ts">
import Divider from 'primevue/divider'
import { computed, onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'

import ExportMenu from '@/components/domain/ExportMenu.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import PageHeader from '@/components/domain/PageHeader.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import HelpButton from '@/components/ui/HelpButton.vue'
import { Button } from '@/components/ui/button'
import { useConfirmDelete } from '@/composables/useConfirmDelete'
import { useServerTable } from '@/composables/useServerTable'
import { useShortcuts } from '@/composables/useShortcuts'
import { useCatalogStore, type LookupItem } from '@/stores/useCatalogStore'
import { useClientesStore } from '@/stores/useClientesStore'
import { useEmpleadosStore } from '@/stores/useEmpleadosStore'
import { useMovimientosStore } from '@/stores/useMovimientosStore'
import { useProyectosStore } from '@/stores/useProyectosStore'
import { useReportesStore } from '@/stores/useReportesStore'
import type {
  MovimientoFiltro,
  MovimientoListItem,
  MovimientoResumen,
} from '@/stores/useMovimientosStore'
import MovimientoDrawer from './components/MovimientoDrawer.vue'
import MovimientosFilterBar from './components/MovimientosFilterBar.vue'
import MovimientosTable from './components/MovimientosTable.vue'

/**
 * The cash ledger. See `docs/09-modulos-funcionales.md` §3.2.
 *
 * This is the only screen whose filtering and paging happen on the server: the table grows without
 * bound and cannot be shipped whole to the frontend. The totals under the table describe the whole
 * filter, not the visible page.
 */

const route = useRoute()
const { confirmDelete } = useConfirmDelete()
const store = useMovimientosStore()
const catalog = useCatalogStore()
const reportes = useReportesStore()
const empleados = useEmpleadosStore()
const clientes = useClientesStore()
const proyectos = useProyectosStore()

const drawerRef = ref<InstanceType<typeof MovimientoDrawer> | null>(null)

const table = useServerTable<MovimientoFiltro, MovimientoListItem, MovimientoResumen>({
  key: 'movimientos',
  initialFilter: { concepto: '' },
  fetch: (query) => store.fetchPaged(query),
  defaultSort: { field: 'fecha', dir: 'Desc' },
})

const tipos = ref<LookupItem[]>([])
const categorias = ref<LookupItem[]>([])
const opcionesEmpleado = ref<LookupItem[]>([])
const opcionesCliente = ref<LookupItem[]>([])
const opcionesProyecto = ref<LookupItem[]>([])

async function cargarSelectores(): Promise<void> {
  const [t, c, emp, cli, proy] = await Promise.all([
    catalog.loadTiposMovimiento(),
    catalog.loadCategorias(),
    empleados.fetchLookup(true),
    clientes.lookup(undefined, 200),
    proyectos.lookup(undefined, undefined, 200),
  ])
  tipos.value = t
  categorias.value = c
  opcionesEmpleado.value = emp
  opcionesCliente.value = cli
  opcionesProyecto.value = proy
}

const resumen = computed(() => table.summary.value)

function onDelete(row: MovimientoListItem): void {
  confirmDelete({
    entityKey: 'Entity.Movimiento',
    label: row.concepto,
    action: () => store.remove(row.id, row.rowVersion),
    onDone: () => table.reload(),
  })
}

useShortcuts({ 'ctrl+n': () => drawerRef.value?.openCreate() })

onMounted(async () => {
  if (route.query.filtroProyectoId) {
    table.filter.value.proyectoId = String(route.query.filtroProyectoId)
  }
  if (route.query.filtroClienteId) {
    table.filter.value.clienteId = String(route.query.filtroClienteId)
  }
  table.start()
  await cargarSelectores()
  if (route.query.proyectoId) {
    await drawerRef.value?.openCreate({
      proyectoId: String(route.query.proyectoId),
      clienteId: route.query.clienteId ? String(route.query.clienteId) : undefined,
    })
  }
})
</script>

<template>
  <section class="flex h-full flex-col gap-4 p-6">
    <PageHeader :title="$t('Menu.Movimientos')" :subtitle="$t('Movimientos.Subtitle')">
      <template #actions>
        <!-- Exports the filter, not the page: the count in the tooltip says so. -->
        <ExportMenu
          reporte="movimientos"
          :cantidad="resumen?.cantidad"
          :run="(formato, destino) => reportes.exportMovimientos(table.filter.value, formato, destino)"
        />
        <Button @click="drawerRef?.openCreate()">
          <AppIcon name="plus" :size="16" />
          {{ $t('General.New') }}
        </Button>
        <HelpButton topic-id="movimientos-overview" title="Ayuda sobre el Libro de Movimientos y Caja" />
      </template>
    </PageHeader>

    <MovimientosFilterBar
      :table="table"
      :tipos="tipos"
      :categorias="categorias"
      :opciones-cliente="opcionesCliente"
      :opciones-proyecto="opcionesProyecto"
    />

    <Divider />

    <MovimientosTable
      :table="table"
      @edit="(id) => drawerRef?.openEdit(id)"
      @delete="onDelete"
    />

    <!-- The summary covers every matching row, so it does not change when paging. -->
    <div
      v-if="resumen"
      class="flex flex-wrap items-center gap-6 rounded-md border border-border bg-surface-raised px-4 py-3 text-sm"
    >
      <span class="text-muted-foreground">
        {{ $t('Movimientos.ResumenCantidad', { count: resumen.cantidad }) }}
      </span>
      <span class="flex items-center gap-2">
        <span class="text-muted-foreground">{{ $t('Movimientos.Ingresos') }}</span>
        <MoneyText :value="resumen.totalIngresos" colored />
      </span>
      <span class="flex items-center gap-2">
        <span class="text-muted-foreground">{{ $t('Movimientos.Gastos') }}</span>
        <MoneyText :value="`-${resumen.totalGastos}`" colored />
      </span>
      <span class="flex items-center gap-2 font-medium">
        <span class="text-muted-foreground">{{ $t('Movimientos.Balance') }}</span>
        <MoneyText :value="resumen.balance" colored />
      </span>
    </div>

    <MovimientoDrawer
      ref="drawerRef"
      :tipos="tipos"
      :categorias="categorias"
      :opciones-empleado="opcionesEmpleado"
      :opciones-cliente="opcionesCliente"
      :opciones-proyecto="opcionesProyecto"
      @saved="table.reload()"
    />
  </section>
</template>
