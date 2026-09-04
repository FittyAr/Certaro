import type { AppConfig } from '../types'
import type {
  MockCategory,
  MockCliente,
  MockDb,
  MockEmpleado,
  MockFactura,
  MockLiquidacion,
  MockMovimiento,
  MockOrden,
  MockProyecto,
  MockTipoMovimiento,
  MockTrabajo,
} from './types'

export const DEFAULT_CONFIG: AppConfig = {
  application: {
    name: 'Certaro',
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
    nombreComercial: 'Certaro',
    lema: 'Instalaciones Eléctricas',
    contratista: 'Pablo Báez',
    cuit: '20-12345678-9',
    direccion: 'Av. Principal 123',
    telefono: '+54 9 11 1234-5678',
    email: 'contacto@certaro.com',
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
    asistenciaMaxRangoDias: 92,
  },
  dashboard: {
    lastPeriod: 'mensual',
    privacyMode: false,
    casasDolar: ['blue', 'oficial'],
    cotizacionPorDefecto: 'blue',
    topClientesCantidad: 5,
    topCategoriasCantidad: 5,
    ultimosMovimientosCantidad: 10,
    proyectosRankingCantidad: 5,
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
    pieDePagina: 'Certaro - Gestión de Proyectos',
  },
}

const CONFIG_STORAGE_KEY = 'certaro_mock_config_v2'
const DB_STORAGE_KEY = 'certaro_mock_db_v2'

export function loadMockConfig(): AppConfig {
  if (typeof window !== 'undefined' && typeof localStorage !== 'undefined') {
    try {
      const stored = localStorage.getItem(CONFIG_STORAGE_KEY)
      if (stored) return JSON.parse(stored)
    } catch {
      // Storage unavailable or parsing error; fallback to default config.
    }
  }
  return structuredClone(DEFAULT_CONFIG)
}

export function saveMockConfig(cfg: AppConfig): void {
  if (typeof window !== 'undefined' && typeof localStorage !== 'undefined') {
    try {
      localStorage.setItem(CONFIG_STORAGE_KEY, JSON.stringify(cfg))
    } catch {
      // Storage unavailable or quota exceeded; ignore.
    }
  }
}

export let mockConfig = loadMockConfig()

export function setMockConfig(cfg: AppConfig): void {
  mockConfig = cfg
  saveMockConfig(cfg)
}

export function generateUuid(): string {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID()
  }
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
    const r = (Math.random() * 16) | 0
    const v = c === 'x' ? r : (r & 0x3) | 0x8
    return v.toString(16)
  })
}

