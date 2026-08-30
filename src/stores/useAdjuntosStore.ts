import { defineStore } from 'pinia'
import { ref } from 'vue'

import {
  addAdjunto,
  deleteAdjunto,
  listAdjuntos,
  openAdjunto,
  revealAdjunto,
  type AdjuntoItem,
  type AdjuntoInput,
  type EntidadAdjunto,
} from '@/api/adjuntos'
import type { Uuid } from '@/api/types'

/**
 * Attachments. See `docs/13-servicios-externos-y-archivos.md` §1.
 *
 * The drawer loads the list when it opens and clears it when it closes, so a stale count never
 * survives from a previous open.
 */
export const useAdjuntosStore = defineStore('adjuntos', () => {
  const items = ref<AdjuntoItem[]>([])
  const loading = ref(false)

  async function load(tipo: EntidadAdjunto, entidadId: Uuid): Promise<void> {
    loading.value = true
    try {
      items.value = await listAdjuntos(tipo, entidadId)
    } finally {
      loading.value = false
    }
  }

  async function add(input: AdjuntoInput): Promise<AdjuntoItem> {
    const item = await addAdjunto(input)
    items.value.push(item)
    return item
  }

  async function remove(id: Uuid): Promise<void> {
    await deleteAdjunto(id)
    items.value = items.value.filter((a) => a.id !== id)
  }

  async function open(id: Uuid): Promise<void> {
    await openAdjunto(id)
  }

  async function reveal(id: Uuid): Promise<void> {
    await revealAdjunto(id)
  }

  function clear(): void {
    items.value = []
  }

  return { items, loading, load, add, remove, open, reveal, clear }
})
