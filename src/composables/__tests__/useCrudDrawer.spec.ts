/* eslint-disable @typescript-eslint/no-explicit-any */
import { effectScope } from 'vue'
import { beforeEach, describe, expect, it, vi } from 'vitest'

vi.mock('vue-i18n', () => ({ useI18n: () => ({ t: (k: string) => k }) }))
vi.mock('primevue/useconfirm', () => ({ useConfirm: () => ({ require: vi.fn() }) }))
vi.mock('primevue/usetoast', () => ({ useToast: () => ({ add: vi.fn() }) }))
vi.mock('@/composables/useApiError', () => ({
  useApiError: () => ({
    fieldErrors: (e: any) => Object.fromEntries(e.fields.map((f: any) => [f.field, f.messageKey])),
    notify: (e: any) => e,
    toFieldErrors: () => ({}),
  }),
}))
vi.mock('@/composables/useEscapeStack', () => ({
  useEscapeLayer: () => ({ push: vi.fn(), pop: vi.fn() }),
}))

import { useCrudDrawer } from '@/composables/useCrudDrawer'

function drawer(opts: Partial<Parameters<typeof useCrudDrawer>[0]> = {}) {
  const scope = effectScope()
  return scope.run(() =>
    useCrudDrawer({
      entityKey: 'Entity.Test',
      empty: () => ({ nombre: '' }),
      load: async () => ({ nombre: 'cargado' }),
      create: async () => ({}),
      update: async () => ({}),
      ...opts,
    }),
  )!
}

describe('useCrudDrawer', () => {
  beforeEach(() => vi.clearAllMocks())

  it('openCreate inicia en modo create y marca limpio', () => {
    const d = drawer()
    d.openCreate()
    expect(d.mode.value).toBe('create')
    expect(d.open.value).toBe(true)
    expect(d.isDirty.value).toBe(false)
  })

  it('editar marca dirty al cambiar el modelo', async () => {
    const d = drawer()
    await d.openEdit('id-1' as any)
    expect(d.isDirty.value).toBe(false)
    d.model.value = { nombre: 'cambiado' } as any
    expect(d.isDirty.value).toBe(true)
  })

  it('save con VALIDATION deja el drawer abierto y expone fieldErrors', async () => {
    const create = vi.fn(async () => {
      throw {
        code: 'VALIDATION',
        messageKey: 'Validation.Invalid',
        params: {},
        fields: [{ field: 'nombre', messageKey: 'Validation.Required', params: {} }],
        traceId: '',
      }
    })
    const d = drawer({ create })
    d.openCreate()
    await d.save()
    expect(d.open.value).toBe(true)
    expect(d.fieldErrors.value.nombre).toBeDefined()
  })

  it('save con CONCURRENCY marca staleConflict', async () => {
    const create = vi.fn(async () => {
      throw {
        code: 'CONCURRENCY',
        messageKey: 'Error.Concurrency',
        params: {},
        fields: [],
        traceId: '',
      }
    })
    const d = drawer({ create })
    d.openCreate()
    await d.save()
    expect(d.staleConflict.value).toBe(true)
    expect(d.open.value).toBe(true)
  })
})
