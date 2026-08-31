import { invoke } from '@tauri-apps/api/core'
import type { AppConfig } from './types'

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

/**
 * Detects whether the current execution context is inside the Tauri runtime with IPC bridge.
 */
export function isTauri(): boolean {
  return typeof window !== 'undefined' && ('__TAURI_INTERNALS__' in window || '__TAURI__' in window)
}

const DEFAULT_CONFIG: AppConfig = {
  application: {
    name: 'ElectroObra',
    environment: 'development',
    seedEnabled: true,
    lastPageSize: 30,
    theme: 'system',
    lastRoute: 'dashboard',
    sidebarExpanded: true,
    dataDir: null,
  },
  locale: {
    language: 'es',
    formatoFecha: 'dd/MM/yyyy',
    formatoFechaHora: 'dd/MM/yyyy HH:mm',
    primerDiaSemana: 1,
    simboloMoneda: '$',
    separadorMiles: '.',
    separadorDecimal: ',',
    decimalesMoneda: 2,
    decimalesPorcentaje: 2,
    monedaPorDefecto: 'ars',
    zonaHoraria: 'America/Argentina/Buenos_Aires',
  },
  business: {
    nombreComercial: 'ElectroObra',
    lema: 'Instalaciones Eléctricas',
    contratista: 'Pablo Báez',
    cuit: '20-12345678-9',
    direccion: 'Av. Principal 123',
    telefono: '+54 9 11 1234-5678',
    email: 'contacto@electroobra.com',
    logoPath: null,
    ivaSugerido: '21.0000',
    facturaDiasVencimientoDefault: 30,
    categoriaProfundidadMaxima: 3,
    diasPorFrecuencia: {
      diario: '1.0000',
      semanal: '6.0000',
      quincenal: '15.0000',
      mensual: '30.0000',
    },
  },
  settlement: {
    multiplicadorSabado: '1.5000',
    multiplicadorDomingo: '2.0000',
    multiplicadorFeriado: '2.0000',
    incluirSabado: false,
    incluirDomingo: false,
    incluirFeriado: false,
    periodoPorDefectoDias: 15,
    sincronizarFeriadosAlIniciar: true,
    aniosFeriadosASincronizar: 2,
  },
  dashboard: {
    lastPeriod: 'mensual',
    privacyMode: false,
    casasDolar: ['blue', 'oficial'],
    cotizacionPorDefecto: 'blue',
    topClientesCantidad: 5,
    topCategoriasCantidad: 5,
    ultimosMovimientosCantidad: 10,
    obrasRankingCantidad: 5,
    alertaCaidaIngresosPct: '20.0000',
  },
  externalApis: {
    dollarUrl: 'https://dolarapi.com/v1/dolares',
    holidayUrl: 'https://api.argentinadatos.com/v1/feriados',
    timeoutSeconds: 10,
    reintentos: 2,
    dollarAutoUpdate: true,
    dollarCacheMinutes: 60,
  },
  attachments: {
    maxSizeMb: 10,
    maxTotalMb: 1000,
    trashRetentionDays: 30,
    extensionesPermitidas: ['.pdf', '.png', '.jpg', '.jpeg', '.docx', '.xlsx'],
  },
  backup: {
    enabled: true,
    directory: '',
    retentionDays: 30,
    minimoAConservar: 3,
    maxAgeDays: 90,
  },
  communication: {
    emailCliente: 'systemDefault',
    gmailUrl: 'https://mail.google.com/mail/?view=cm&fs=1&to={to}&su={subject}&body={body}',
    outlookUrl: 'https://outlook.live.com/mail/0/deeplink/compose?to={to}&subject={subject}&body={body}',
    yahooUrl: 'https://compose.mail.yahoo.com/?to={to}&subj={subject}&body={body}',
    codigoPais: '+54',
    whatsAppTemplate: 'Hola {cliente}, le enviamos el comprobante.',
    whatsAppLiquidacionTemplate: 'Hola {empleado}, adjuntamos su liquidación del período.',
    emailLiquidacionAsunto: 'Liquidación de haberes - {periodo}',
  },
  logging: {
    level: 'debug',
    retentionDays: 30,
    consoleEnabled: true,
    filter: 'info',
  },
  validation: {
    fechaMinima: '2020-01-01',
    fechaFuturaMaxDias: 365,
  },
  report: {
    font: 'Helvetica',
    mostrarLogo: true,
    mostrarFirmas: true,
    pieDePagina: 'ElectroObra - Gestión de Obras',
  },
}

let mockConfig = structuredClone(DEFAULT_CONFIG)

