<script setup lang="ts" generic="TFilter extends object, TRow">
import Column from 'primevue/column'
import ContextMenu from 'primevue/contextmenu'
import DataTable, { type DataTableSortEvent } from 'primevue/datatable'
import Paginator, { type PageState } from 'primevue/paginator'
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'

import { PAGE_SIZES } from '@/api/types'
import ListState from '@/components/domain/ListState.vue'
import type { useServerTable } from '@/composables/useServerTable'

/**
 * Wraps `DataTable` with the configuration every list shares and wires it to `useServerTable`.
 * A view never binds the table's events by hand. See `docs/16-frontend.md` §5.1.
 */

const props = defineProps<{
  table: ReturnType<typeof useServerTable<TFilter, TRow>>
  emptyKey: string
  /** Column key that identifies a row, for selection and keyboard navigation. */
  dataKey?: string
  /** Row-specific context menu items. If not provided, no context menu is shown. */
  contextMenuItems?: (row: TRow) => { label?: string; icon?: string; command?: () => void; disabled?: boolean; separator?: boolean }[]
}>()

const emit = defineEmits<{ rowEdit: [row: TRow] }>()

const table = computed(() => props.table)

/** `0` means "all", which the paginator cannot express, so it is folded into the total. */
const rowsPerPage = computed(() =>
  table.value.pageSize.value === 0
    ? Math.max(table.value.total.value, 1)
    : table.value.pageSize.value,
)

const pageSizeOptions = PAGE_SIZES.filter((size) => size !== 0)
const { t } = useI18n()

// Vue-i18n consumes brace placeholders, while PrimeVue must receive them untouched.
const pageReportTemplate = computed(() =>
  t('General.PageReport', { first: '{first}', last: '{last}', totalRecords: '{totalRecords}' }),
)

defineOptions({ inheritAttrs: false })

const contextMenu = ref<any>(null)
const contextMenuModel = ref<unknown[]>([])
const contextMenuSelection = ref<TRow | null>(null)

function onSort(event: DataTableSortEvent): void {
  table.value.onSort({
    sortField: typeof event.sortField === 'string' ? event.sortField : null,
    sortOrder: event.sortOrder ?? null,
  })
}

function onPage(event: PageState): void {
  table.value.onPage({ page: event.page, rows: event.rows })
}

function onRowContextMenu(event: { originalEvent: Event; data: TRow }): void {
  if (!props.contextMenuItems) return
  contextMenuSelection.value = event.data as TRow
  contextMenuModel.value = props.contextMenuItems(event.data as TRow) as unknown[]
  ;(contextMenu.value as any)?.show(event.originalEvent as MouseEvent)
}
</script>

<template>
  <div :class="['flex w-full flex-col', ($attrs.class as string) || '']">
    <ListState
      :loading="table.loading.value"
      :first-load="table.firstLoad.value"
      :error="table.error.value"
      :is-empty="table.isEmpty.value"
      :is-filtered="table.isFiltered.value"
      :empty-key="props.emptyKey"
      @retry="table.reload()"
      @clear-filters="table.resetFilter()"
    >
      <template #empty-action><slot name="empty-action" /></template>

      <!-- A reload dims the previous rows instead of replacing them, so the screen does not jump. -->
      <div :class="table.loading.value ? 'opacity-60 transition-opacity' : ''">
        <DataTable
          :value="table.rows.value"
          :data-key="props.dataKey ?? 'id'"
          :sort-field="table.sort.value?.field ?? undefined"
          :sort-order="table.sort.value?.dir === 'Desc' ? -1 : 1"
          removable-sort
          scrollable
          scroll-height="flex"
          size="small"
          class="text-sm"
          context-menu
          v-model:context-menu-selection="contextMenuSelection"
          @sort="onSort"
          @row-dblclick="emit('rowEdit', $event.data as TRow)"
          @row-contextmenu="onRowContextMenu"
        >
          <slot />
          <Column v-if="$slots.actions" :header="$t('General.Actions')" :style="{ width: '6rem' }">
            <template #body="slotProps">
              <slot name="actions" v-bind="slotProps" />
            </template>
          </Column>
        </DataTable>
        <ContextMenu v-if="props.contextMenuItems" ref="contextMenu" :model="contextMenuModel as any" />

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
