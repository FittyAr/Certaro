import { useConfirm } from 'primevue/useconfirm'
import { useToast } from 'primevue/usetoast'
import { useI18n } from 'vue-i18n'

import { useApiError } from '@/composables/useApiError'

/**
 * One delete confirmation for the whole system. See `docs/16-frontend.md` §5.3.
 *
 * The legacy system had a different wording per screen and some of them were never translated.
 */

export interface ConfirmDeleteOptions {
  /** i18n key of the entity name, e.g. `Entity.Movimiento`. */
  entityKey: string
  /** What identifies the record for the user: a concept, a name, a number. */
  label: string
  action: () => Promise<unknown>
  onDone?: () => void
}

export function useConfirmDelete() {
  const confirm = useConfirm()
  const toast = useToast()
  const { t } = useI18n()
  const { notify } = useApiError()

  function confirmDelete(opts: ConfirmDeleteOptions): void {
    confirm.require({
      header: t('General.DeleteConfirmTitle'),
      message: t('General.DeleteConfirm', {
        entity: t(opts.entityKey),
        label: opts.label,
      }),
      acceptLabel: t('General.Delete'),
      rejectLabel: t('General.Cancel'),
      acceptProps: { severity: 'danger' },
      accept: async () => {
        try {
          await opts.action()
          toast.add({
            severity: 'success',
            summary: t('General.Deleted'),
            detail: opts.label,
            life: 3000,
          })
          opts.onDone?.()
        } catch (e) {
          // A refusal such as "this type is used by 12 movements" is a warning with its own
          // message, and `notify` already picks the right severity for it.
          notify(e)
        }
      },
    })
  }

  return { confirmDelete }
}