function mockBrowserCommand<T>(command: string, args?: Record<string, unknown>): T {
  switch (command) {
    case 'app_is_ready':
      return true as T
    case 'ping':
      return `pong: ${String(args?.message ?? '')}` as T
    case 'app_info':
      return {
        name: mockConfig.application.name,
        version: '0.1.0',
        environment: 'development',
        dataDir: 'Browser Preview Mode',
      } as T
    case 'app_config':
    case 'config_get_all':
      return structuredClone(mockConfig) as T
    case 'config_set': {
      const cambios = (args?.cambios ?? {}) as Record<string, unknown>
      for (const [key, val] of Object.entries(cambios)) {
        const parts = key.split('.')
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        let target: any = mockConfig
        for (let i = 0; i < parts.length - 1; i++) {
          const part = parts[i]
          if (part) target = target[part]
        }
        if (target && parts.length > 0) {
          const last = parts[parts.length - 1]
          if (last) {
            try {
              target[last] =
                typeof val === 'string' && (val.startsWith('{') || val.startsWith('['))
                  ? JSON.parse(val)
                  : val
            } catch {
              target[last] = val
            }
          }
        }
      }
      return structuredClone(mockConfig) as T
    }
    case 'config_reset':
      mockConfig = structuredClone(DEFAULT_CONFIG)
      return structuredClone(mockConfig) as T
    case 'sistema_detect_legacy_db':
      return null as T
    case 'sistema_info':
      return {
        version: '0.1.0',
        baseSaludable: true,
        estado: 'Dashboard.Estado.Saludable',
        migraciones: 2,
        tamanoBytes: 524288,
      } as unknown as T
    case 'dev_seed_database':
      return {
        categorias: 8,
        tiposMovimiento: 3,
        empleados: 5,
        clientes: 4,
        obras: 4,
        trabajos: 5,
        ordenesTrabajo: 3,
        movimientos: 9,
        facturas: 3,
        liquidaciones: 2,
      } as unknown as T
    case 'backup_list':
      return [] as T
    case 'dashboard_stats':
      return {
        periodo: (args?.periodo as string) ?? 'Mensual',
        desde: new Date(new Date().getFullYear(), new Date().getMonth(), 1).toISOString(),
        hasta: new Date().toISOString(),
        totalIngresos: '0.0000',
        totalGastos: '0.0000',
        balance: '0.0000',
        cantidadMovimientos: 0,
        rentabilidad: '0.0000',
        anteriorIngresos: '0.0000',
        anteriorGastos: '0.0000',
        variacionIngresos: null,
        variacionGastos: null,
        variacionBalance: null,
        clientesActivos: 0,
        trabajosPendientes: 0,
        obrasPausadas: 0,
        facturasVencidas: 0,
        liquidacionesPendientes: 0,
        serieMensual: Array.from({ length: 12 }, (_, i) => ({
          mes: i + 1,
          ingresos: '0.0000',
          gastos: '0.0000',
        })),
        topClientes: [],
        gastosPorCategoria: [],
        mejoresObras: [],
        peoresObras: [],
        ultimosMovimientos: [],
        estadoSistema: {
          version: '0.1.0',
          baseSaludable: true,
          estado: 'Dashboard.EstadoOk',
          migraciones: 2,
          tamanoBytes: 524288,
        },
      } as unknown as T
    case 'dashboard_alertas':
      return [] as T
    case 'cotizaciones_list':
    case 'cotizaciones_get':
      return [
        {
          casa: 'blue',
          nombre: 'Dólar Blue',
          compra: '1280.0000',
          venta: '1300.0000',
          fechaActualizacion: new Date().toISOString(),
          esVieja: false,
        },
        {
          casa: 'oficial',
          nombre: 'Dólar Oficial',
          compra: '950.0000',
          venta: '990.0000',
          fechaActualizacion: new Date().toISOString(),
          esVieja: false,
        },
      ] as T
    case 'feriados_list':
      return [] as T
    case 'feriados_sync':
      return { agregados: 0, total: 0 } as T
    case 'tipos_movimiento_list':
    case 'categorias_list':
    case 'movimientos_list':
    case 'clientes_list':
    case 'obras_list':
    case 'trabajos_list':
    case 'facturas_list':
    case 'empleados_list':
    case 'liquidaciones_list':
    case 'certificados_list':
      return { items: [], totalCount: 0, page: 1, size: 30 } as T
    case 'tipos_movimiento_lookup':
    case 'categorias_lookup':
    case 'clientes_lookup':
    case 'obras_lookup':
    case 'trabajos_lookup':
    case 'empleados_lookup':
      return [] as T
    default:
      return null as unknown as T
  }
}

export async function callCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) {
    return Promise.resolve(mockBrowserCommand<T>(command, args))
  }
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
