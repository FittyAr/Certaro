import { effectScope, nextTick } from 'vue'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import type { ListQuery, PagedResult } from '@/api/types'
import { useServerTable } from '@/composables/useServerTable'

/** Behaviour required by `docs/16-frontend.md` §5.1. */

interface Filtro {
  texto: string
}

function page(items: string[], total = items.length): PagedResult<string> {
  return {
    items,
    totalCount: total,
    page: 1,
    size: 30,
    totalPages: 1,
    hasPrevious: false,
    hasNext: false,
  }
}

function table(fetch: (q: ListQuery<Filtro>) => Promise<PagedResult<string>>) {
  const scope = effectScope()
  return scope.run(() =>
    useServerTable<Filtro, string>({
      key: 'test',
      initialFilter: { texto: '' },
      fetch,
      syncUrl: false,
    }),
  )!
}

/** Lets the debounce elapse and the pending promises settle. */
async function settle() {
  await vi.advanceTimersByTimeAsync(400)
  await nextTick()
}

describe('useServerTable', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    // jsdom in this configuration exposes no storage, and the persistence rule is part of what is
    // under test, so a minimal in-memory one is installed for the suite.
    const store = new Map<string, string>()
    vi.stubGlobal('localStorage', {
      getItem: (key: string) => store.get(key) ?? null,
      setItem: (key: string, value: string) => void store.set(key, value),
      removeItem: (key: string) => void store.delete(key),
      clear: () => store.clear(),
    })
  })

  it('un cambio de filtro se agrupa en una sola peticion', async () => {
    const fetch = vi.fn((_query: ListQuery<Filtro>) => Promise.resolve(page(['a'])))
    const t = table(fetch)

    t.filter.value.texto = 'c'
    await nextTick()
    t.filter.value.texto = 'ca'
    await nextTick()
    t.filter.value.texto = 'cab'
    await settle()

    expect(fetch).toHaveBeenCalledTimes(1)
    expect(fetch.mock.calls[0]![0].filtro).toEqual({ texto: 'cab' })
  })

  it('una respuesta vieja que llega tarde no pisa a la nueva', async () => {
    // Without this, typing quickly leaves the table showing the result of an earlier search.
    const responses = [
      new Promise<PagedResult<string>>((resolve) =>
        setTimeout(() => resolve(page(['vieja'])), 500),
      ),
      Promise.resolve(page(['nueva'])),
    ]
    let call = 0
    const t = table(() => responses[call++]!)

    void t.reload()
    void t.reload()
    await vi.advanceTimersByTimeAsync(600)

    expect(t.rows.value).toEqual(['nueva'])
  })

  it('cambiar un filtro vuelve a la primera pagina', async () => {
    const t = table(async () => page(['a']))
    t.page.value = 3
    await settle()

    t.filter.value.texto = 'algo'
    await settle()

    expect(t.page.value).toBe(1)
  })

  it('cambiar el orden no vuelve a la primera pagina', async () => {
    const t = table(async () => page(['a']))
    t.page.value = 3
    await settle()

    t.onSort({ sortField: 'fecha', sortOrder: -1 })
    await settle()

    expect(t.page.value).toBe(3)
    expect(t.sort.value).toEqual({ field: 'fecha', dir: 'Desc' })
  })

  it('un error no vacia la tabla', async () => {
    let fail = false
    const t = table(async () => {
      if (fail)
        throw {
          code: 'PERSISTENCE',
          messageKey: 'Error.Persistence',
          params: {},
          fields: [],
          traceId: '',
        }
      return page(['una fila'])
    })

    await t.reload()
    fail = true
    await t.reload()

    // Losing what was on screen because of one failed request is worse than showing it stale.
    expect(t.rows.value).toEqual(['una fila'])
    expect(t.error.value?.code).toBe('PERSISTENCE')
  })

  it('el tamano de pagina y el orden se persisten, los filtros no', async () => {
    const first = table(async () => page(['a']))
    first.pageSize.value = 50
    first.onSort({ sortField: 'nombre', sortOrder: 1 })
    first.filter.value.texto = 'no deberia volver'
    await settle()

    const second = table(async () => page(['a']))

    expect(second.pageSize.value).toBe(50)
    expect(second.sort.value).toEqual({ field: 'nombre', dir: 'Asc' })
    expect(second.filter.value.texto).toBe('')
  })

  it('la pagina del paginador es base cero y la del backend base uno', async () => {
    const fetch = vi.fn((_query: ListQuery<Filtro>) => Promise.resolve(page(['a'])))
    const t = table(fetch)
    t.onPage({ page: 2, rows: 30 })
    await settle()

    expect(t.page.value).toBe(3)
    expect(fetch.mock.calls.at(-1)![0].page).toBe(3)
  })

  it('distingue estar vacio de no tener resultados', async () => {
    const t = table(async () => page([], 0))
    await t.reload()

    expect(t.isEmpty.value).toBe(true)
    expect(t.isFiltered.value).toBe(false)

    t.filter.value.texto = 'algo'
    await settle()

    expect(t.isFiltered.value).toBe(true)
  })
})
