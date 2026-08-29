import { invoke } from '@tauri-apps/api/core'

/**
 * The only module that talks to Tauri. See `docs/11-contratos-tauri.md` §4.1.
 *
 * No component calls `invoke` directly, and this layer neither handles errors nor transforms data:
 * it normalises the error shape and gets out of the way. Formatting belongs to the components,
 * error handling to the stores and the global interceptor.
 */

export interface ApiFieldError {
  field: string
  messageKey: string
  params: Record<string, string>
}

export interface ApiError {
  code: string
  messageKey: string
  params: Record<string, string>
  fields: ApiFieldError[]
  traceId: string
}

export type ApiErrorCode =
  | 'VALIDATION'
  | 'NOT_FOUND'
  | 'CONFLICT'
  | 'CONCURRENCY'
  | 'DEPENDENCY_IN_USE'
  | 'DOMAIN'
  | 'PERSISTENCE'
  | 'EXTERNAL_UNAVAILABLE'
  | 'IO'
  | 'UNEXPECTED'
  | 'IPC'

export function isApiError(value: unknown): value is ApiError {
  return (
    typeof value === 'object' &&
    value !== null &&
    'code' in value &&
    'messageKey' in value &&
    typeof (value as ApiError).code === 'string'
  )
}

/**
 * Anything Tauri throws that is not already an `ApiError` — a panic, a missing command, a
 * serialisation failure — becomes one, so callers only ever handle a single error shape.
 */
function normalise(error: unknown): ApiError {
  if (isApiError(error)) return error
  return {
    code: 'IPC',
    messageKey: 'Error.Unexpected',
    params: {},
    fields: [],
    traceId: '',
  }
}

export async function callCommand<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await invoke<T>(command, args)
  } catch (error) {
    const apiError = normalise(error)
    // Logged with the trace id so a user report maps to a line in the backend log file.
    console.error(`[ipc] ${command} failed`, {
      code: apiError.code,
      traceId: apiError.traceId,
    })
    throw apiError
  }
}
