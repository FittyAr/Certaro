export interface MockCategory {
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

export interface MockTipoMovimiento {
  id: string
  nombre: string
  descripcion: string | null
  esIngreso: boolean
  esSistema: boolean
  movimientosCount: number
  puedeEliminarse: boolean
  rowVersion: string
}

export interface MockCliente {
  id: string
  nombre: string
  cuit: string | null
  direccion: string | null
  telefono: string | null
  email: string | null
  condicionIva: string | null
  proyectosCount: number
  facturasCount: number
  deuda: string
  puedeEliminarse: boolean
  rowVersion: string
}

export interface MockProyecto {
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

export interface MockTrabajo {
  id: string
  proyectoId: string
  proyectoNumero: number
  proyectoNombre: string
  clienteId: string
  clienteNombre: string
  descripcion: string
  fechaInicio: string
  fechaFin: string | null
  presupuesto: string
  estado: string
  rowVersion: string
}

export interface MockOrden {
  id: string
  trabajoId: string
  titulo: string
  numeroCertificado: string | null
  fecha: string
  totalCertificados: number
  totalNeto: string
  rowVersion: string
}

export interface MockCertificado {
  id: string
  ordenTrabajoId: string
  ordenTitulo: string
  numero: number
  fecha: string
  totalCertificado: string
  totalNeto: string
  rowVersion: string
}

export interface MockEmpleado {
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

export interface MockFactura {
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

export interface MockMovimiento {
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

export interface MockLiquidacion {
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

export interface MockFeriado {
  fecha: string
  nombre: string
  tipo: string | null
  origen: string
}

export interface MockDb {
  categorias: MockCategory[]
  tiposMovimiento: MockTipoMovimiento[]
  clientes: MockCliente[]
  proyectos: MockProyecto[]
  trabajos: MockTrabajo[]
  ordenes: MockOrden[]
  certificados: MockCertificado[]
  empleados: MockEmpleado[]
  facturas: MockFactura[]
  movimientos: MockMovimiento[]
  liquidaciones: MockLiquidacion[]
  feriados: MockFeriado[]
}