export function createSeedMockDb(): MockDb {
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

  const cli1 = { id: '30000000-0000-0000-0000-000000000001', nombre: 'Constructora del Plata S.A.', cuit: '30-71234567-9', direccion: 'Av. del Libertador 1234, CABA', telefono: '011-4567-8900', email: 'info@constructoradelplata.com', condicionIva: 'Responsable Inscripto', proyectosCount: 1, facturasCount: 1, deuda: '0.0000', puedeEliminarse: false, rowVersion: 'v1' }
  const cli2 = { id: '30000000-0000-0000-0000-000000000002', nombre: 'Desarrollos Urbanos SRL', cuit: '30-79876543-1', direccion: 'San Martín 567, Rosario', telefono: '0341-423-4567', email: 'admin@desarrollosurbanos.com', condicionIva: 'Responsable Inscripto', proyectosCount: 1, facturasCount: 1, deuda: '7502.0000', puedeEliminarse: false, rowVersion: 'v1' }
  const cli3 = { id: '30000000-0000-0000-0000-000000000003', nombre: 'Consorcio Torre Alvear', cuit: '30-65432109-8', direccion: 'Av. Alvear 1890, CABA', telefono: '011-4812-3456', email: 'consorcio@torrealvear.com', condicionIva: 'Consumidor Final', proyectosCount: 1, facturasCount: 1, deuda: '0.0000', puedeEliminarse: false, rowVersion: 'v1' }
  const cli4 = { id: '30000000-0000-0000-0000-000000000004', nombre: 'Juan Carlos Pérez', cuit: '20-28123456-3', direccion: 'Belgrano 432, San Isidro', telefono: '011-15-5432-1098', email: 'jcperez@gmail.com', condicionIva: 'Consumidor Final', proyectosCount: 1, facturasCount: 0, deuda: '0.0000', puedeEliminarse: false, rowVersion: 'v1' }
  const clientes: MockCliente[] = [cli1, cli2, cli3, cli4]

  const obra1 = { id: '40000000-0000-0000-0000-000000000001', numero: 1, nombre: 'Instalación Eléctrica Integral Torre Alvear', direccion: 'Av. Alvear 1890', localidad: 'CABA', clienteId: cli3.id, clienteNombre: cli3.nombre, estado: 'Activa', trabajosCount: 2, rentabilidad: '11120.0000', puedeEliminarse: false, rowVersion: 'v1' }
  const obra2 = { id: '40000000-0000-0000-0000-000000000002', numero: 2, nombre: 'Iluminación y Fuerza Motriz Planta del Plata', direccion: 'Parque Industrial Norte', localidad: 'Tigre', clienteId: cli1.id, clienteNombre: cli1.nombre, estado: 'Activa', trabajosCount: 1, rentabilidad: '4550.0000', puedeEliminarse: false, rowVersion: 'v1' }
  const obra3 = { id: '40000000-0000-0000-0000-000000000003', numero: 3, nombre: 'Cableado Estructurado Oficinas Centro', direccion: 'San Martín 567', localidad: 'Rosario', clienteId: cli2.id, clienteNombre: cli2.nombre, estado: 'Finalizada', trabajosCount: 1, rentabilidad: '14000.0000', puedeEliminarse: false, rowVersion: 'v1' }
  const obra4 = { id: '40000000-0000-0000-0000-000000000004', numero: 4, nombre: 'Refacción y Tablero Eléctrico Domiciliario', direccion: 'Belgrano 432', localidad: 'San Isidro', clienteId: cli4.id, clienteNombre: cli4.nombre, estado: 'Activa', trabajosCount: 1, rentabilidad: '6500.0000', puedeEliminarse: false, rowVersion: 'v1' }
  const proyectos: MockProyecto[] = [obra1, obra2, obra3, obra4]

  const trab1 = { id: '50000000-0000-0000-0000-000000000001', proyectoId: obra1.id, proyectoNumero: 1, proyectoNombre: obra1.nombre, clienteId: cli3.id, clienteNombre: cli3.nombre, descripcion: 'Tendido de bandejas portacables en subsuelos', fechaInicio: '2025-02-01', fechaFin: null, presupuesto: '1850000.0000', estado: 'EnProceso', rowVersion: 'v1' }
  const trab2 = { id: '50000000-0000-0000-0000-000000000002', proyectoId: obra1.id, proyectoNumero: 1, proyectoNombre: obra1.nombre, clienteId: cli3.id, clienteNombre: cli3.nombre, descripcion: 'Montaje de tableros seccionales por piso', fechaInicio: '2025-02-10', fechaFin: null, presupuesto: '3200000.0000', estado: 'EnProceso', rowVersion: 'v1' }
  const trab3 = { id: '50000000-0000-0000-0000-000000000003', proyectoId: obra2.id, proyectoNumero: 2, proyectoNombre: obra2.nombre, clienteId: cli1.id, clienteNombre: cli1.nombre, descripcion: 'Iluminación perimetral LED alta potencia', fechaInicio: '2025-01-20', fechaFin: '2025-02-25', presupuesto: '950000.0000', estado: 'Finalizado', rowVersion: 'v1' }
  const trab4 = { id: '50000000-0000-0000-0000-000000000004', proyectoId: obra3.id, proyectoNumero: 3, proyectoNombre: obra3.nombre, clienteId: cli2.id, clienteNombre: cli2.nombre, descripcion: 'Puestos de red Cat6 y rack central', fechaInicio: '2025-01-10', fechaFin: '2025-02-20', presupuesto: '1400000.0000', estado: 'Finalizado', rowVersion: 'v1' }
  const trab5 = { id: '50000000-0000-0000-0000-000000000005', proyectoId: obra4.id, proyectoNumero: 4, proyectoNombre: obra4.nombre, clienteId: cli4.id, clienteNombre: cli4.nombre, descripcion: 'Recableado completo y disyuntor diferencial', fechaInicio: '2025-02-15', fechaFin: null, presupuesto: '650000.0000', estado: 'EnProceso', rowVersion: 'v1' }
  const trabajos: MockTrabajo[] = [trab1, trab2, trab3, trab4, trab5]

  const ord1 = { id: '60000000-0000-0000-0000-000000000001', trabajoId: trab1.id, titulo: 'Certificación de Avance Etapa 1', numeroCertificado: 'CERT-001', fecha: '2025-02-20', totalCertificados: 1, totalNeto: '1998000.0000', rowVersion: 'v1' }
  const ord2 = { id: '60000000-0000-0000-0000-000000000002', trabajoId: trab2.id, titulo: 'Certificación de Avance Etapa 2', numeroCertificado: 'CERT-002', fecha: '2025-02-22', totalCertificados: 1, totalNeto: '3200000.0000', rowVersion: 'v1' }
  const ordenes: MockOrden[] = [ord1, ord2]

  const cert1 = { id: '70000000-0000-0000-0000-000000000001', ordenTrabajoId: ord1.id, ordenTitulo: ord1.titulo, numero: 1, fecha: '2025-02-22', totalCertificado: '1850000.0000', totalNeto: '1998000.0000', rowVersion: 'v1' }
  const certificados = [cert1]

  const emp1 = { id: '80000000-0000-0000-0000-000000000001', nombre: 'Ricardo Darín', dni: '20.123.456', cargo: 'Operario Electricista', tarifaDiaria: '45000.0000', sueldoBase: '450000.0000', pagoFrecuencia: 'Quincenal', email: 'ricardo.darin@obra.com', telefono: '1145678901', fechaIngreso: '2025-01-15', fechaEgreso: null, activo: true, rowVersion: 'v1' }
  const emp2 = { id: '80000000-0000-0000-0000-000000000002', nombre: 'Guillermo Francella', dni: '22.345.678', cargo: 'Capataz de Proyecto', tarifaDiaria: '55000.0000', sueldoBase: '550000.0000', pagoFrecuencia: 'Quincenal', email: 'guillermo.francella@obra.com', telefono: '1145678902', fechaIngreso: '2025-01-15', fechaEgreso: null, activo: true, rowVersion: 'v1' }
  const emp3 = { id: '80000000-0000-0000-0000-000000000003', nombre: 'Natalia Oreiro', dni: '25.678.901', cargo: 'Técnica Instaladora', tarifaDiaria: '48000.0000', sueldoBase: '480000.0000', pagoFrecuencia: 'Quincenal', email: 'natalia.oreiro@obra.com', telefono: '1145678903', fechaIngreso: '2025-01-15', fechaEgreso: null, activo: true, rowVersion: 'v1' }
  const emp4 = { id: '80000000-0000-0000-0000-000000000004', nombre: 'Diego Peretti', dni: '18.901.234', cargo: 'Ayudante Práctico', tarifaDiaria: '38000.0000', sueldoBase: '380000.0000', pagoFrecuencia: 'Quincenal', email: 'diego.peretti@obra.com', telefono: '1145678904', fechaIngreso: '2025-01-15', fechaEgreso: null, activo: true, rowVersion: 'v1' }
  const empleados: MockEmpleado[] = [emp1, emp2, emp3, emp4]

  const fact1 = { id: '90000000-0000-0000-0000-000000000001', numero: '0001-00000101', fecha: '2025-02-15', fechaVencimiento: '2025-03-15', clienteId: cli1.id, clienteNombre: cli1.nombre, estado: 'Cobrada', subtotal: '1500000.0000', iva: '315000.0000', total: '1815000.0000', saldoPendiente: '0.0000', rowVersion: 'v1' }
  const fact2 = { id: '90000000-0000-0000-0000-000000000002', numero: '0001-00000102', fecha: '2025-02-20', fechaVencimiento: '2025-03-20', clienteId: cli2.id, clienteNombre: cli2.nombre, estado: 'Emitida', subtotal: '620000.0000', iva: '130200.0000', total: '750200.0000', saldoPendiente: '750200.0000', rowVersion: 'v1' }
  const facturas: MockFactura[] = [fact1, fact2]

  const mov1 = { id: 'a0000000-0000-0000-0000-000000000001', fecha: '2025-02-05', concepto: 'Cobro de anticipo Obra Torre Alvear', monto: '12000.0000', cantidad: '1.0000', total: '12000.0000', moneda: 'Usd', cotizacionAplicada: '1220.0000', tipoMovimientoId: tipoIngreso.id, tipoMovimientoNombre: tipoIngreso.nombre, esIngreso: true, categoriaId: null, categoriaNombre: null, categoriaColor: null, clienteId: cli3.id, trabajoId: trab1.id, empleadoId: null, facturaId: null, tipoConceptoPagoId: null, bloqueadoPorLiquidacion: false, rowVersion: 'v1' }
  const mov2 = { id: 'a0000000-0000-0000-0000-000000000002', fecha: '2025-02-06', concepto: 'Compra cable sintenax 4x16mm x 100m', monto: '620.0000', cantidad: '1.0000', total: '620.0000', moneda: 'Usd', cotizacionAplicada: '1225.0000', tipoMovimientoId: tipoGasto.id, tipoMovimientoNombre: tipoGasto.nombre, esIngreso: false, categoriaId: cat2.id, categoriaNombre: cat2.nombre, categoriaColor: cat2.colorHex, clienteId: null, trabajoId: trab1.id, empleadoId: null, facturaId: null, tipoConceptoPagoId: null, bloqueadoPorLiquidacion: false, rowVersion: 'v1' }
  const mov3 = { id: 'a0000000-0000-0000-0000-000000000003', fecha: '2025-02-08', concepto: 'Tableros seccionales chapa 60x80 con disyuntores', monto: '1850000.0000', cantidad: '1.0000', total: '1850000.0000', moneda: 'Ars', cotizacionAplicada: null, tipoMovimientoId: tipoGasto.id, tipoMovimientoNombre: tipoGasto.nombre, esIngreso: false, categoriaId: cat1.id, categoriaNombre: cat1.nombre, categoriaColor: cat1.colorHex, clienteId: null, trabajoId: trab2.id, empleadoId: null, facturaId: null, tipoConceptoPagoId: null, bloqueadoPorLiquidacion: false, rowVersion: 'v1' }
  const mov4 = { id: 'a0000000-0000-0000-0000-000000000004', fecha: '2025-02-10', concepto: 'Adelanto quincena Ricardo Darín', monto: '100000.0000', cantidad: '1.0000', total: '100000.0000', moneda: 'Ars', cotizacionAplicada: null, tipoMovimientoId: tipoAdelanto.id, tipoMovimientoNombre: tipoAdelanto.nombre, esIngreso: false, categoriaId: null, categoriaNombre: null, categoriaColor: null, clienteId: null, trabajoId: null, empleadoId: emp1.id, facturaId: null, tipoConceptoPagoId: null, bloqueadoPorLiquidacion: false, rowVersion: 'v1' }
  const mov5 = { id: 'a0000000-0000-0000-0000-000000000005', fecha: '2025-02-10', concepto: 'Adelanto quincena Guillermo Francella', monto: '150000.0000', cantidad: '1.0000', total: '150000.0000', moneda: 'Ars', cotizacionAplicada: null, tipoMovimientoId: tipoAdelanto.id, tipoMovimientoNombre: tipoAdelanto.nombre, esIngreso: false, categoriaId: null, categoriaNombre: null, categoriaColor: null, clienteId: null, trabajoId: null, empleadoId: emp2.id, facturaId: null, tipoConceptoPagoId: null, bloqueadoPorLiquidacion: false, rowVersion: 'v1' }
  const mov6 = { id: 'a0000000-0000-0000-0000-000000000006', fecha: '2025-02-12', concepto: 'Alquiler andamios tubulares y tablones', monto: '260.0000', cantidad: '1.0000', total: '260.0000', moneda: 'Usd', cotizacionAplicada: '1230.0000', tipoMovimientoId: tipoGasto.id, tipoMovimientoNombre: tipoGasto.nombre, esIngreso: false, categoriaId: cat3.id, categoriaNombre: cat3.nombre, categoriaColor: cat3.colorHex, clienteId: null, trabajoId: trab1.id, empleadoId: null, facturaId: null, tipoConceptoPagoId: null, bloqueadoPorLiquidacion: false, rowVersion: 'v1' }
  const mov7 = { id: 'a0000000-0000-0000-0000-000000000007', fecha: '2025-02-15', concepto: 'Cobro Certificación N°1 Factura 0001-00000101', monto: '1815000.0000', cantidad: '1.0000', total: '1815000.0000', moneda: 'Ars', cotizacionAplicada: null, tipoMovimientoId: tipoIngreso.id, tipoMovimientoNombre: tipoIngreso.nombre, esIngreso: true, categoriaId: null, categoriaNombre: null, categoriaColor: null, clienteId: cli1.id, trabajoId: trab3.id, empleadoId: null, facturaId: fact1.id, tipoConceptoPagoId: null, bloqueadoPorLiquidacion: false, rowVersion: 'v1' }
  const mov8 = { id: 'a0000000-0000-0000-0000-000000000008', fecha: '2025-02-18', concepto: 'Flete y traslado de luminarias a planta', monto: '85000.0000', cantidad: '1.0000', total: '85000.0000', moneda: 'Ars', cotizacionAplicada: null, tipoMovimientoId: tipoGasto.id, tipoMovimientoNombre: tipoGasto.nombre, esIngreso: false, categoriaId: cat4.id, categoriaNombre: cat4.nombre, categoriaColor: cat4.colorHex, clienteId: null, trabajoId: trab3.id, empleadoId: null, facturaId: null, tipoConceptoPagoId: null, bloqueadoPorLiquidacion: false, rowVersion: 'v1' }
  const mov9 = { id: 'a0000000-0000-0000-0000-000000000009', fecha: '2025-02-20', concepto: 'Venta bobinas vacías y recortes de cobre', monto: '180000.0000', cantidad: '1.0000', total: '180000.0000', moneda: 'Ars', cotizacionAplicada: null, tipoMovimientoId: tipoChatarra.id, tipoMovimientoNombre: tipoChatarra.nombre, esIngreso: true, categoriaId: null, categoriaNombre: null, categoriaColor: null, clienteId: null, trabajoId: null, empleadoId: null, facturaId: null, tipoConceptoPagoId: null, bloqueadoPorLiquidacion: false, rowVersion: 'v1' }
  const mov10 = { id: 'a0000000-0000-0000-0000-000000000010', fecha: '2025-02-22', concepto: 'Pago IIBB mensual provincia Bs As', monto: '94000.0000', cantidad: '1.0000', total: '94000.0000', moneda: 'Ars', cotizacionAplicada: null, tipoMovimientoId: tipoGasto.id, tipoMovimientoNombre: tipoGasto.nombre, esIngreso: false, categoriaId: cat5.id, categoriaNombre: cat5.nombre, categoriaColor: cat5.colorHex, clienteId: null, trabajoId: null, empleadoId: null, facturaId: null, tipoConceptoPagoId: null, bloqueadoPorLiquidacion: false, rowVersion: 'v1' }
  const mov11 = { id: 'a0000000-0000-0000-0000-000000000011', fecha: '2025-02-24', concepto: 'Combustible camioneta Hilux traslados Rosario', monto: '48000.0000', cantidad: '1.0000', total: '48000.0000', moneda: 'Ars', cotizacionAplicada: null, tipoMovimientoId: tipoGasto.id, tipoMovimientoNombre: tipoGasto.nombre, esIngreso: false, categoriaId: cat6.id, categoriaNombre: cat6.nombre, categoriaColor: cat6.colorHex, clienteId: null, trabajoId: trab4.id, empleadoId: null, facturaId: null, tipoConceptoPagoId: null, bloqueadoPorLiquidacion: false, rowVersion: 'v1' }
  const movimientos: MockMovimiento[] = [mov1, mov2, mov3, mov4, mov5, mov6, mov7, mov8, mov9, mov10, mov11]

  const liq1 = { id: 'b0000000-0000-0000-0000-000000000001', empleadoId: emp1.id, empleadoNombre: emp1.nombre, empleadoCargo: emp1.cargo, fechaInicio: '2025-02-01', fechaFin: '2025-02-15', diasTrabajados: '11.0000', tarifaAplicada: '45000.0000', totalBruto: '495000.0000', totalAdelantos: '100000.0000', totalNeto: '395000.0000', tienePdf: true, rowVersion: 'v1' }
  const liq2 = { id: 'b0000000-0000-0000-0000-000000000002', empleadoId: emp2.id, empleadoNombre: emp2.nombre, empleadoCargo: emp2.cargo, fechaInicio: '2025-02-01', fechaFin: '2025-02-15', diasTrabajados: '11.0000', tarifaAplicada: '55000.0000', totalBruto: '605000.0000', totalAdelantos: '150000.0000', totalNeto: '455000.0000', tienePdf: true, rowVersion: 'v1' }
  const liquidaciones: MockLiquidacion[] = [liq1, liq2]

  const fer1 = { fecha: '2025-01-01', nombre: 'Año Nuevo', tipo: 'Inamovible', origen: 'Oficial' }
  const fer2 = { fecha: '2025-03-24', nombre: 'Día de la Memoria', tipo: 'Inamovible', origen: 'Oficial' }
  const fer3 = { fecha: '2025-04-02', nombre: 'Día del Veterano', tipo: 'Inamovible', origen: 'Oficial' }
  const fer4 = { fecha: '2025-05-01', nombre: 'Día del Trabajador', tipo: 'Inamovible', origen: 'Oficial' }
  const fer5 = { fecha: '2025-05-25', nombre: 'Revolución de Mayo', tipo: 'Inamovible', origen: 'Oficial' }
  const fer6 = { fecha: '2025-06-20', nombre: 'Paso a la Inmortalidad de Belgrano', tipo: 'Inamovible', origen: 'Oficial' }
  const fer7 = { fecha: '2025-07-09', nombre: 'Día de la Independencia', tipo: 'Inamovible', origen: 'Oficial' }
  const fer8 = { fecha: '2025-12-08', nombre: 'Inmaculada Concepción', tipo: 'Inamovible', origen: 'Oficial' }
  const fer9 = { fecha: '2025-12-25', nombre: 'Navidad', tipo: 'Inamovible', origen: 'Oficial' }
  const feriados = [fer1, fer2, fer3, fer4, fer5, fer6, fer7, fer8, fer9]

  return {
    categorias,
    tiposMovimiento,
    clientes,
    proyectos,
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

export function loadMockDb(): MockDb {
  if (typeof window !== 'undefined' && typeof localStorage !== 'undefined') {
    try {
      const stored = localStorage.getItem(DB_STORAGE_KEY)
      if (stored) return JSON.parse(stored)
    } catch {
      // Storage unavailable or parsing error; fallback to fresh seeded mock db.
    }
  }
  const initial = createSeedMockDb()
  saveMockDb(initial)
  return initial
}

export function saveMockDb(db: MockDb): void {
  if (typeof window !== 'undefined' && typeof localStorage !== 'undefined') {
    try {
      localStorage.setItem(DB_STORAGE_KEY, JSON.stringify(db))
    } catch {
      // Storage unavailable or quota exceeded; ignore.
    }
  }
}

export let mockDb: MockDb = loadMockDb()

export function setMockDb(db: MockDb): void {
  mockDb = db
  saveMockDb(db)
}

export function mockAudit(rowVersion: string): { createdAt: string; updatedAt: null; rowVersion: string; isDeleted: boolean; deletedAt: null } {
  return { createdAt: new Date().toISOString(), updatedAt: null, rowVersion, isDeleted: false, deletedAt: null }
}
