<script setup lang="ts" generic="TFilter extends object">
import Column from 'primevue/column'
import ContextMenu from 'primevue/contextmenu'
import Paginator, { type PageState } from 'primevue/paginator'
import TreeTable from 'primevue/treetable'
import type { TreeNode } from 'primevue/treenode'
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import { PAGE_SIZES } from '@/api/types'
import ListState from '@/components/domain/ListState.vue'
import MoneyText from '@/components/domain/MoneyText.vue'
import StatePill from '@/components/domain/StatePill.vue'
import AppIcon from '@/components/ui/AppIcon.vue'
import { Button } from '@/components/ui/button'
import { useApiError } from '@/composables/useApiError'
import type { useServerTable } from '@/composables/useServerTable'
import type { TrabajoListItem } from '@/stores/useTrabajosStore'
import { useSistemaStore } from '@/stores/useSistemaStore'
import type { ProyectoListItem } from '@/stores/useProyectosStore'

import { useProyectosContextMenu } from '@/components/domain/useProyectosContextMenu'
import { useProyectosTreeNodes } from '@/components/domain/useProyectosTreeNodes'

const props = defineProps<{
  table: ReturnType<typeof useServerTable<TFilter, ProyectoListItem>>
}>()

const emit = defineEmits<{
  proyectoEdit: [row: ProyectoListItem]
  proyectoDelete: [row: ProyectoListItem]
  proyectoTransition: [row: ProyectoListItem, estado: string]
  proyectoCreateTrabajo: [row: ProyectoListItem]
  trabajoNavigate: [row: TrabajoListItem]
}>()

const { t } = useI18n()
const router = useRouter()
const { notify } = useApiError()
const sistemaStore = useSistemaStore()

const contextMenu = ref<InstanceType<typeof ContextMenu> | null>(null)
const contextNode = ref<TreeNode | null>(null)

const {
  expandedKeys,
  treeValue,
  handleExpand,
  handleCollapse,
  pruneRemoved,
} = useProyectosTreeNodes(props.table.rows, notify)

const rowsPerPage = computed(() =>
  props.table.pageSize.value === 0
    ? Math.max(props.table.total.value, 1)
    : props.table.pageSize.value,
)
const pageSizeOptions = PAGE_SIZES.filter((size) => size !== 0)
const pageReportTemplate = computed(() =>
  t('General.PageReport', { first: '{first}', last: '{last}', totalRecords: '{totalRecords}' }),
)

// Clear cache when table reloads (page change, filter change)
watch(
  () => props.table.rows.value.map((r) => r.id).join(','),
  () => {
    const validIds = new Set(props.table.rows.value.map((r) => r.id))
    pruneRemoved(validIds)
  },
)

function onRowNavigate(node: TreeNode): void {
  if (node.data.isProyecto && node.data.proyecto) {
    void router.push({ name: 'proyecto-detalle', params: { proyectoId: node.data.proyecto.id } })
  } else if (node.data.isTrabajo && node.data.trabajo) {
    void router.push({ name: 'trabajo-detalle', params: { trabajoId: node.data.trabajo.id } })
  }
}

const contextMenuModel = useProyectosContextMenu(contextNode, {
  onProyectoEdit: (row) => emit('proyectoEdit', row),
  onProyectoDelete: (row) => emit('proyectoDelete', row),
  onProyectoTransition: (row, estado) => emit('proyectoTransition', row, estado),
  onProyectoCreateTrabajo: (row) => emit('proyectoCreateTrabajo', row),
  onTrabajoNavigate: (row) => emit('trabajoNavigate', row),
})

function onRowContextMenu(event: { originalEvent: Event; node: TreeNode }): void {
  contextNode.value = event.node
  contextMenu.value?.show(event.originalEvent as MouseEvent)
}

function onSort(event: { sortField: unknown; sortOrder: unknown }): void {
  props.table.onSort({
    sortField: typeof event.sortField === 'string' ? event.sortField : null,
    sortOrder: (event.sortOrder as number) ?? null,
  })
}

function onPage(event: PageState): void {
  props.table.onPage({ page: event.page, rows: event.rows })
}
</script>

