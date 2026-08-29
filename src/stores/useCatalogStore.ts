import { defineStore } from 'pinia'
import { ref } from 'vue'

import { lookupCategorias } from '@/api/categorias'
import { lookupTiposMovimiento } from '@/api/tiposMovimiento'
import type { LookupItem } from '@/api/types'

export type { LookupItem } from '@/api/types'

/**
 * The catalogues many screens need for their selectors. See `docs/16-frontend.md` §7.2.
 *
 * They are loaded once and kept until something writes to them. There is no time-based expiry:
 * this is a single-user desktop application, so nothing changes the data behind our back — but a
 * create or an edit does, and each module invalidates what it touched.
 */
export const useCatalogStore = defineStore('catalog', () => {
  const tiposMovimiento = ref<LookupItem[]>([])
  const categorias = ref<LookupItem[]>([])

  async function loadTiposMovimiento(force = false): Promise<LookupItem[]> {
    if (force || tiposMovimiento.value.length === 0) {
      tiposMovimiento.value = await lookupTiposMovimiento(undefined, 100)
    }
    return tiposMovimiento.value
  }

  async function loadCategorias(force = false): Promise<LookupItem[]> {
    if (force || categorias.value.length === 0) {
      categorias.value = await lookupCategorias(undefined, 200)
    }
    return categorias.value
  }

  function invalidateTiposMovimiento(): void {
    tiposMovimiento.value = []
  }

  function invalidateCategorias(): void {
    categorias.value = []
  }

  return {
    tiposMovimiento,
    categorias,
    loadTiposMovimiento,
    loadCategorias,
    invalidateTiposMovimiento,
    invalidateCategorias,
  }
})
