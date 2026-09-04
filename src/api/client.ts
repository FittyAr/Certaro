import { invoke } from '@tauri-apps/api/core'
import { mockBrowserCommand } from './mock/handler'

/**
 * The only module that talks to Tauri. See `docs/11-contratos-tauri.md` §4.1.
 *
 * In web preview mode (outside Tauri runtime), provides a rich in-memory mock database
 * that persists in localStorage and responds reactively to all entity CRUD, lists, lookups, and dev seeding.
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

export function isTauri(): boolean {
  return typeof window !== 'undefined' && ('__TAURI_INTERNALS__' in window || '__TAURI__' in window)
}

export async function callCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) {
    return Promise.resolve(mockBrowserCommand<T>(command, args))
  }
  try {
    return await invoke<T>(command, args)
  } catch (error) {
    const apiError = normalise(error)
    console.error(`[ipc] ${command} failed`, {
      code: apiError.code,
      traceId: apiError.traceId,
    })
    throw apiError
  }
}

// Re-export mock validation utilities and state for testing and development environments
export {
  validateMockCategoria,
  validateMockCliente,
  validateMockEmpleado,
  validateMockFactura,
  validateMockFeriado,
  validateMockMovimiento,
  validateMockProyecto,
  validateMockTipoMovimiento,
  validateMockTrabajo,
} from './mock/validation'

export {
  DEFAULT_CONFIG,
  mockConfig,
  mockDb,
  loadMockConfig,
  saveMockConfig,
  loadMockDb,
  saveMockDb,
} from './mock/database'
