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
import { useTrabajosStore, type TrabajoListItem } from '@/stores/useTrabajosStore'
import type { ProyectoListItem } from '@/stores/useProyectosStore'

const props = defineProps<{
  table: ReturnType<typeof useServerTable<TFilter, ProyectoListItem>>
}>()

const emit = defineEmits<{
  proyectoEdit: [row: ProyectoListItem]
  proyectoDelete: [row: ProyectoListItem]
  proyectoTransition: [row: ProyectoListItem, estado: string]
  trabajoNavigate: [row: TrabajoListItem]
}>()

const { t } = useI18n()
const router = useRouter()
const { notify } = useApiError()
const trabajosStore = useTrabajosStore()

const contextMenu = ref<InstanceType<typeof ContextMenu> | null>(null)
const contextNode = ref<TreeNode | null>(null)

const expandedKeys = ref<Record<string, boolean>>({})
const trabajosMap = ref<Map<string, TrabajoListItem[]>>(new Map())
const loadingTrabajos = ref<Set<string>>(new Set())

const rowsPerPage = computed(() =>
  props.table.pageSize.value === 0
    ? Math.max(props.table.total.value, 1)
    : props.table.pageSize.value,
)
const pageSizeOptions = PAGE_SIZES.filter((size) => size !== 0)
const pageReportTemplate = computed(() =>
  t('General.PageReport', { first: '{first}', last: '{last}', totalRecords: '{totalRecords}' }),
)

const treeValue = computed<TreeNode[]>(() => {
  return props.table.rows.value.map((proyecto) => {
    const isExpanded = expandedKeys.value[proyecto.id] === true
    const trabajos = trabajosMap.value.get(proyecto.id) ?? []
    const isLoading = loadingTrabajos.value.has(proyecto.id)

    const children: TreeNode[] | undefined = isExpanded
      ? isLoading
        ? [{ key: `${proyecto.id}-loading`, data: { isLoading: true }, leaf: true }]
        : trabajos.length > 0
          ? trabajos.map((trab) => ({
              key: trab.id,
              data: {
                isTrabajo: true,
                trabajo: trab,
                // Map to common column fields
                numero: '—',
                nombre: trab.descripcion,
                clienteNombre: trab.clienteNombre,
                localidad: trab.fechaInicio,
                estado: trab.estado,
                trabajosCount: null,
                rentabilidad: trab.presupuesto,
                proyecto: null,
              },
              leaf: true,
            }))
          : [{ key: `${proyecto.id}-empty`, data: { isEmpty: true }, leaf: true }]
      : undefined

    return {
      key: proyecto.id,
      data: {
        isProyecto: true,
        proyecto,
        numero: proyecto.numero,
        nombre: proyecto.nombre,
        clienteNombre: proyecto.clienteNombre,
        localidad: proyecto.localidad,
        estado: proyecto.estado,
        trabajosCount: proyecto.trabajosCount,
        rentabilidad: proyecto.rentabilidad,
      },
      children,
      leaf: proyecto.trabajosCount === 0,
    }
  })
})

async function onExpand(node: TreeNode): Promise<void> {
  const proyectoId = String(node.key)
  const proyecto = props.table.rows.value.find((p) => p.id === proyectoId)
  if (!proyecto || proyecto.trabajosCount === 0) return
  if (trabajosMap.value.has(proyectoId)) return

  loadingTrabajos.value = new Set(loadingTrabajos.value).add(proyectoId)
  try {
    const res = await trabajosStore.fetchPaged({
      page: 1,
      pageSize: 100,
      filtro: { proyectoId } as unknown as Record<string, unknown>,
      sortDir: 'Asc',
    })
    const next = new Map(trabajosMap.value)
    next.set(proyectoId, res.items)
    trabajosMap.value = next
  } catch (e) {
    notify(e)
  } finally {
    const next = new Set(loadingTrabajos.value)
    next.delete(proyectoId)
    loadingTrabajos.value = next
  }
}

function onCollapse(node: TreeNode): void {
  // Keep data cached, no need to clear
  void node
}

function handleExpand(node: TreeNode): void {
  expandedKeys.value = { ...expandedKeys.value, [String(node.key)]: true }
  void onExpand(node)
}

