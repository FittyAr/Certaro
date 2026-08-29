import { useConfirm } from 'primevue/useconfirm'
import { useToast } from 'primevue/usetoast'
import { computed, ref, type Ref } from 'vue'
import { useI18n } from 'vue-i18n'

import { useApiError } from '@/composables/useApiError'
import { useEscapeLayer } from '@/composables/useEscapeStack'

/**
 * Creating and editing in a side drawer. See `docs/16-frontend.md` §5.2.
 *
 * A drawer rather than a page or a modal so the list stays visible behind it and the filtering
 * context is not lost.
 */

export interface CrudDrawerOptions<TDto, TId = string> {
  empty: () => TDto
  load: (id: TId) => Promise<TDto>
  create: (dto: TDto) => Promise<unknown>
  update: (id: TId, dto: TDto) => Promise<unknown>
  onSaved?: () => void
  /** i18n key of the entity name, for the title and the messages. */
  entityKey: string
}

export function useCrudDrawer<TDto extends object, TId = string>(
  opts: CrudDrawerOptions<TDto, TId>,
) {
  const { t } = useI18n()
  const toast = useToast()
  const confirm = useConfirm()
  const { fieldErrors: toFieldErrors, notify } = useApiError()

  const open = ref(false)
  const mode = ref<'create' | 'edit'>('create')
  const model = ref(opts.empty()) as Ref<TDto>
  const currentId = ref<TId | null>(null) as Ref<TId | null>
  const saving = ref(false)
  const loading = ref(false)
  const fieldErrors = ref<Record<string, string>>({})
  /** Set when the record changed underneath, so the drawer can offer to reload it. */
  const staleConflict = ref(false)

  /** Snapshot taken when the drawer opened, to tell a real edit from a round trip. */
  let pristine = JSON.stringify(opts.empty())

  const isDirty = computed(() => JSON.stringify(model.value) !== pristine)

  const layer = useEscapeLayer('drawer', async () => {
    if (!open.value) return false
    await close()
    return true
  })

  function reset(next: TDto): void {
    model.value = next
    pristine = JSON.stringify(next)
    fieldErrors.value = {}
    staleConflict.value = false
  }

  function openCreate(): void {
    mode.value = 'create'
    currentId.value = null
    reset(opts.empty())
    open.value = true
    layer.push()
  }

  async function openEdit(id: TId): Promise<void> {
    mode.value = 'edit'
    currentId.value = id
    reset(opts.empty())
    open.value = true
    layer.push()

    loading.value = true
    try {
      reset(await opts.load(id))
    } catch (e) {
      notify(e)
      forceClose()
    } finally {
      loading.value = false
    }
  }

  function forceClose(): void {
    open.value = false
    layer.pop()
    reset(opts.empty())
    currentId.value = null
  }

  /** Asks before throwing away a half-filled form: an accidental Escape must not lose it. */
  async function close(): Promise<void> {
    if (!isDirty.value) {
      forceClose()
      return
    }
    await new Promise<void>((resolve) => {
      confirm.require({
        message: t('General.DiscardChangesConfirm'),
        header: t('General.Confirm'),
        acceptLabel: t('General.Discard'),
        rejectLabel: t('General.KeepEditing'),
        accept: () => {
          forceClose()
          resolve()
        },
        reject: () => resolve(),
        onHide: () => resolve(),
      })
    })
  }

  async function save(): Promise<void> {
    if (saving.value) return
    saving.value = true
    fieldErrors.value = {}
    staleConflict.value = false

    try {
      if (mode.value === 'edit' && currentId.value !== null) {
        await opts.update(currentId.value, model.value)
      } else {
        await opts.create(model.value)
      }
      toast.add({
        severity: 'success',
        summary: t('General.Saved'),
        detail: t(opts.entityKey),
        life: 3000,
      })
      forceClose()
      opts.onSaved?.()
    } catch (e) {
      const error = notify(e)
      if (error.code === 'VALIDATION') {
        // The drawer stays open with the problems marked on the fields.
        fieldErrors.value = toFieldErrors(error)
      } else if (error.code === 'CONCURRENCY') {
        staleConflict.value = true
      }
    } finally {
      saving.value = false
    }
  }

  /** Discards the local edit and takes the record as it now stands in the database. */
  async function reloadCurrent(): Promise<void> {
    if (currentId.value === null) return
    loading.value = true
    try {
      reset(await opts.load(currentId.value))
    } catch (e) {
      notify(e)
    } finally {
      loading.value = false
    }
  }

  return {
    open,
    mode,
    model,
    currentId,
    saving,
    loading,
    fieldErrors,
    staleConflict,
    isDirty,
    openCreate,
    openEdit,
    save,
    close,
    forceClose,
    reloadCurrent,
  }
}
