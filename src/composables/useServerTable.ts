import { computed, ref, watch, type Ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import type { ApiError } from '@/api/client'
import type { ListQuery, PagedResult, SortDir } from '@/api/types'
import { DEFAULT_PAGE_SIZE } from '@/api/types'

/**
 * Server-side paging, filtering and sorting. See `docs/16-frontend.md` §5.1.
 *
 * Every list screen uses this; there is exactly one implementation of pagination in the system.
 */

export interface ServerTableOptions<TFilter extends object, TRow, TSummary = unknown> {
  /** Key under which page size and sort are persisted, e.g. `movimientos`. */
  key: string
  initialFilter: TFilter
  fetch: (query: ListQuery<TFilter>) => Promise<PagedResult<TRow> & { resumen?: TSummary }>
  defaultSort?: { field: string; dir: SortDir }
  /** Off in tests and in embedded tables that must not fight over the query string. */
  syncUrl?: boolean
}

const DEBOUNCE_MS = 300

interface Persisted {
  pageSize?: number
  sort?: { field: string; dir: SortDir } | null
}

function readPersisted(key: string): Persisted {
  try {
    return JSON.parse(localStorage.getItem(`eo.table.${key}`) ?? '{}') as Persisted
  } catch {
    // A corrupt entry is not worth an error: the defaults are perfectly usable.
    return {}
  }
}

function writePersisted(key: string, value: Persisted): void {
  try {
    localStorage.setItem(`eo.table.${key}`, JSON.stringify(value))
  } catch {
    // Storage can be full or disabled; losing a preference is not worth interrupting the user.
  }
}

/** Only the filter entries that carry a value, so the URL stays readable. */
function meaningfulFilter(filter: object): Record<string, string> {
  const out: Record<string, string> = {}
  for (const [key, value] of Object.entries(filter)) {
    if (value === null || value === undefined || value === '' || value === false) continue
    out[key] = String(value)
  }
  return out
}

export function useServerTable<TFilter extends object, TRow, TSummary = unknown>(
  opts: ServerTableOptions<TFilter, TRow, TSummary>,
) {
  const persisted = readPersisted(opts.key)

  const rows = ref([]) as Ref<TRow[]>
  const total = ref(0)
  const summary = ref<TSummary | null>(null) as Ref<TSummary | null>
  const loading = ref(false)
  /** True only for the very first load, which is what shows a skeleton instead of a dimmed table. */
  const firstLoad = ref(true)
  const error = ref<ApiError | null>(null)

  const filter = ref({ ...opts.initialFilter }) as Ref<TFilter>
  const page = ref(1)
  const pageSize = ref(persisted.pageSize ?? DEFAULT_PAGE_SIZE)
  const sort = ref(persisted.sort ?? opts.defaultSort ?? null)

  const isEmpty = computed(() => !loading.value && !error.value && rows.value.length === 0)
  /** Empty because of the filters rather than because there is nothing: a different message. */
  const isFiltered = computed(() => Object.keys(meaningfulFilter(filter.value)).length > 0)

  // A response that arrives after a newer one is discarded. Without this, typing quickly in a
  // filter leaves the table showing the result of an earlier search: hard to reproduce, easy to
  // prevent.
  let sequence = 0
  let debounce: ReturnType<typeof setTimeout> | undefined

  const route = opts.syncUrl === false ? null : useRoute()
  const router = opts.syncUrl === false ? null : useRouter()

  async function reload(): Promise<void> {
    const mine = ++sequence
    loading.value = true

    try {
      const result = await opts.fetch({
        filtro: { ...filter.value },
        page: page.value,
        pageSize: pageSize.value,
        sortBy: sort.value?.field,
        sortDir: sort.value?.dir,
      })
      if (mine !== sequence) return

      rows.value = result.items
      total.value = result.totalCount
      summary.value = (result.resumen ?? null) as TSummary | null
      error.value = null
    } catch (e) {
      if (mine !== sequence) return
      // `rows` is left alone on purpose: losing what was already on screen because of one failed
      // request is worse than showing it slightly out of date.
      error.value = e as ApiError
    } finally {
      if (mine === sequence) {
        loading.value = false
        firstLoad.value = false
      }
    }
  }

  function reloadDebounced(): void {
    clearTimeout(debounce)
    debounce = setTimeout(() => void reload(), DEBOUNCE_MS)
  }

  function syncUrl(): void {
    if (!router || !route) return
    const query: Record<string, string> = meaningfulFilter(filter.value)
    if (page.value !== 1) query.page = String(page.value)
    if (pageSize.value !== DEFAULT_PAGE_SIZE) query.pageSize = String(pageSize.value)
    void router.replace({ query })
  }

  /** Reads back what {@link syncUrl} wrote, so F5 and a shared link land on the same view. */
  function readUrl(): void {
    if (!route) return
    const query = route.query
    for (const key of Object.keys(filter.value)) {
      const value = query[key]
      if (typeof value === 'string') {
        ;(filter.value as Record<string, unknown>)[key] = value
      }
    }
    if (typeof query.page === 'string') page.value = Number(query.page) || 1
    if (typeof query.pageSize === 'string') pageSize.value = Number(query.pageSize)
  }

  watch(
    filter,
    () => {
      page.value = 1
      syncUrl()
      reloadDebounced()
    },
    { deep: true },
  )

  watch(page, () => {
    syncUrl()
    void reload()
  })

  watch(pageSize, (size) => {
    page.value = 1
    writePersisted(opts.key, { pageSize: size, sort: sort.value })
    syncUrl()
    void reload()
  })

  // Changing the sort does not reset the page: the user is looking at page three and wants it
  // sorted differently, not to be sent back to the beginning.
  watch(sort, (value) => {
    writePersisted(opts.key, { pageSize: pageSize.value, sort: value })
    void reload()
  })

  function resetFilter(): void {
    filter.value = { ...opts.initialFilter }
  }

  function onPage(event: { page: number; rows: number }): void {
    // PrimeVue reports a zero-based page; the backend counts from one.
    page.value = event.page + 1
    pageSize.value = event.rows
  }

  function onSort(event: { sortField: string | null; sortOrder: number | null }): void {
    sort.value = event.sortField
      ? { field: event.sortField, dir: event.sortOrder === -1 ? 'Desc' : 'Asc' }
      : null
  }

  function start(): void {
    readUrl()
    void reload()
  }

  return {
    rows,
    total,
    summary,
    loading,
    firstLoad,
    error,
    isEmpty,
    isFiltered,
    filter,
    page,
    pageSize,
    sort,
    reload,
    resetFilter,
    onPage,
    onSort,
    start,
  }
}

export type ServerTable<TFilter extends object = object, TRow = unknown> = ReturnType<
  typeof useServerTable<TFilter, TRow>
>