<template>
  <div class="flex w-full flex-col">
    <ListState
      :loading="table.loading.value"
      :first-load="table.firstLoad.value"
      :error="table.error.value"
      :is-empty="table.isEmpty.value"
      :is-filtered="table.isFiltered.value"
      empty-key="Proyectos.Empty"
      @retry="table.reload()"
      @clear-filters="table.resetFilter()"
    >
      <template #empty-action><slot name="empty-action" /></template>

      <div :class="table.loading.value ? 'opacity-60 transition-opacity' : ''">
        <TreeTable
          :value="treeValue"
          :expanded-keys="expandedKeys"
          data-key="key"
          :sort-field="table.sort.value?.field ?? undefined"
          :sort-order="table.sort.value?.dir === 'Desc' ? -1 : 1"
          removable-sort
          scrollable
          scroll-height="flex"
          size="small"
          class="text-sm"
          @sort="onSort"
          @node-expand="handleExpand"
          @node-collapse="handleCollapse"
          @row-contextmenu="onRowContextMenu"
        >
          <Column
            v-if="sistemaStore.mostrarColumnaNumeroProyectos"
            field="numero"
            :header="$t('Proyectos.Numero')"
            sortable
            header-class="text-center justify-center"
            class="text-center"
            :style="{ width: '6rem', textAlign: 'center' }"
          >
            <template #body="{ node }">
              <span v-if="node.data.isLoading" class="text-muted-foreground">...</span>
              <span v-else-if="node.data.isEmpty" class="text-muted-foreground text-xs">— sin trabajos —</span>
              <div v-else class="tabular-nums text-center font-mono font-medium">{{ node.data.numero }}</div>
            </template>
          </Column>

          <Column field="nombre" :header="$t('Proyectos.Nombre')" sortable expander>
            <template #body="{ node }">
              <span v-if="node.data.isLoading" class="flex items-center gap-2 text-muted-foreground">
                <AppIcon name="loader-2" :size="14" class="animate-spin" /> Cargando...
              </span>
              <span v-else-if="node.data.isEmpty" class="text-muted-foreground text-xs italic">No hay trabajos en este proyecto</span>
              <span
                v-else
                class="truncate cursor-pointer hover:underline"
                :class="node.data.isTrabajo ? 'text-muted-foreground pl-1' : 'font-medium'"
                @click="onRowNavigate(node)"
              >
                {{ node.data.nombre }}
              </span>
            </template>
          </Column>

          <Column field="clienteNombre" :header="$t('Proyectos.Cliente')" sortable>
            <template #body="{ node }">
              <span v-if="node.data.isLoading || node.data.isEmpty">—</span>
              <span v-else>{{ node.data.clienteNombre }}</span>
            </template>
          </Column>

          <Column field="localidad" :header="$t('Proyectos.Localidad')">
            <template #body="{ node }">
              <span v-if="node.data.isLoading || node.data.isEmpty">—</span>
              <span v-else>{{ node.data.localidad ?? '—' }}</span>
            </template>
          </Column>

          <Column field="estado" :header="$t('Proyectos.Estado')" sortable>
            <template #body="{ node }">
              <template v-if="node.data.isLoading || node.data.isEmpty">—</template>
              <StatePill
                v-else-if="node.data.isProyecto"
                entity="Proyecto"
                :value="node.data.estado"
              />
              <StatePill v-else entity="Trabajo" :value="node.data.estado" />
            </template>
          </Column>

          <Column field="trabajosCount" :header="$t('Proyectos.Trabajos')" sortable :style="{ width: '7rem' }">
            <template #body="{ node }">
              <span v-if="node.data.isTrabajo || node.data.isLoading || node.data.isEmpty">—</span>
              <span v-else class="tabular-nums">{{ node.data.trabajosCount }}</span>
            </template>
          </Column>

          <Column field="rentabilidad" :header="$t('Proyectos.Rentabilidad')" sortable>
            <template #body="{ node }">
              <template v-if="node.data.isLoading || node.data.isEmpty">—</template>
              <template v-else-if="node.data.isTrabajo">
                <span v-if="node.data.presupuesto" class="text-xs text-muted-foreground">
                  <span class="opacity-80">Presup.:</span> <MoneyText :value="node.data.presupuesto" />
                </span>
                <span v-else>—</span>
              </template>
              <MoneyText v-else :value="node.data.rentabilidad" colored />
            </template>
          </Column>

          <Column :header="$t('General.Actions')" :style="{ width: '8rem' }">
            <template #body="{ node }">
              <div v-if="node.data.isProyecto" class="flex gap-1">
                <Button
                  v-if="node.data.proyecto.estado !== 'Finalizada' && node.data.proyecto.estado !== 'Cancelada'"
                  variant="ghost"
                  size="sm"
                  title="Agregar Trabajo a este Proyecto"
                  @click="emit('proyectoCreateTrabajo', node.data.proyecto)"
                >
                  <AppIcon name="plus" :size="14" />
                </Button>
                <Button
                  v-if="node.data.proyecto.estado !== 'Finalizada' && node.data.proyecto.estado !== 'Cancelada'"
                  variant="ghost"
                  size="sm"
                  :title="$t('Actions.Proyecto.Finalizada')"
                  @click="emit('proyectoTransition', node.data.proyecto, 'Finalizada')"
                >
                  <AppIcon name="check" :size="14" />
                </Button>
                <Button variant="ghost" size="sm" :aria-label="$t('General.Edit')" @click="emit('proyectoEdit', node.data.proyecto)">
                  <AppIcon name="pencil" :size="14" />
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  :disabled="!node.data.proyecto.puedeEliminarse"
                  :title="!node.data.proyecto.puedeEliminarse ? $t('Proyectos.NoBorrable') : undefined"
                  :aria-label="$t('General.Delete')"
                  @click="emit('proyectoDelete', node.data.proyecto)"
                >
                  <AppIcon name="trash-2" :size="14" />
                </Button>
              </div>
              <div v-else-if="node.data.isTrabajo" class="flex gap-1">
                <Button variant="ghost" size="sm" :title="$t('General.View')" @click="emit('trabajoNavigate', node.data.trabajo)">
                  <AppIcon name="eye" :size="14" />
                </Button>
              </div>
            </template>
          </Column>
        </TreeTable>

        <ContextMenu ref="contextMenu" :model="contextMenuModel" />

        <Paginator
          :first="(table.page.value - 1) * rowsPerPage"
          :rows="rowsPerPage"
          :total-records="table.total.value"
          :rows-per-page-options="pageSizeOptions"
          template="FirstPageLink PrevPageLink CurrentPageReport NextPageLink LastPageLink RowsPerPageDropdown"
          :current-page-report-template="pageReportTemplate"
          @page="onPage"
        />
      </div>
    </ListState>
  </div>
</template>
