import { useToast } from 'primevue/usetoast'
import { useI18n } from 'vue-i18n'

import { isApiError, type ApiError } from '@/api/client'

/** Re-exported so a view can hold an error without importing from `api/`. */
export type { ApiError } from '@/api/client'

/**
 * Turns an `ApiError` into something the user can read. See `docs/16-frontend.md` §6.2.
 *
 * Two rules that hold everywhere:
 *
 * - `messageKey` is an i18n key, never a sentence. A missing key falls back to a generic message
 *   and is logged, so it shows up in a test run rather than in front of a user.
 * - `traceId` identifies the matching line in the backend log and is shown only for the errors
 *   that a user would report.
 */

/** How long each severity stays on screen, in milliseconds. `undefined` means it stays. */
const LIFE: Record<string, number | undefined> = {
  success: 3000,
  info: 4000,
  warn: 5000,
  error: undefined,
}

export function useApiError() {
  const toast = useToast()
  const { t, te } = useI18n()

  function translate(error: ApiError): string {
    if (te(error.messageKey)) return t(error.messageKey, error.params)
    console.warn(`[i18n] missing key ${error.messageKey}`)
    return t('Error.Unexpected')
  }

  /** Field path to message, ready to hand to a form. */
  function fieldErrors(error: ApiError): Record<string, string> {
    return Object.fromEntries(
      error.fields.map((f) => [
        f.field,
        te(f.messageKey) ? t(f.messageKey, f.params) : t('Error.Validation'),
      ]),
    )
  }

  /**
   * Shows the error, if it is one that deserves a toast.
   *
   * A validation error never gets one: it is painted on the offending fields, and a toast on top
   * of that is noise. Returns whether the caller still has to do something with it.
   */
  function notify(raw: unknown): ApiError {
    const error = isApiError(raw)
      ? raw
      : ({
          code: 'UNEXPECTED',
          messageKey: 'Error.Unexpected',
          params: {},
          fields: [],
          traceId: '',
        } satisfies ApiError)

    const severity = severityOf(error)
    if (severity) {
      toast.add({
        severity,
        summary: t(`Error.Summary.${severity}`),
        detail: withTrace(translate(error), error),
        life: LIFE[severity],
      })
    }
    return error
  }

  function withTrace(message: string, error: ApiError): string {
    // Only the errors a user would report carry the identifier; adding it to a business rule
    // message would be noise.
    return error.code === 'UNEXPECTED' && error.traceId ? `${message} (${error.traceId})` : message
  }

  return { translate, fieldErrors, notify }
}

/** `null` when the error is handled by the form rather than by a toast. */
function severityOf(error: ApiError): 'info' | 'warn' | 'error' | null {
  switch (error.code) {
    case 'VALIDATION':
      return null
    case 'NOT_FOUND':
    case 'CONFLICT':
    case 'CONCURRENCY':
    case 'DEPENDENCY_IN_USE':
    case 'DOMAIN':
      return 'warn'
    case 'EXTERNAL_UNAVAILABLE':
      // Informative, not an error: the system keeps working without the external service.
      return 'info'
    default:
      return 'error'
  }
}
