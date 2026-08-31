import { invoke } from '@tauri-apps/api/core'
import type { AppConfig } from './types'

/**
 * The only module that talks to Tauri. See `docs/11-contratos-tauri.md` §4.1.
 *
 * In web preview mode (outside Tauri runtime), provides a rich in-memory mock database
 * that responds reactively to all entity CRUD, lists, lookups, and dev seeding.
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

// In-Memory Mock Database
interface MockCategory {
  id: string
  nombre: string
  descripcion: string | null
  colorHex: string | null
  icono: string | null
  categoriaPadreId: string | null
  categoriaPadreNombre: string | null
  nivel: number
  movimientosCount: number
  subcategoriasCount: number
  puedeEliminarse: boolean
  rowVersion: string
}

interface MockTipoMovimiento {
  id: string
  nombre: string
  descripcion: string | null
  esIngreso: boolean
  esSistema: boolean
  movimientosCount: number
  puedeEliminarse: boolean
  rowVersion: string
}

interface MockCliente {
  id: string
  nombre: string
  cuit: string | null
  direccion: string | null
  telefono: string | null
  email: string | null
  condicionIva: string | null
  obrasCount: number
  facturasCount: number
  deuda: string
  puedeEliminarse: boolean
  rowVersion: string
}

interface MockObra {
  id: string
  numero: number
  nombre: string
  direccion: string | null
  localidad: string | null
  clienteId: string
  clienteNombre: string
  estado: string
  trabajosCount: number
  rentabilidad: string
  puedeEliminarse: boolean
  rowVersion: string
}

interface MockTrabajo {
  id: string
  obraId: string
  obraNumero: number
  obraNombre: string
  clienteId: string
  clienteNombre: string
  descripcion: string
  fechaInicio: string
  fechaFin: string | null
  presupuesto: string
  estado: string
  rowVersion: string
}

interface MockOrden {
  id: string
  trabajoId: string
  titulo: string
  numeroCertificado: string | null
  fecha: string
  totalCertificados: number
  totalNeto: string
  rowVersion: string
}

interface MockCertificado {
  id: string
  ordenTrabajoId: string
  ordenTitulo: string
  numero: number
  fecha: string
  totalCertificado: string
  totalNeto: string
  rowVersion: string
}

interface MockEmpleado {
  id: string
  nombre: string
  dni: string | null
  cargo: string | null
  tarifaDiaria: string
  sueldoBase: string
  pagoFrecuencia: string
  email: string | null
  telefono: string | null
  fechaIngreso: string
  fechaEgreso: string | null
  activo: boolean
  rowVersion: string
}

interface MockFactura {
  id: string
  numero: string
  fecha: string
  fechaVencimiento: string | null
  clienteId: string
  clienteNombre: string
  estado: string
  subtotal: string
  iva: string
  total: string
  saldoPendiente: string
  rowVersion: string
}

interface MockMovimiento {
  id: string
  fecha: string
  concepto: string
  monto: string
  cantidad: string
  total: string
  moneda: string
  cotizacionAplicada: string | null
  tipoMovimientoId: string
  tipoMovimientoNombre: string
  esIngreso: boolean
  categoriaId: string | null
  categoriaNombre: string | null
  categoriaColor: string | null
  clienteId: string | null
  trabajoId: string | null
  empleadoId: string | null
  facturaId: string | null
  tipoConceptoPagoId: string | null
  bloqueadoPorLiquidacion: boolean
  rowVersion: string
  createdAt?: string
  updatedAt?: string | null
}

interface MockLiquidacion {
  id: string
  empleadoId: string
  empleadoNombre: string
  empleadoCargo: string | null
  fechaInicio: string
  fechaFin: string
  diasTrabajados: string
  tarifaAplicada: string
  totalBruto: string
  totalAdelantos: string
  totalNeto: string
  tienePdf: boolean
  rowVersion: string
}

interface MockFeriado {
  fecha: string
  nombre: string
  tipo: string | null
  origen: string
}

interface MockDb {
  categorias: MockCategory[]
  tiposMovimiento: MockTipoMovimiento[]
  clientes: MockCliente[]
  obras: MockObra[]
  trabajos: MockTrabajo[]
  ordenes: MockOrden[]
  certificados: MockCertificado[]
  empleados: MockEmpleado[]
  facturas: MockFactura[]
  movimientos: MockMovimiento[]
  liquidaciones: MockLiquidacion[]
  feriados: MockFeriado[]
}

function createSeedMockDb(): MockDb {
  const cat1 = { id: '10000000-0000-0000-0000-000000000001', nombre: 'Materiales Eléctricos', descripcion: 'Insumos eléctricos', colorHex: '#3B82F6', icono: 'package', categoriaPadreId: null, categoriaPadreNombre: null, nivel: 0, movimientosCount: 3, subcategoriasCount: 2, puedeEliminarse: false, rowVersion: 'v1' }
  const cat2 = { id: '10000000-0000-0000-0000-000000000002', nombre: 'Cables y Conductores', descripcion: 'Cables sintetizados y unipolar', colorHex: '#3B82F6', icono: 'layers', categoriaPadreId: cat1.id, categoriaPadreNombre: cat1.nombre, nivel: 1, movimientosCount: 1, subcategoriasCount: 0, puedeEliminarse: false, rowVersion: 'v1' }
  const cat3 = { id: '10000000-0000-0000-0000-000000000003', nombre: 'Herramientas y Equipos', descripcion: 'Herramientas de mano e instrumental', colorHex: '#F59E0B', icono: 'wrench', categoriaPadreId: null, categoriaPadreNombre: null, nivel: 0, movimientosCount: 1, subcategoriasCount: 0, puedeEliminarse: false, rowVersion: 'v1' }
  const cat4 = { id: '10000000-0000-0000-0000-000000000004', nombre: 'Servicios y Fletes', descripcion: 'Servicios de logística y traslados', colorHex: '#10B981', icono: 'briefcase', categoriaPadreId: null, categoriaPadreNombre: null, nivel: 0, movimientosCount: 2, subcategoriasCount: 0, puedeEliminarse: false, rowVersion: 'v1' }
  const cat5 = { id: '10000000-0000-0000-0000-000000000005', nombre: 'Impuestos y Tasas', descripcion: 'Monotributo, IIBB y cargas', colorHex: '#EF4444', icono: 'receipt', categoriaPadreId: null, categoriaPadreNombre: null, nivel: 0, movimientosCount: 1, subcategoriasCount: 0, puedeEliminarse: false, rowVersion: 'v1' }
  const cat6 = { id: '10000000-0000-0000-0000-000000000006', nombre: 'Viáticos y Combustible', descripcion: 'Combustible y peajes', colorHex: '#06B6D4', icono: 'truck', categoriaPadreId: null, categoriaPadreNombre: null, nivel: 0, movimientosCount: 1, subcategoriasCount: 0, puedeEliminarse: false, rowVersion: 'v1' }
  const categorias: MockCategory[] = [cat1, cat2, cat3, cat4, cat5, cat6]

  const tipoIngreso = { id: '00000000-0000-0000-0000-000000000001', nombre: 'Ingreso', descripcion: 'Ingreso estándar del sistema', esIngreso: true, esSistema: true, movimientosCount: 2, puedeEliminarse: false, rowVersion: 'v1' }
  const tipoGasto = { id: '00000000-0000-0000-0000-000000000002', nombre: 'Gasto', descripcion: 'Gasto operativo', esIngreso: false, esSistema: true, movimientosCount: 4, puedeEliminarse: false, rowVersion: 'v1' }
  const tipoAdelanto = { id: '00000000-0000-0000-0000-000000000003', nombre: 'Adelanto', descripcion: 'Adelanto de sueldo a personal', esIngreso: false, esSistema: true, movimientosCount: 2, puedeEliminarse: false, rowVersion: 'v1' }
  const tipoAjuste = { id: '00000000-0000-0000-0000-000000000004', nombre: 'Ajuste', descripcion: 'Ajuste contable', esIngreso: true, esSistema: true, movimientosCount: 0, puedeEliminarse: false, rowVersion: 'v1' }
  const tipoChatarra = { id: '20000000-0000-0000-0000-000000000001', nombre: 'Venta de chatarra / sobrantes', descripcion: 'Ventas accesorias', esIngreso: true, esSistema: false, movimientosCount: 1, puedeEliminarse: false, rowVersion: 'v1' }
  const tiposMovimiento: MockTipoMovimiento[] = [tipoIngreso, tipoGasto, tipoAdelanto, tipoAjuste, tipoChatarra]

  const cli1 = { id: '30000000-0000-0000-0000-000000000001', nombre: 'Constructora del Plata S.A.', cuit: '30-71234567-9', direccion: 'Av. del Libertador 1234, CABA', telefono: '011-4567-8900', email: 'info@constructoradelplata.com', condicionIva: 'Responsable Inscripto', obrasCount: 1, facturasCount: 1, deuda: '0.0000', puedeEliminarse: false, rowVersion: 'v1' }
  const cli2 = { id: '30000000-0000-0000-0000-000000000002', nombre: 'Desarrollos Urbanos SRL', cuit: '30-79876543-1', direccion: 'San Martín 567, Rosario', telefono: '0341-423-4567', email: 'admin@desarrollosurbanos.com', condicionIva: 'Responsable Inscripto', obrasCount: 1, facturasCount: 1, deuda: '7502.0000', puedeEliminarse: false, rowVersion: 'v1' }
  const cli3 = { id: '30000000-0000-0000-0000-000000000003', nombre: 'Consorcio Torre Alvear', cuit: '30-65432109-8', direccion: 'Av. Alvear 1890, CABA', telefono: '011-4812-3456', email: 'consorcio@torrealvear.com', condicionIva: 'Consumidor Final', obrasCount: 1, facturasCount: 1, deuda: '0.0000', puedeEliminarse: false, rowVersion: 'v1' }
  const cli4 = { id: '30000000-0000-0000-0000-000000000004', nombre: 'Juan Carlos Pérez', cuit: '20-28123456-3', direccion: 'Belgrano 432, San Isidro', telefono: '011-15-5432-1098', email: 'jcperez@gmail.com', condicionIva: 'Consumidor Final', obrasCount: 1, facturasCount: 0, deuda: '0.0000', puedeEliminarse: false, rowVersion: 'v1' }
  const clientes: MockCliente[] = [cli1, cli2, cli3, cli4]

  const obra1 = { id: '40000000-0000-0000-0000-000000000001', numero: 1, nombre: 'Instalación Eléctrica Integral Torre Alvear', direccion: 'Av. Alvear 1890', localidad: 'CABA', clienteId: cli3.id, clienteNombre: cli3.nombre, estado: 'Activa', trabajosCount: 2, rentabilidad: '11120.0000', puedeEliminarse: false, rowVersion: 'v1' }
  const obra2 = { id: '40000000-0000-0000-0000-000000000002', numero: 2, nombre: 'Iluminación y Fuerza Motriz Planta del Plata', direccion: 'Parque Industrial Norte', localidad: 'Tigre', clienteId: cli1.id, clienteNombre: cli1.nombre, estado: 'Activa', trabajosCount: 1, rentabilidad: '4550.0000', puedeEliminarse: false, rowVersion: 'v1' }
  const obra3 = { id: '40000000-0000-0000-0000-000000000003', numero: 3, nombre: 'Cableado Estructurado Oficinas Centro', direccion: 'San Martín 567', localidad: 'Rosario', clienteId: cli2.id, clienteNombre: cli2.nombre, estado: 'Finalizada', trabajosCount: 1, rentabilidad: '14000.0000', puedeEliminarse: false, rowVersion: 'v1' }
  const obra4 = { id: '40000000-0000-0000-0000-000000000004', numero: 4, nombre: 'Refacción y Tablero Eléctrico Domiciliario', direccion: 'Belgrano 432', localidad: 'San Isidro', clienteId: cli4.id, clienteNombre: cli4.nombre, estado: 'Activa', trabajosCount: 1, rentabilidad: '6500.0000', puedeEliminarse: false, rowVersion: 'v1' }
  const obras: MockObra[] = [obra1, obra2, obra3, obra4]

  const trab1 = { id: '50000000-0000-0000-0000-000000000001', obraId: obra1.id, obraNumero: 1, obraNombre: obra1.nombre, clienteId: cli3.id, clienteNombre: cli3.nombre, descripcion: 'Tendido de bandejas portacables en subsuelos', fechaInicio: '2025-02-01', fechaFin: null, presupuesto: '1850000.0000', estado: 'EnProceso', rowVersion: 'v1' }
  const trab2 = { id: '50000000-0000-0000-0000-000000000002', obraId: obra1.id, obraNumero: 1, obraNombre: obra1.nombre, clienteId: cli3.id, clienteNombre: cli3.nombre, descripcion: 'Montaje de tableros seccionales por piso', fechaInicio: '2025-02-10', fechaFin: null, presupuesto: '3200000.0000', estado: 'EnProceso', rowVersion: 'v1' }
  const trab3 = { id: '50000000-0000-0000-0000-000000000003', obraId: obra2.id, obraNumero: 2, obraNombre: obra2.nombre, clienteId: cli1.id, clienteNombre: cli1.nombre, descripcion: 'Iluminación perimetral LED alta potencia', fechaInicio: '2025-01-20', fechaFin: '2025-02-25', presupuesto: '950000.0000', estado: 'Finalizado', rowVersion: 'v1' }
  const trab4 = { id: '50000000-0000-0000-0000-000000000004', obraId: obra3.id, obraNumero: 3, obraNombre: obra3.nombre, clienteId: cli2.id, clienteNombre: cli2.nombre, descripcion: 'Puestos de red Cat6 y rack central', fechaInicio: '2025-01-10', fechaFin: '2025-02-20', presupuesto: '1400000.0000', estado: 'Finalizado', rowVersion: 'v1' }
  const trab5 = { id: '50000000-0000-0000-0000-000000000005', obraId: obra4.id, obraNumero: 4, obraNombre: obra4.nombre, clienteId: cli4.id, clienteNombre: cli4.nombre, descripcion: 'Recableado completo y disyuntor diferencial', fechaInicio: '2025-02-15', fechaFin: null, presupuesto: '650000.0000', estado: 'EnProceso', rowVersion: 'v1' }
  const trabajos: MockTrabajo[] = [trab1, trab2, trab3, trab4, trab5]

  const ord1 = { id: '60000000-0000-0000-0000-000000000001', trabajoId: trab1.id, titulo: 'Certificación de Avance Etapa 1', numeroCertificado: 'CERT-001', fecha: '2025-02-20', totalCertificados: 1, totalNeto: '1998000.0000', rowVersion: 'v1' }
  const ord2 = { id: '60000000-0000-0000-0000-000000000002', trabajoId: trab2.id, titulo: 'Certificación de Avance Etapa 2', numeroCertificado: 'CERT-002', fecha: '2025-02-22', totalCertificados: 1, totalNeto: '3200000.0000', rowVersion: 'v1' }
  const ordenes: MockOrden[] = [ord1, ord2]

  const cert1 = { id: '70000000-0000-0000-0000-000000000001', ordenTrabajoId: ord1.id, ordenTitulo: ord1.titulo, numero: 1, fecha: '2025-02-22', totalCertificado: '1850000.0000', totalNeto: '1998000.0000', rowVersion: 'v1' }
  const certificados: MockCertificado[] = [cert1]

  const emp1 = { id: '80000000-0000-0000-0000-000000000001', nombre: 'Ricardo Darín', dni: '20.123.456', cargo: 'Operario Electricista', tarifaDiaria: '45000.0000', sueldoBase: '450000.0000', pagoFrecuencia: 'Quincenal', email: 'ricardo.darin@obra.com', telefono: '1145678901', fechaIngreso: '2025-01-15', fechaEgreso: null, activo: true, rowVersion: 'v1' }
  const emp2 = { id: '80000000-0000-0000-0000-000000000002', nombre: 'Guillermo Francella', dni: '22.345.678', cargo: 'Capataz de Obra', tarifaDiaria: '55000.0000', sueldoBase: '550000.0000', pagoFrecuencia: 'Quincenal', email: 'guillermo.francella@obra.com', telefono: '1145678902', fechaIngreso: '2025-01-15', fechaEgreso: null, activo: true, rowVersion: 'v1' }
  const emp3 = { id: '80000000-0000-0000-0000-000000000003', nombre: 'Natalia Oreiro', dni: '25.678.901', cargo: 'Técnica Instaladora', tarifaDiaria: '48000.0000', sueldoBase: '480000.0000', pagoFrecuencia: 'Quincenal', email: 'natalia.oreiro@obra.com', telefono: '1145678903', fechaIngreso: '2025-01-15', fechaEgreso: null, activo: true, rowVersion: 'v1' }
  const emp4 = { id: '80000000-0000-0000-0000-000000000004', nombre: 'Diego Peretti', dni: '18.901.234', cargo: 'Ayudante Práctico', tarifaDiaria: '38000.0000', sueldoBase: '380000.0000', pagoFrecuencia: 'Quincenal', email: 'diego.peretti@obra.com', telefono: '1145678904', fechaIngreso: '2025-01-15', fechaEgreso: null, activo: true, rowVersion: 'v1' }
  const emp5 = { id: '80000000-0000-0000-0000-000000000005', nombre: 'Érica Rivas', dni: '27.234.567', cargo: 'Administrativa de Obra', tarifaDiaria: '42000.0000', sueldoBase: '420000.0000', pagoFrecuencia: 'Quincenal', email: 'erica.rivas@obra.com', telefono: '1145678905', fechaIngreso: '2025-01-15', fechaEgreso: null, activo: true, rowVersion: 'v1' }
  const empleados: MockEmpleado[] = [emp1, emp2, emp3, emp4, emp5]

  const fact1 = { id: '90000000-0000-0000-0000-000000000001', numero: '0001-00000101', fecha: '2025-02-10', fechaVencimiento: '2025-03-10', clienteId: cli1.id, clienteNombre: cli1.nombre, estado: 'Emitida', subtotal: '850000.0000', iva: '178500.0000', total: '1028500.0000', saldoPendiente: '1028500.0000', rowVersion: 'v1' }
  const fact2 = { id: '90000000-0000-0000-0000-000000000002', numero: '0001-00000102', fecha: '2025-02-10', fechaVencimiento: '2025-03-10', clienteId: cli3.id, clienteNombre: cli3.nombre, estado: 'Pagada', subtotal: '1200000.0000', iva: '252000.0000', total: '1452000.0000', saldoPendiente: '0.0000', rowVersion: 'v1' }
  const fact3 = { id: '90000000-0000-0000-0000-000000000003', numero: '0001-00000103', fecha: '2025-02-15', fechaVencimiento: '2025-03-15', clienteId: cli2.id, clienteNombre: cli2.nombre, estado: 'Borrador', subtotal: '620000.0000', iva: '130200.0000', total: '750200.0000', saldoPendiente: '750200.0000', rowVersion: 'v1' }
  const facturas: MockFactura[] = [fact1, fact2, fact3]

  const mov1: MockMovimiento = { id: 'a0000000-0000-0000-0000-000000000001', fecha: '2025-02-18T14:30:00Z', concepto: 'Cobro Certificado N.º 1 Torre Alvear', monto: '1452000.0000', cantidad: '1.0000', total: '1452000.0000', moneda: 'Ars', cotizacionAplicada: null, tipoMovimientoId: tipoIngreso.id, tipoMovimientoNombre: tipoIngreso.nombre, esIngreso: true, categoriaId: cat4.id, categoriaNombre: cat4.nombre, categoriaColor: cat4.colorHex, clienteId: cli3.id, trabajoId: trab1.id, empleadoId: null, facturaId: fact2.id, tipoConceptoPagoId: null, bloqueadoPorLiquidacion: false, rowVersion: 'v1', createdAt: '2025-02-18T14:30:00Z', updatedAt: null }
  const mov2: MockMovimiento = { id: 'a0000000-0000-0000-0000-000000000002', fecha: '2025-02-17T11:00:00Z', concepto: 'Anticipo Obra Planta del Plata', monto: '500000.0000', cantidad: '1.0000', total: '500000.0000', moneda: 'Ars', cotizacionAplicada: null, tipoMovimientoId: tipoIngreso.id, tipoMovimientoNombre: tipoIngreso.nombre, esIngreso: true, categoriaId: cat4.id, categoriaNombre: cat4.nombre, categoriaColor: cat4.colorHex, clienteId: cli1.id, trabajoId: trab3.id, empleadoId: null, facturaId: null, tipoConceptoPagoId: null, bloqueadoPorLiquidacion: false, rowVersion: 'v1', createdAt: '2025-02-17T11:00:00Z', updatedAt: null }
  const mov3: MockMovimiento = { id: 'a0000000-0000-0000-0000-000000000003', fecha: '2025-02-16T16:00:00Z', concepto: 'Venta de cables sobrantes de cobre', monto: '85000.0000', cantidad: '1.0000', total: '85000.0000', moneda: 'Ars', cotizacionAplicada: null, tipoMovimientoId: tipoChatarra.id, tipoMovimientoNombre: tipoChatarra.nombre, esIngreso: true, categoriaId: cat1.id, categoriaNombre: cat1.nombre, categoriaColor: cat1.colorHex, clienteId: null, trabajoId: null, empleadoId: null, facturaId: null, tipoConceptoPagoId: null, bloqueadoPorLiquidacion: false, rowVersion: 'v1', createdAt: '2025-02-16T16:00:00Z', updatedAt: null }
  const mov4: MockMovimiento = { id: 'a0000000-0000-0000-0000-000000000004', fecha: '2025-02-15T10:00:00Z', concepto: 'Compra de cables sintetizados y termomagnéticas', monto: '340000.0000', cantidad: '1.0000', total: '340000.0000', moneda: 'Ars', cotizacionAplicada: null, tipoMovimientoId: tipoGasto.id, tipoMovimientoNombre: tipoGasto.nombre, esIngreso: false, categoriaId: cat2.id, categoriaNombre: cat2.nombre, categoriaColor: cat2.colorHex, clienteId: null, trabajoId: trab1.id, empleadoId: null, facturaId: null, tipoConceptoPagoId: null, bloqueadoPorLiquidacion: false, rowVersion: 'v1', createdAt: '2025-02-15T10:00:00Z', updatedAt: null }
  const mov5: MockMovimiento = { id: 'a0000000-0000-0000-0000-000000000005', fecha: '2025-02-14T09:30:00Z', concepto: 'Adquisición de pinza amperimétrica True RMS', monto: '125000.0000', cantidad: '1.0000', total: '125000.0000', moneda: 'Ars', cotizacionAplicada: null, tipoMovimientoId: tipoGasto.id, tipoMovimientoNombre: tipoGasto.nombre, esIngreso: false, categoriaId: cat3.id, categoriaNombre: cat3.nombre, categoriaColor: cat3.colorHex, clienteId: null, trabajoId: null, empleadoId: null, facturaId: null, tipoConceptoPagoId: null, bloqueadoPorLiquidacion: false, rowVersion: 'v1', createdAt: '2025-02-14T09:30:00Z', updatedAt: null }
  const mov6: MockMovimiento = { id: 'a0000000-0000-0000-0000-000000000006', fecha: '2025-02-13T12:00:00Z', concepto: 'Combustible y peajes traslados a Tigre', monto: '45000.0000', cantidad: '1.0000', total: '45000.0000', moneda: 'Ars', cotizacionAplicada: null, tipoMovimientoId: tipoGasto.id, tipoMovimientoNombre: tipoGasto.nombre, esIngreso: false, categoriaId: cat6.id, categoriaNombre: cat6.nombre, categoriaColor: cat6.colorHex, clienteId: null, trabajoId: trab3.id, empleadoId: null, facturaId: null, tipoConceptoPagoId: null, bloqueadoPorLiquidacion: false, rowVersion: 'v1', createdAt: '2025-02-13T12:00:00Z', updatedAt: null }
  const mov7: MockMovimiento = { id: 'a0000000-0000-0000-0000-000000000007', fecha: '2025-02-12T15:00:00Z', concepto: 'Pago de Monotributo / IIBB mensual', monto: '62000.0000', cantidad: '1.0000', total: '62000.0000', moneda: 'Ars', cotizacionAplicada: null, tipoMovimientoId: tipoGasto.id, tipoMovimientoNombre: tipoGasto.nombre, esIngreso: false, categoriaId: cat5.id, categoriaNombre: cat5.nombre, categoriaColor: cat5.colorHex, clienteId: null, trabajoId: null, empleadoId: null, facturaId: null, tipoConceptoPagoId: null, bloqueadoPorLiquidacion: false, rowVersion: 'v1', createdAt: '2025-02-12T15:00:00Z', updatedAt: null }
  const mov8: MockMovimiento = { id: 'a0000000-0000-0000-0000-000000000008', fecha: '2025-02-10T17:00:00Z', concepto: 'Adelanto quincenal Ricardo Darín', monto: '50000.0000', cantidad: '1.0000', total: '50000.0000', moneda: 'Ars', cotizacionAplicada: null, tipoMovimientoId: tipoAdelanto.id, tipoMovimientoNombre: tipoAdelanto.nombre, esIngreso: false, categoriaId: null, categoriaNombre: null, categoriaColor: null, clienteId: null, trabajoId: null, empleadoId: emp1.id, facturaId: null, tipoConceptoPagoId: null, bloqueadoPorLiquidacion: true, rowVersion: 'v1', createdAt: '2025-02-10T17:00:00Z', updatedAt: null }
  const mov9: MockMovimiento = { id: 'a0000000-0000-0000-0000-000000000009', fecha: '2025-02-10T17:15:00Z', concepto: 'Adelanto quincenal Natalia Oreiro', monto: '40000.0000', cantidad: '1.0000', total: '40000.0000', moneda: 'Ars', cotizacionAplicada: null, tipoMovimientoId: tipoAdelanto.id, tipoMovimientoNombre: tipoAdelanto.nombre, esIngreso: false, categoriaId: null, categoriaNombre: null, categoriaColor: null, clienteId: null, trabajoId: null, empleadoId: emp3.id, facturaId: null, tipoConceptoPagoId: null, bloqueadoPorLiquidacion: false, rowVersion: 'v1', createdAt: '2025-02-10T17:15:00Z', updatedAt: null }
  const movimientos: MockMovimiento[] = [mov1, mov2, mov3, mov4, mov5, mov6, mov7, mov8, mov9]

  const liq1 = { id: 'b0000000-0000-0000-0000-000000000001', empleadoId: emp1.id, empleadoNombre: emp1.nombre, empleadoCargo: emp1.cargo, fechaInicio: '2025-02-01', fechaFin: '2025-02-15', diasTrabajados: '11.0000', tarifaAplicada: '45000.0000', totalBruto: '495000.0000', totalAdelantos: '50000.0000', totalNeto: '445000.0000', tienePdf: false, rowVersion: 'v1' }
  const liq2 = { id: 'b0000000-0000-0000-0000-000000000002', empleadoId: emp2.id, empleadoNombre: emp2.nombre, empleadoCargo: emp2.cargo, fechaInicio: '2025-02-01', fechaFin: '2025-02-15', diasTrabajados: '11.0000', tarifaAplicada: '55000.0000', totalBruto: '605000.0000', totalAdelantos: '0.0000', totalNeto: '605000.0000', tienePdf: false, rowVersion: 'v1' }
  const liquidaciones: MockLiquidacion[] = [liq1, liq2]

  const feriados: MockFeriado[] = [
    { fecha: '2025-01-01', nombre: 'Año Nuevo', tipo: 'Inamovible', origen: 'Api' },
    { fecha: '2025-03-03', nombre: 'Carnaval', tipo: 'Inamovible', origen: 'Api' },
    { fecha: '2025-03-04', nombre: 'Carnaval', tipo: 'Inamovible', origen: 'Api' },
    { fecha: '2025-03-24', nombre: 'Día Nacional de la Memoria por la Verdad y la Justicia', tipo: 'Inamovible', origen: 'Api' },
    { fecha: '2025-04-02', nombre: 'Día del Veterano y de los Caídos en la Guerra de Malvinas', tipo: 'Inamovible', origen: 'Api' },
    { fecha: '2025-05-01', nombre: 'Día del Trabajador', tipo: 'Inamovible', origen: 'Api' },
    { fecha: '2025-05-25', nombre: 'Día de la Revolución de Mayo', tipo: 'Inamovible', origen: 'Api' },
    { fecha: '2025-07-09', nombre: 'Día de la Independencia', tipo: 'Inamovible', origen: 'Api' },
    { fecha: '2025-12-25', nombre: 'Navidad', tipo: 'Inamovible', origen: 'Api' },
  ]

  return {
    categorias,
    tiposMovimiento,
    clientes,
    obras,
    trabajos,
    ordenes,
    certificados,
    empleados,
    facturas,
    movimientos,
    liquidaciones,
    feriados,
  }
}

let mockDb: MockDb = createSeedMockDb()

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
      mockDb = createSeedMockDb()
      return {
        categorias: mockDb.categorias.length,
        tiposMovimiento: mockDb.tiposMovimiento.length,
        empleados: mockDb.empleados.length,
        clientes: mockDb.clientes.length,
        obras: mockDb.obras.length,
        trabajos: mockDb.trabajos.length,
        ordenesTrabajo: mockDb.ordenes.length,
        movimientos: mockDb.movimientos.length,
        facturas: mockDb.facturas.length,
        liquidaciones: mockDb.liquidaciones.length,
      } as unknown as T
    case 'backup_list':
      return [] as T
    case 'dashboard_stats':
    case 'dashboard_kpis':
      return {
        periodo: (args?.periodo as string) ?? 'Mensual',
        desde: new Date(new Date().getFullYear(), new Date().getMonth(), 1).toISOString(),
        hasta: new Date().toISOString(),
        totalIngresos: '2037000.0000',
        totalGastos: '662000.0000',
        balance: '1375000.0000',
        cantidadMovimientos: mockDb.movimientos.length,
        rentabilidad: '67.5000',
        anteriorIngresos: '1800000.0000',
        anteriorGastos: '550000.0000',
        variacionIngresos: '13.1600',
        variacionGastos: '20.3600',
        variacionBalance: '10.0000',
        clientesActivos: mockDb.clientes.length,
        trabajosPendientes: 3,
        obrasPausadas: 0,
        facturasVencidas: 0,
        liquidacionesPendientes: 1,
        serieMensual: Array.from({ length: 12 }, (_, i) => ({
          mes: i + 1,
          ingresos: i === new Date().getMonth() ? '2037000.0000' : '1500000.0000',
          gastos: i === new Date().getMonth() ? '662000.0000' : '450000.0000',
        })),
        topClientes: [
          { id: mockDb.clientes[2]?.id ?? '', nombre: mockDb.clientes[2]?.nombre ?? '', total: '1452000.0000' },
          { id: mockDb.clientes[0]?.id ?? '', nombre: mockDb.clientes[0]?.nombre ?? '', total: '500000.0000' },
          { id: mockDb.clientes[1]?.id ?? '', nombre: mockDb.clientes[1]?.nombre ?? '', total: '85000.0000' },
        ],
        gastosPorCategoria: [
          { id: mockDb.categorias[1]?.id ?? '', nombre: mockDb.categorias[1]?.nombre ?? '', colorHex: mockDb.categorias[1]?.colorHex ?? '#3B82F6', total: '340000.0000', porcentaje: '51.3600' },
          { id: mockDb.categorias[2]?.id ?? '', nombre: mockDb.categorias[2]?.nombre ?? '', colorHex: mockDb.categorias[2]?.colorHex ?? '#F59E0B', total: '125000.0000', porcentaje: '18.8800' },
          { id: mockDb.categorias[4]?.id ?? '', nombre: mockDb.categorias[4]?.nombre ?? '', colorHex: mockDb.categorias[4]?.colorHex ?? '#EF4444', total: '62000.0000', porcentaje: '9.3600' },
        ],
        mejoresObras: [
          { id: mockDb.obras[0]?.id ?? '', numero: 1, nombre: mockDb.obras[0]?.nombre ?? '', rentabilidad: '1112000.0000', margen: '76.5800' },
        ],
        peoresObras: [],
        ultimosMovimientos: mockDb.movimientos.slice(0, 5),
        estadoSistema: {
          version: '0.1.0',
          baseSaludable: true,
          estado: 'Dashboard.EstadoOk',
          migraciones: 2,
          tamanoBytes: 524288,
        },
      } as unknown as T
    case 'dashboard_serie_mensual':
      return Array.from({ length: 12 }, (_, i) => ({
        mes: i + 1,
        ingresos: i === new Date().getMonth() ? '2037000.0000' : '1500000.0000',
        gastos: i === new Date().getMonth() ? '662000.0000' : '450000.0000',
      })) as T
    case 'dashboard_top_clientes':
      return [
        { id: mockDb.clientes[2]?.id ?? '', nombre: mockDb.clientes[2]?.nombre ?? '', total: '1452000.0000' },
        { id: mockDb.clientes[0]?.id ?? '', nombre: mockDb.clientes[0]?.nombre ?? '', total: '500000.0000' },
        { id: mockDb.clientes[1]?.id ?? '', nombre: mockDb.clientes[1]?.nombre ?? '', total: '85000.0000' },
      ] as T
    case 'dashboard_gastos_categorias':
      return [
        { id: mockDb.categorias[1]?.id ?? '', nombre: mockDb.categorias[1]?.nombre ?? '', colorHex: mockDb.categorias[1]?.colorHex ?? '#3B82F6', total: '340000.0000', porcentaje: '51.3600' },
        { id: mockDb.categorias[2]?.id ?? '', nombre: mockDb.categorias[2]?.nombre ?? '', colorHex: mockDb.categorias[2]?.colorHex ?? '#F59E0B', total: '125000.0000', porcentaje: '18.8800' },
        { id: mockDb.categorias[4]?.id ?? '', nombre: mockDb.categorias[4]?.nombre ?? '', colorHex: mockDb.categorias[4]?.colorHex ?? '#EF4444', total: '62000.0000', porcentaje: '9.3600' },
      ] as T
    case 'dashboard_rentabilidad_obras':
      return [
        { id: mockDb.obras[0]?.id ?? '', numero: 1, nombre: mockDb.obras[0]?.nombre ?? '', rentabilidad: '1112000.0000', margen: '76.5800' },
      ] as T
    case 'dashboard_ultimos_movimientos':
      return mockDb.movimientos.slice(0, 10) as T
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
      return mockDb.feriados as T
    case 'feriados_sync':
      return { agregados: 0, total: mockDb.feriados.length, aniosConError: 0 } as T

    // ==========================================
    // MOVIMIENTOS
    // ==========================================
    case 'movimientos_list': {
      let totalIngresosNum = 0
      let totalGastosNum = 0
      for (const m of mockDb.movimientos) {
        const val = parseFloat(m.total) || 0
        if (m.esIngreso) totalIngresosNum += val
        else totalGastosNum += val
      }
      const resumen = {
        totalIngresos: totalIngresosNum.toFixed(4),
        totalGastos: totalGastosNum.toFixed(4),
        balance: (totalIngresosNum - totalGastosNum).toFixed(4),
        cantidad: mockDb.movimientos.length,
      }
      return { items: mockDb.movimientos, totalCount: mockDb.movimientos.length, page: 1, size: 30, resumen } as T
    }
    case 'movimientos_resumen':
    case 'movimiento_resumen': {
      let totalIngresosNum = 0
      let totalGastosNum = 0
      for (const m of mockDb.movimientos) {
        const val = parseFloat(m.total) || 0
        if (m.esIngreso) totalIngresosNum += val
        else totalGastosNum += val
      }
      return {
        totalIngresos: totalIngresosNum.toFixed(4),
        totalGastos: totalGastosNum.toFixed(4),
        balance: (totalIngresosNum - totalGastosNum).toFixed(4),
        cantidad: mockDb.movimientos.length,
      } as T
    }
    case 'movimientos_get':
    case 'movimiento_get': {
      const id = String(args?.id ?? '')
      const found = mockDb.movimientos.find(m => m.id === id) || mockDb.movimientos[0]
      return {
        ...found,
        createdAt: found?.createdAt ?? new Date().toISOString(),
        updatedAt: found?.updatedAt ?? null,
      } as T
    }
    case 'movimientos_create':
    case 'movimiento_create': {
      const dto = (args?.dto ?? {}) as Record<string, unknown>
      const tipo = mockDb.tiposMovimiento.find(t => t.id === dto.tipoMovimientoId)
      const cat = mockDb.categorias.find(c => c.id === dto.categoriaId)
      const montoNum = parseFloat(String(dto.monto || '0'))
      const cantNum = parseFloat(String(dto.cantidad || '1'))
      const total = (montoNum * cantNum).toFixed(4)
      const newMov: MockMovimiento = {
        id: crypto.randomUUID(),
        fecha: String(dto.fecha || new Date().toISOString()),
        concepto: String(dto.concepto || ''),
        monto: String(dto.monto || '0.0000'),
        cantidad: String(dto.cantidad || '1.0000'),
        total,
        moneda: String(dto.moneda || 'Ars'),
        cotizacionAplicada: (dto.cotizacionAplicada as string | null) ?? null,
        tipoMovimientoId: String(dto.tipoMovimientoId || ''),
        tipoMovimientoNombre: tipo?.nombre ?? 'General',
        esIngreso: tipo?.esIngreso ?? true,
        categoriaId: (dto.categoriaId as string | null) ?? null,
        categoriaNombre: cat?.nombre ?? null,
        categoriaColor: cat?.colorHex ?? null,
        clienteId: (dto.clienteId as string | null) ?? null,
        trabajoId: (dto.trabajoId as string | null) ?? null,
        empleadoId: (dto.empleadoId as string | null) ?? null,
        facturaId: (dto.facturaId as string | null) ?? null,
        tipoConceptoPagoId: (dto.tipoConceptoPagoId as string | null) ?? null,
        bloqueadoPorLiquidacion: false,
        rowVersion: crypto.randomUUID(),
        createdAt: new Date().toISOString(),
        updatedAt: null,
      }
      mockDb.movimientos.unshift(newMov)
      return newMov as T
    }
    case 'movimientos_update':
    case 'movimiento_update': {
      const id = String(args?.id ?? '')
      const dto = (args?.dto ?? {}) as Record<string, unknown>
      const idx = mockDb.movimientos.findIndex(m => m.id === id)
      if (idx >= 0) {
        const tipo = mockDb.tiposMovimiento.find(t => t.id === dto.tipoMovimientoId)
        const cat = mockDb.categorias.find(c => c.id === dto.categoriaId)
        const montoNum = parseFloat(String(dto.monto || '0'))
        const cantNum = parseFloat(String(dto.cantidad || '1'))
        const total = (montoNum * cantNum).toFixed(4)
        mockDb.movimientos[idx] = {
          ...mockDb.movimientos[idx]!,
          fecha: String(dto.fecha || mockDb.movimientos[idx]!.fecha),
          concepto: String(dto.concepto || mockDb.movimientos[idx]!.concepto),
          monto: String(dto.monto || mockDb.movimientos[idx]!.monto),
          cantidad: String(dto.cantidad || mockDb.movimientos[idx]!.cantidad),
          total,
          moneda: String(dto.moneda || mockDb.movimientos[idx]!.moneda),
          cotizacionAplicada: (dto.cotizacionAplicada as string | null) ?? null,
          tipoMovimientoId: String(dto.tipoMovimientoId || mockDb.movimientos[idx]!.tipoMovimientoId),
          tipoMovimientoNombre: tipo?.nombre ?? mockDb.movimientos[idx]!.tipoMovimientoNombre,
          esIngreso: tipo?.esIngreso ?? mockDb.movimientos[idx]!.esIngreso,
          categoriaId: (dto.categoriaId as string | null) ?? null,
          categoriaNombre: cat?.nombre ?? null,
          categoriaColor: cat?.colorHex ?? null,
          clienteId: (dto.clienteId as string | null) ?? null,
          trabajoId: (dto.trabajoId as string | null) ?? null,
          empleadoId: (dto.empleadoId as string | null) ?? null,
          facturaId: (dto.facturaId as string | null) ?? null,
          tipoConceptoPagoId: (dto.tipoConceptoPagoId as string | null) ?? null,
          rowVersion: crypto.randomUUID(),
          updatedAt: new Date().toISOString(),
        }
        return mockDb.movimientos[idx] as T
      }
      return mockDb.movimientos[0] as T
    }
    case 'movimientos_delete':
    case 'movimiento_delete': {
      const id = String(args?.id ?? '')
      mockDb.movimientos = mockDb.movimientos.filter(m => m.id !== id)
      return null as T
    }

    // ==========================================
    // CLIENTES
    // ==========================================
    case 'clientes_list':
      return { items: mockDb.clientes, totalCount: mockDb.clientes.length, page: 1, size: 30 } as T
    case 'clientes_lookup':
      return mockDb.clientes.map(c => ({ id: c.id, label: c.nombre })) as T
    case 'clientes_get':
    case 'cliente_get': {
      const id = String(args?.id ?? '')
      const cli = mockDb.clientes.find(c => c.id === id) || mockDb.clientes[0]!
      return {
        ...cli,
        contactos: [
          { id: crypto.randomUUID(), etiqueta: 'Administración', email: cli.email ?? '', nombre: cli.nombre, telefono: cli.telefono, esPrincipal: true },
        ],
        createdAt: new Date().toISOString(),
        updatedAt: null,
      } as T
    }
    case 'clientes_create':
    case 'cliente_create': {
      const dto = (args?.dto ?? {}) as Record<string, unknown>
      const newCli: MockCliente = {
        id: crypto.randomUUID(),
        nombre: String(dto.nombre || ''),
        cuit: (dto.cuit as string | null) ?? null,
        direccion: (dto.direccion as string | null) ?? null,
        telefono: (dto.telefono as string | null) ?? null,
        email: (dto.email as string | null) ?? null,
        condicionIva: (dto.condicionIva as string | null) ?? 'Responsable Inscripto',
        obrasCount: 0,
        facturasCount: 0,
        deuda: '0.0000',
        puedeEliminarse: true,
        rowVersion: crypto.randomUUID(),
      }
      mockDb.clientes.unshift(newCli)
      return newCli as T
    }
    case 'clientes_update':
    case 'cliente_update': {
      const id = String(args?.id ?? '')
      const dto = (args?.dto ?? {}) as Record<string, unknown>
      const idx = mockDb.clientes.findIndex(c => c.id === id)
      if (idx >= 0) {
        mockDb.clientes[idx] = {
          ...mockDb.clientes[idx]!,
          nombre: String(dto.nombre || mockDb.clientes[idx]!.nombre),
          cuit: (dto.cuit as string | null) ?? mockDb.clientes[idx]!.cuit,
          direccion: (dto.direccion as string | null) ?? mockDb.clientes[idx]!.direccion,
          telefono: (dto.telefono as string | null) ?? mockDb.clientes[idx]!.telefono,
          email: (dto.email as string | null) ?? mockDb.clientes[idx]!.email,
          condicionIva: (dto.condicionIva as string | null) ?? mockDb.clientes[idx]!.condicionIva,
          rowVersion: crypto.randomUUID(),
        }
        return mockDb.clientes[idx] as T
      }
      return mockDb.clientes[0] as T
    }
    case 'clientes_delete':
    case 'cliente_delete': {
      const id = String(args?.id ?? '')
      mockDb.clientes = mockDb.clientes.filter(c => c.id !== id)
      return null as T
    }
    case 'clientes_cuenta_corriente':
      return {
        clienteId: args?.id,
        saldo: '0.0000',
        movimientos: [],
        facturas: [],
      } as T

    // ==========================================
    // OBRAS
    // ==========================================
    case 'obras_list':
      return { items: mockDb.obras, totalCount: mockDb.obras.length, page: 1, size: 30 } as T
    case 'obras_lookup':
      return mockDb.obras.map(o => ({ id: o.id, label: `${o.numero}. ${o.nombre}` })) as T
    case 'obras_get':
    case 'obra_get': {
      const id = String(args?.id ?? '')
      const ob = mockDb.obras.find(o => o.id === id) || mockDb.obras[0]!
      return {
        ...ob,
        createdAt: new Date().toISOString(),
        updatedAt: null,
      } as T
    }
    case 'obras_create':
    case 'obra_create': {
      const dto = (args?.dto ?? {}) as Record<string, unknown>
      const cli = mockDb.clientes.find(c => c.id === dto.clienteId)
      const newOb: MockObra = {
        id: crypto.randomUUID(),
        numero: mockDb.obras.length + 1,
        nombre: String(dto.nombre || ''),
        direccion: (dto.direccion as string | null) ?? null,
        localidad: (dto.localidad as string | null) ?? null,
        clienteId: String(dto.clienteId || ''),
        clienteNombre: cli?.nombre ?? '',
        estado: 'Activa',
        trabajosCount: 0,
        rentabilidad: '0.0000',
        puedeEliminarse: true,
        rowVersion: crypto.randomUUID(),
      }
      mockDb.obras.unshift(newOb)
      return newOb as T
    }
    case 'obras_update':
    case 'obra_update': {
      const id = String(args?.id ?? '')
      const dto = (args?.dto ?? {}) as Record<string, unknown>
      const idx = mockDb.obras.findIndex(o => o.id === id)
      if (idx >= 0) {
        const cli = mockDb.clientes.find(c => c.id === dto.clienteId)
        mockDb.obras[idx] = {
          ...mockDb.obras[idx]!,
          nombre: String(dto.nombre || mockDb.obras[idx]!.nombre),
          direccion: (dto.direccion as string | null) ?? mockDb.obras[idx]!.direccion,
          localidad: (dto.localidad as string | null) ?? mockDb.obras[idx]!.localidad,
          clienteId: String(dto.clienteId || mockDb.obras[idx]!.clienteId),
          clienteNombre: cli?.nombre ?? mockDb.obras[idx]!.clienteNombre,
          rowVersion: crypto.randomUUID(),
        }
        return mockDb.obras[idx] as T
      }
      return mockDb.obras[0] as T
    }
    case 'obras_transition': {
      const id = String(args?.id ?? '')
      const nuevoEstado = String(args?.nuevoEstado ?? 'Activa')
      const idx = mockDb.obras.findIndex(o => o.id === id)
      if (idx >= 0) {
        mockDb.obras[idx]!.estado = nuevoEstado
        return mockDb.obras[idx] as T
      }
      return mockDb.obras[0] as T
    }
    case 'obras_delete':
    case 'obra_delete': {
      const id = String(args?.id ?? '')
      mockDb.obras = mockDb.obras.filter(o => o.id !== id)
      return null as T
    }
    case 'obras_next_numero':
      return (mockDb.obras.length + 1) as T

    // ==========================================
    // TRABAJOS
    // ==========================================
    case 'trabajos_list':
      return { items: mockDb.trabajos, totalCount: mockDb.trabajos.length, page: 1, size: 30 } as T
    case 'trabajos_lookup':
      return mockDb.trabajos.map(t => ({ id: t.id, label: t.descripcion })) as T
    case 'trabajos_get':
    case 'trabajo_get': {
      const id = String(args?.id ?? '')
      const trab = mockDb.trabajos.find(t => t.id === id) || mockDb.trabajos[0]!
      return {
        ...trab,
        createdAt: new Date().toISOString(),
        updatedAt: null,
      } as T
    }
    case 'trabajos_create':
    case 'trabajo_create': {
      const dto = (args?.dto ?? {}) as Record<string, unknown>
      const ob = mockDb.obras.find(o => o.id === dto.obraId)
      const newTrab: MockTrabajo = {
        id: crypto.randomUUID(),
        obraId: String(dto.obraId || ''),
        obraNumero: ob?.numero ?? 1,
        obraNombre: ob?.nombre ?? '',
        clienteId: ob?.clienteId ?? '',
        clienteNombre: ob?.clienteNombre ?? '',
        descripcion: String(dto.descripcion || ''),
        fechaInicio: String(dto.fechaInicio || new Date().toISOString().split('T')[0]),
        fechaFin: (dto.fechaFin as string | null) ?? null,
        presupuesto: String(dto.presupuesto || '0.0000'),
        estado: 'EnProceso',
        rowVersion: crypto.randomUUID(),
      }
      mockDb.trabajos.unshift(newTrab)
      return newTrab as T
    }
    case 'trabajos_update':
    case 'trabajo_update': {
      const id = String(args?.id ?? '')
      const dto = (args?.dto ?? {}) as Record<string, unknown>
      const idx = mockDb.trabajos.findIndex(t => t.id === id)
      if (idx >= 0) {
        mockDb.trabajos[idx] = {
          ...mockDb.trabajos[idx]!,
          descripcion: String(dto.descripcion || mockDb.trabajos[idx]!.descripcion),
          fechaInicio: String(dto.fechaInicio || mockDb.trabajos[idx]!.fechaInicio),
          fechaFin: (dto.fechaFin as string | null) ?? mockDb.trabajos[idx]!.fechaFin,
          presupuesto: String(dto.presupuesto || mockDb.trabajos[idx]!.presupuesto),
          rowVersion: crypto.randomUUID(),
        }
        return mockDb.trabajos[idx] as T
      }
      return mockDb.trabajos[0] as T
    }
    case 'trabajos_transition': {
      const id = String(args?.id ?? '')
      const nuevoEstado = String(args?.nuevoEstado ?? 'EnProceso')
      const idx = mockDb.trabajos.findIndex(t => t.id === id)
      if (idx >= 0) {
        mockDb.trabajos[idx]!.estado = nuevoEstado
        return mockDb.trabajos[idx] as T
      }
      return mockDb.trabajos[0] as T
    }
    case 'trabajos_delete':
    case 'trabajo_delete': {
      const id = String(args?.id ?? '')
      mockDb.trabajos = mockDb.trabajos.filter(t => t.id !== id)
      return null as T
    }

    // ==========================================
    // EMPLEADOS
    // ==========================================
    case 'empleados_list':
      return { items: mockDb.empleados, totalCount: mockDb.empleados.length, page: 1, size: 30 } as T
    case 'empleados_lookup':
      return mockDb.empleados.map(e => ({ id: e.id, label: `${e.nombre} (${e.cargo ?? ''})` })) as T
    case 'empleados_cargos':
      return Array.from(new Set(mockDb.empleados.map(e => e.cargo).filter(Boolean))) as T
    case 'empleados_get':
    case 'empleado_get': {
      const id = String(args?.id ?? '')
      const emp = mockDb.empleados.find(e => e.id === id) || mockDb.empleados[0]!
      return {
        ...emp,
        multiplicadorSabado: '1.5000',
        multiplicadorDomingo: '2.0000',
        multiplicadorFeriado: '2.0000',
        createdAt: new Date().toISOString(),
        updatedAt: null,
      } as T
    }
    case 'empleados_create':
    case 'empleado_create': {
      const dto = (args?.dto ?? {}) as Record<string, unknown>
      const newEmp: MockEmpleado = {
        id: crypto.randomUUID(),
        nombre: String(dto.nombre || ''),
        dni: (dto.dni as string | null) ?? null,
        cargo: (dto.cargo as string | null) ?? null,
        tarifaDiaria: String(dto.tarifaDiaria || '0.0000'),
        sueldoBase: String(dto.sueldoBase || '0.0000'),
        pagoFrecuencia: String(dto.pagoFrecuencia || 'Quincenal'),
        email: (dto.email as string | null) ?? null,
        telefono: (dto.telefono as string | null) ?? null,
        fechaIngreso: String(dto.fechaIngreso || new Date().toISOString().split('T')[0]),
        fechaEgreso: (dto.fechaEgreso as string | null) ?? null,
        activo: Boolean(dto.activo ?? true),
        rowVersion: crypto.randomUUID(),
      }
      mockDb.empleados.unshift(newEmp)
      return newEmp as T
    }
    case 'empleados_update':
    case 'empleado_update': {
      const id = String(args?.id ?? '')
      const dto = (args?.dto ?? {}) as Record<string, unknown>
      const idx = mockDb.empleados.findIndex(e => e.id === id)
      if (idx >= 0) {
        mockDb.empleados[idx] = {
          ...mockDb.empleados[idx]!,
          nombre: String(dto.nombre || mockDb.empleados[idx]!.nombre),
          dni: (dto.dni as string | null) ?? mockDb.empleados[idx]!.dni,
          cargo: (dto.cargo as string | null) ?? mockDb.empleados[idx]!.cargo,
          tarifaDiaria: String(dto.tarifaDiaria || mockDb.empleados[idx]!.tarifaDiaria),
          sueldoBase: String(dto.sueldoBase || mockDb.empleados[idx]!.sueldoBase),
          pagoFrecuencia: String(dto.pagoFrecuencia || mockDb.empleados[idx]!.pagoFrecuencia),
          email: (dto.email as string | null) ?? mockDb.empleados[idx]!.email,
          telefono: (dto.telefono as string | null) ?? mockDb.empleados[idx]!.telefono,
          fechaIngreso: String(dto.fechaIngreso || mockDb.empleados[idx]!.fechaIngreso),
          fechaEgreso: (dto.fechaEgreso as string | null) ?? mockDb.empleados[idx]!.fechaEgreso,
          activo: Boolean(dto.activo ?? mockDb.empleados[idx]!.activo),
          rowVersion: crypto.randomUUID(),
        }
        return mockDb.empleados[idx] as T
      }
      return mockDb.empleados[0] as T
    }
    case 'empleados_delete':
    case 'empleado_delete': {
      const id = String(args?.id ?? '')
      mockDb.empleados = mockDb.empleados.filter(e => e.id !== id)
      return null as T
    }
    case 'asistencias_mes':
      return { asistencias: [], totalDiasTrabajados: '11.0000' } as T
    case 'asistencias_upsert':
    case 'asistencias_bulk':
      return { creadas: 10 } as T

    // ==========================================
    // FACTURAS
    // ==========================================
    case 'facturas_list':
      return { items: mockDb.facturas, totalCount: mockDb.facturas.length, page: 1, size: 30 } as T
    case 'facturas_get':
    case 'factura_get': {
      const id = String(args?.id ?? '')
      const fact = mockDb.facturas.find(f => f.id === id) || mockDb.facturas[0]!
      return {
        ...fact,
        observaciones: 'Facturación de obra',
        items: [],
        pagos: [],
        createdAt: new Date().toISOString(),
        updatedAt: null,
      } as T
    }
    case 'facturas_create':
    case 'factura_create': {
      const dto = (args?.dto ?? {}) as Record<string, unknown>
      const cli = mockDb.clientes.find(c => c.id === dto.clienteId)
      const subNum = parseFloat(String(dto.subtotal || '0'))
      const ivaNum = parseFloat(String(dto.iva || '0'))
      const totNum = subNum + ivaNum
      const newFact: MockFactura = {
        id: crypto.randomUUID(),
        numero: String(dto.numero || `0001-0000010${mockDb.facturas.length + 1}`),
        fecha: String(dto.fecha || new Date().toISOString().split('T')[0]),
        fechaVencimiento: (dto.fechaVencimiento as string | null) ?? null,
        clienteId: String(dto.clienteId || ''),
        clienteNombre: cli?.nombre ?? '',
        estado: 'Borrador',
        subtotal: subNum.toFixed(4),
        iva: ivaNum.toFixed(4),
        total: totNum.toFixed(4),
        saldoPendiente: totNum.toFixed(4),
        rowVersion: crypto.randomUUID(),
      }
      mockDb.facturas.unshift(newFact)
      return newFact as T
    }
    case 'facturas_update':
    case 'factura_update': {
      const id = String(args?.id ?? '')
      const dto = (args?.dto ?? {}) as Record<string, unknown>
      const idx = mockDb.facturas.findIndex(f => f.id === id)
      if (idx >= 0) {
        const subNum = parseFloat(String(dto.subtotal || mockDb.facturas[idx]!.subtotal))
        const ivaNum = parseFloat(String(dto.iva || mockDb.facturas[idx]!.iva))
        const totNum = subNum + ivaNum
        mockDb.facturas[idx] = {
          ...mockDb.facturas[idx]!,
          numero: String(dto.numero || mockDb.facturas[idx]!.numero),
          fecha: String(dto.fecha || mockDb.facturas[idx]!.fecha),
          fechaVencimiento: (dto.fechaVencimiento as string | null) ?? mockDb.facturas[idx]!.fechaVencimiento,
          subtotal: subNum.toFixed(4),
          iva: ivaNum.toFixed(4),
          total: totNum.toFixed(4),
          rowVersion: crypto.randomUUID(),
        }
        return mockDb.facturas[idx] as T
      }
      return mockDb.facturas[0] as T
    }
    case 'facturas_transition': {
      const id = String(args?.id ?? '')
      const nuevoEstado = String(args?.nuevoEstado ?? 'Emitida')
      const idx = mockDb.facturas.findIndex(f => f.id === id)
      if (idx >= 0) {
        mockDb.facturas[idx]!.estado = nuevoEstado
        return mockDb.facturas[idx] as T
      }
      return mockDb.facturas[0] as T
    }
    case 'facturas_delete':
    case 'factura_delete': {
      const id = String(args?.id ?? '')
      mockDb.facturas = mockDb.facturas.filter(f => f.id !== id)
      return null as T
    }
    case 'pagos_factura_registrar': {
      const id = String(args?.facturaId ?? '')
      const idx = mockDb.facturas.findIndex(f => f.id === id)
      if (idx >= 0) {
        mockDb.facturas[idx]!.estado = 'Pagada'
        mockDb.facturas[idx]!.saldoPendiente = '0.0000'
      }
      return { id: crypto.randomUUID(), facturaId: id, monto: '1000.0000', fecha: new Date().toISOString().split('T')[0], medioPago: 'Transferencia', rowVersion: 'v1' } as T
    }
    case 'pagos_factura_eliminar':
      return null as T

    // ==========================================
    // CERTIFICADOS
    // ==========================================
    case 'certificados_list':
      return { items: mockDb.certificados, totalCount: mockDb.certificados.length, page: 1, size: 30 } as T
    case 'certificados_get':
    case 'certificado_get': {
      const id = String(args?.id ?? '')
      const cert = mockDb.certificados.find(c => c.id === id) || mockDb.certificados[0]!
      return {
        ...cert,
        ajusteUocra: '148000.0000',
        otrosDescuentos: '0.0000',
        observaciones: 'Certificado aprobado',
        items: [],
        createdAt: new Date().toISOString(),
        updatedAt: null,
      } as T
    }
    case 'certificados_borrador':
      return {
        ordenTrabajoId: mockDb.ordenes[0]?.id ?? '',
        ordenTitulo: mockDb.ordenes[0]?.titulo ?? '',
        numeroSugerido: 1,
        trabajoDescripcion: mockDb.trabajos[0]?.descripcion ?? '',
        obraNombre: mockDb.obras[0]?.nombre ?? '',
        clienteNombre: mockDb.clientes[2]?.nombre ?? '',
        ajusteUocraPorcentaje: '8.0000',
        otrosDescuentos: '0.0000',
        items: [],
      } as T
    case 'certificados_emitir': {
      const newCert: MockCertificado = {
        id: crypto.randomUUID(),
        ordenTrabajoId: mockDb.ordenes[0]?.id ?? '',
        ordenTitulo: mockDb.ordenes[0]?.titulo ?? '',
        numero: mockDb.certificados.length + 1,
        fecha: new Date().toISOString().split('T')[0] ?? '',
        totalCertificado: '1500000.0000',
        totalNeto: '1620000.0000',
        rowVersion: crypto.randomUUID(),
      }
      mockDb.certificados.unshift(newCert)
      return newCert as T
    }
    case 'certificados_anular': {
      const id = String(args?.id ?? '')
      mockDb.certificados = mockDb.certificados.filter(c => c.id !== id)
      return null as T
    }

    // ==========================================
    // LIQUIDACIONES
    // ==========================================
    case 'liquidaciones_list':
      return { items: mockDb.liquidaciones, totalCount: mockDb.liquidaciones.length, page: 1, size: 30 } as T
    case 'liquidaciones_get':
    case 'liquidacion_get': {
      const id = String(args?.id ?? '')
      const liq = mockDb.liquidaciones.find(l => l.id === id) || mockDb.liquidaciones[0]!
      return {
        ...liq,
        incluirSabados: true,
        incluirDomingos: false,
        incluirFeriados: false,
        multiplicadorSabado: '1.5000',
        multiplicadorDomingo: '2.0000',
        multiplicadorFeriado: '2.0000',
        observaciones: 'Liquidación quincenal',
        desglose: {
          jornadasCompletas: '11.0000',
          jornadasMedias: '0.0000',
          faltas: 0,
          faltasJustificadas: 0,
          diasSabado: '1.0000',
          diasDomingo: '0.0000',
          diasFeriado: '0.0000',
          multiplicadorSabado: '1.5000',
          multiplicadorDomingo: '2.0000',
          multiplicadorFeriado: '2.0000',
          recargos: '22500.0000',
        },
        adelantos: [],
        createdAt: new Date().toISOString(),
        updatedAt: null,
      } as T
    }
    case 'liquidaciones_sugerir':
      return [
        {
          empleadoId: mockDb.empleados[0]?.id ?? '',
          empleadoNombre: mockDb.empleados[0]?.nombre ?? '',
          cargo: mockDb.empleados[0]?.cargo ?? '',
          tarifaDiaria: mockDb.empleados[0]?.tarifaDiaria ?? '45000.0000',
          diasSugeridos: '11.0000',
          origen: 'Asistencia',
          desglose: {
            jornadasCompletas: '11.0000',
            jornadasMedias: '0.0000',
            faltas: 0,
            faltasJustificadas: 0,
            diasSabado: '1.0000',
            diasDomingo: '0.0000',
            diasFeriado: '0.0000',
            multiplicadorSabado: '1.5000',
            multiplicadorDomingo: '2.0000',
            multiplicadorFeriado: '2.0000',
            recargos: '22500.0000',
          },
          adelantos: [],
          totalBruto: '495000.0000',
          totalAdelantos: '0.0000',
          totalNeto: '495000.0000',
        },
      ] as T
    case 'liquidaciones_emitir': {
      const newLiq: MockLiquidacion = {
        id: crypto.randomUUID(),
        empleadoId: mockDb.empleados[0]?.id ?? '',
        empleadoNombre: mockDb.empleados[0]?.nombre ?? '',
        empleadoCargo: mockDb.empleados[0]?.cargo ?? '',
        fechaInicio: new Date().toISOString().split('T')[0] ?? '',
        fechaFin: new Date().toISOString().split('T')[0] ?? '',
        diasTrabajados: '11.0000',
        tarifaAplicada: '45000.0000',
        totalBruto: '495000.0000',
        totalAdelantos: '0.0000',
        totalNeto: '495000.0000',
        tienePdf: false,
        rowVersion: crypto.randomUUID(),
      }
      mockDb.liquidaciones.unshift(newLiq)
      return [newLiq] as T
    }
    case 'liquidaciones_delete':
    case 'liquidacion_delete': {
      const id = String(args?.id ?? '')
      mockDb.liquidaciones = mockDb.liquidaciones.filter(l => l.id !== id)
      return null as T
    }

    // ==========================================
    // CATEGORIAS & TIPOS
    // ==========================================
    case 'categorias_list':
      return { items: mockDb.categorias, totalCount: mockDb.categorias.length, page: 1, size: 30 } as T
    case 'categorias_lookup':
      return mockDb.categorias.map(c => ({ id: c.id, label: c.nombre })) as T
    case 'categorias_get': {
      const id = String(args?.id ?? '')
      const cat = mockDb.categorias.find(c => c.id === id) || mockDb.categorias[0]!
      return { ...cat, createdAt: new Date().toISOString(), updatedAt: null } as T
    }
    case 'categorias_create': {
      const dto = (args?.dto ?? {}) as Record<string, unknown>
      const newCat: MockCategory = {
        id: crypto.randomUUID(),
        nombre: String(dto.nombre || ''),
        descripcion: (dto.descripcion as string | null) ?? null,
        colorHex: (dto.colorHex as string | null) ?? '#3B82F6',
        icono: (dto.icono as string | null) ?? 'package',
        categoriaPadreId: (dto.categoriaPadreId as string | null) ?? null,
        categoriaPadreNombre: null,
        nivel: 0,
        movimientosCount: 0,
        subcategoriasCount: 0,
        puedeEliminarse: true,
        rowVersion: crypto.randomUUID(),
      }
      mockDb.categorias.unshift(newCat)
      return newCat as T
    }
    case 'categorias_update': {
      const id = String(args?.id ?? '')
      const dto = (args?.dto ?? {}) as Record<string, unknown>
      const idx = mockDb.categorias.findIndex(c => c.id === id)
      if (idx >= 0) {
        mockDb.categorias[idx] = {
          ...mockDb.categorias[idx]!,
          nombre: String(dto.nombre || mockDb.categorias[idx]!.nombre),
          descripcion: (dto.descripcion as string | null) ?? mockDb.categorias[idx]!.descripcion,
          colorHex: (dto.colorHex as string | null) ?? mockDb.categorias[idx]!.colorHex,
          icono: (dto.icono as string | null) ?? mockDb.categorias[idx]!.icono,
          categoriaPadreId: (dto.categoriaPadreId as string | null) ?? mockDb.categorias[idx]!.categoriaPadreId,
          rowVersion: crypto.randomUUID(),
        }
        return mockDb.categorias[idx] as T
      }
      return mockDb.categorias[0] as T
    }
    case 'categorias_delete': {
      const id = String(args?.id ?? '')
      mockDb.categorias = mockDb.categorias.filter(c => c.id !== id)
      return null as T
    }

    case 'tipos_movimiento_list':
      return { items: mockDb.tiposMovimiento, totalCount: mockDb.tiposMovimiento.length, page: 1, size: 30 } as T
    case 'tipos_movimiento_lookup':
      return mockDb.tiposMovimiento.map(t => ({ id: t.id, label: t.nombre })) as T
    case 'tipos_movimiento_get': {
      const id = String(args?.id ?? '')
      const tipo = mockDb.tiposMovimiento.find(t => t.id === id) || mockDb.tiposMovimiento[0]!
      return { ...tipo, createdAt: new Date().toISOString(), updatedAt: null } as T
    }
    case 'tipos_movimiento_create': {
      const dto = (args?.dto ?? {}) as Record<string, unknown>
      const newTipo: MockTipoMovimiento = {
        id: crypto.randomUUID(),
        nombre: String(dto.nombre || ''),
        descripcion: (dto.descripcion as string | null) ?? null,
        esIngreso: Boolean(dto.esIngreso ?? true),
        esSistema: false,
        movimientosCount: 0,
        puedeEliminarse: true,
        rowVersion: crypto.randomUUID(),
      }
      mockDb.tiposMovimiento.unshift(newTipo)
      return newTipo as T
    }
    case 'tipos_movimiento_update': {
      const id = String(args?.id ?? '')
      const dto = (args?.dto ?? {}) as Record<string, unknown>
      const idx = mockDb.tiposMovimiento.findIndex(t => t.id === id)
      if (idx >= 0) {
        mockDb.tiposMovimiento[idx] = {
          ...mockDb.tiposMovimiento[idx]!,
          nombre: String(dto.nombre || mockDb.tiposMovimiento[idx]!.nombre),
          descripcion: (dto.descripcion as string | null) ?? mockDb.tiposMovimiento[idx]!.descripcion,
          rowVersion: crypto.randomUUID(),
        }
        return mockDb.tiposMovimiento[idx] as T
      }
      return mockDb.tiposMovimiento[0] as T
    }
    case 'tipos_movimiento_delete': {
      const id = String(args?.id ?? '')
      mockDb.tiposMovimiento = mockDb.tiposMovimiento.filter(t => t.id !== id)
      return null as T
    }

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
    console.error(`[ipc] ${command} failed`, {
      code: apiError.code,
      traceId: apiError.traceId,
    })
    throw apiError
  }
}