function handleCollapse(node: TreeNode): void {
  const key = String(node.key)
  const next = { ...expandedKeys.value }
  delete next[key]
  expandedKeys.value = next
  onCollapse(node)
}

// Clear cache when table reloads (page change, filter change)
watch(
  () => props.table.rows.value.map((r) => r.id).join(','),
  () => {
    // Keep expanded but clear trabajos that are no longer in current page
    const validIds = new Set(props.table.rows.value.map((r) => r.id))
    const next = new Map<string, TrabajoListItem[]>()
    for (const [k, v] of trabajosMap.value.entries()) {
      if (validIds.has(k)) next.set(k, v)
    }
    trabajosMap.value = next
    // Remove expanded keys for rows no longer present
    const nextKeys: Record<string, boolean> = {}
    for (const k of Object.keys(expandedKeys.value)) {
      if (validIds.has(k)) nextKeys[k] = true
    }
    expandedKeys.value = nextKeys
  },
)

const contextMenuModel = computed(() => {
  const node = contextNode.value
  if (!node) return []
  const data = node.data as { isProyecto?: boolean; isTrabajo?: boolean; proyecto?: ProyectoListItem; trabajo?: TrabajoListItem }
  if (data.isProyecto && data.proyecto) {
    const p = data.proyecto
    return [
      {
        label: t('General.Edit'),
        icon: 'pi pi-pencil',
        command: () => emit('proyectoEdit', p),
      },
      {
        label: t('General.Delete'),
        icon: 'pi pi-trash',
        disabled: !p.puedeEliminarse,
        command: () => emit('proyectoDelete', p),
      },
      { separator: true },
      {
        label: t('Actions.Proyecto.Finalizada'),
        icon: 'pi pi-check',
        disabled: p.estado === 'Finalizada' || p.estado === 'Cancelada',
        command: () => emit('proyectoTransition', p, 'Finalizada'),
      },
      { separator: true },
      {
        label: t('Proyectos.VerTrabajos'),
        icon: 'pi pi-hammer',
        command: () => void router.push({ name: 'proyecto-trabajos', params: { proyectoId: p.id } }),
      },
      {
        label: t('Proyectos.VerCaja'),
        icon: 'pi pi-wallet',
        command: () => void router.push({ name: 'proyecto-caja', params: { proyectoId: p.id } }),
      },
      {
        label: t('Proyectos.VerKanban') || 'Ver en Kanban',
        icon: 'pi pi-th-large',
        command: () => void router.push({ path: '/kanban', query: { proyectoId: p.id } }),
      },
    ]
  }
  if (data.isTrabajo && data.trabajo) {
    const tr = data.trabajo
    return [
      {
        label: t('General.View'),
        icon: 'pi pi-eye',
        command: () => emit('trabajoNavigate', tr),
      },
      {
        label: t('General.Edit'),
        icon: 'pi pi-pencil',
        command: () => void router.push({ name: 'trabajo-detalle', params: { trabajoId: tr.id } }),
      },
    ]
  }
  return []
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
          <Column field="numero" :header="$t('Proyectos.Numero')" sortable :style="{ width: '6rem' }">
            <template #body="{ node }">
              <span v-if="node.data.isLoading" class="text-muted-foreground">...</span>
              <span v-else-if="node.data.isEmpty" class="text-muted-foreground text-xs">— sin trabajos —</span>
              <span v-else class="tabular-nums">{{ node.data.numero }}</span>
            </template>
          </Column>

          <Column field="nombre" :header="$t('Proyectos.Nombre')" sortable expander>
            <template #body="{ node }">
              <span v-if="node.data.isLoading" class="flex items-center gap-2 text-muted-foreground">
                <AppIcon name="loader-2" :size="14" class="animate-spin" /> Cargando...
              </span>
              <span v-else-if="node.data.isEmpty" class="text-muted-foreground text-xs italic">No hay trabajos en este proyecto</span>
              <span v-else class="truncate" :class="node.data.isTrabajo ? 'text-muted-foreground' : 'font-medium'">
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
              <MoneyText v-else :value="node.data.rentabilidad" :colored="!node.data.isTrabajo" />
            </template>
          </Column>

          <Column :header="$t('General.Actions')" :style="{ width: '8rem' }">
            <template #body="{ node }">
              <div v-if="node.data.isProyecto" class="flex gap-1">
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
