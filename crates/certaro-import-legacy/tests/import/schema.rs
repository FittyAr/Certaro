// ── Legacy schema DDL ──────────────────────────────────────────────────────

pub const CREATE_LEGACY_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS TiposMovimiento (
    Id TEXT PRIMARY KEY,
    Nombre TEXT NOT NULL,
    Descripcion TEXT,
    EsIngreso INTEGER NOT NULL DEFAULT 0,
    EsSistema INTEGER NOT NULL DEFAULT 0,
    CreatedAt TEXT NOT NULL,
    UpdatedAt TEXT NOT NULL,
    DeletedAt TEXT,
    RowVersion BLOB,
    IsDeleted INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS TiposConceptoPago (
    Id TEXT PRIMARY KEY,
    Nombre TEXT NOT NULL,
    EsSistema INTEGER NOT NULL DEFAULT 0,
    CreatedAt TEXT NOT NULL,
    UpdatedAt TEXT NOT NULL,
    DeletedAt TEXT,
    RowVersion BLOB,
    IsDeleted INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS Categorias (
    Id TEXT PRIMARY KEY,
    Nombre TEXT NOT NULL,
    Descripcion TEXT,
    ColorHex TEXT,
    Icono TEXT,
    CreatedAt TEXT NOT NULL,
    UpdatedAt TEXT NOT NULL,
    DeletedAt TEXT,
    RowVersion BLOB,
    IsDeleted INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS Clientes (
    Id TEXT PRIMARY KEY,
    Nombre TEXT NOT NULL,
    Cuit TEXT,
    Email TEXT,
    Telefono TEXT,
    Direccion TEXT,
    CondicionIva TEXT,
    CreatedAt TEXT NOT NULL,
    UpdatedAt TEXT NOT NULL,
    DeletedAt TEXT,
    RowVersion BLOB,
    IsDeleted INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS ClienteContactos (
    Id TEXT PRIMARY KEY,
    ClienteId TEXT NOT NULL,
    Email TEXT,
    Etiqueta TEXT,
    CreatedAt TEXT NOT NULL,
    UpdatedAt TEXT NOT NULL,
    DeletedAt TEXT,
    RowVersion BLOB,
    IsDeleted INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS Obras (
    Id TEXT PRIMARY KEY,
    Numero INTEGER NOT NULL,
    Nombre TEXT NOT NULL,
    Direccion TEXT,
    Localidad TEXT,
    ClienteId TEXT NOT NULL,
    Estado INTEGER NOT NULL DEFAULT 0,
    CreatedAt TEXT NOT NULL,
    UpdatedAt TEXT NOT NULL,
    DeletedAt TEXT,
    RowVersion BLOB,
    IsDeleted INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS Trabajos (
    Id TEXT PRIMARY KEY,
    ObraId TEXT NOT NULL,
    Descripcion TEXT NOT NULL,
    Presupuesto INTEGER NOT NULL DEFAULT 0,
    FechaInicio TEXT NOT NULL,
    FechaFin TEXT,
    Estado INTEGER NOT NULL DEFAULT 0,
    CreatedAt TEXT NOT NULL,
    UpdatedAt TEXT NOT NULL,
    DeletedAt TEXT,
    RowVersion BLOB,
    IsDeleted INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS OrdenesTrabajo (
    Id TEXT PRIMARY KEY,
    TrabajoId TEXT NOT NULL,
    Titulo TEXT NOT NULL,
    Fecha TEXT NOT NULL,
    NumeroCertificado TEXT,
    AjusteUocraPorcentaje INTEGER NOT NULL DEFAULT 0,
    OtrosDescuentos INTEGER NOT NULL DEFAULT 0,
    Observaciones TEXT,
    CreatedAt TEXT NOT NULL,
    UpdatedAt TEXT NOT NULL,
    DeletedAt TEXT,
    RowVersion BLOB,
    IsDeleted INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS OrdenTrabajoItems (
    Id TEXT PRIMARY KEY,
    OrdenTrabajoId TEXT NOT NULL,
    Descripcion TEXT NOT NULL,
    Unidad TEXT NOT NULL,
    Cantidad INTEGER NOT NULL DEFAULT 0,
    PrecioUnitario INTEGER NOT NULL DEFAULT 0,
    PorcentajeAnterior INTEGER NOT NULL DEFAULT 0,
    PorcentajeActual INTEGER NOT NULL DEFAULT 0,
    Ejecutado INTEGER NOT NULL DEFAULT 0,
    Nota TEXT,
    CreatedAt TEXT NOT NULL,
    UpdatedAt TEXT NOT NULL,
    DeletedAt TEXT,
    RowVersion BLOB,
    IsDeleted INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS Facturas (
    Id TEXT PRIMARY KEY,
    Numero TEXT NOT NULL,
    ClienteId TEXT NOT NULL,
    Fecha TEXT NOT NULL,
    Subtotal INTEGER NOT NULL DEFAULT 0,
    Iva INTEGER NOT NULL DEFAULT 0,
    Total INTEGER NOT NULL DEFAULT 0,
    Estado INTEGER NOT NULL DEFAULT 0,
    Observaciones TEXT,
    CreatedAt TEXT NOT NULL,
    UpdatedAt TEXT NOT NULL,
    DeletedAt TEXT,
    RowVersion BLOB,
    IsDeleted INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS PagosFactura (
    Id TEXT PRIMARY KEY,
    FacturaId TEXT NOT NULL,
    Fecha TEXT NOT NULL,
    Monto INTEGER NOT NULL DEFAULT 0,
    MedioPago INTEGER NOT NULL DEFAULT 0,
    CreatedAt TEXT NOT NULL,
    UpdatedAt TEXT NOT NULL,
    DeletedAt TEXT,
    RowVersion BLOB,
    IsDeleted INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS Empleados (
    Id TEXT PRIMARY KEY,
    Nombre TEXT NOT NULL,
    Dni TEXT,
    Telefono TEXT,
    Email TEXT,
    Cargo TEXT,
    FechaIngreso TEXT NOT NULL,
    SueldoBase INTEGER NOT NULL DEFAULT 0,
    TarifaDiaria INTEGER NOT NULL DEFAULT 0,
    PagoFrecuencia INTEGER NOT NULL DEFAULT 0,
    Activo INTEGER NOT NULL DEFAULT 1,
    CreatedAt TEXT NOT NULL,
    UpdatedAt TEXT NOT NULL,
    DeletedAt TEXT,
    RowVersion BLOB,
    IsDeleted INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS AsistenciasEmpleado (
    Id TEXT PRIMARY KEY,
    EmpleadoId TEXT NOT NULL,
    TrabajoId TEXT,
    Fecha TEXT NOT NULL,
    TipoJornada INTEGER NOT NULL DEFAULT 0,
    Observaciones TEXT,
    CreatedAt TEXT NOT NULL,
    UpdatedAt TEXT NOT NULL,
    DeletedAt TEXT,
    RowVersion BLOB,
    IsDeleted INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS Liquidaciones (
    Id TEXT PRIMARY KEY,
    EmpleadoId TEXT NOT NULL,
    FechaInicio TEXT NOT NULL,
    FechaFin TEXT NOT NULL,
    DiasTrabajados INTEGER NOT NULL DEFAULT 0,
    TarifaAplicada INTEGER NOT NULL DEFAULT 0,
    IncluirSabados INTEGER NOT NULL DEFAULT 0,
    IncluirDomingos INTEGER NOT NULL DEFAULT 0,
    IncluirFeriados INTEGER NOT NULL DEFAULT 0,
    MultiplicadorSabado INTEGER NOT NULL DEFAULT 0,
    MultiplicadorDomingo INTEGER NOT NULL DEFAULT 0,
    MultiplicadorFeriado INTEGER NOT NULL DEFAULT 0,
    TotalBruto INTEGER NOT NULL DEFAULT 0,
    TotalAdelantos INTEGER NOT NULL DEFAULT 0,
    Observaciones TEXT,
    CreatedAt TEXT NOT NULL,
    UpdatedAt TEXT NOT NULL,
    DeletedAt TEXT,
    RowVersion BLOB,
    IsDeleted INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS Movimientos (
    Id TEXT PRIMARY KEY,
    Fecha TEXT NOT NULL,
    Concepto TEXT NOT NULL,
    Monto INTEGER NOT NULL DEFAULT 0,
    Cantidad INTEGER NOT NULL DEFAULT 0,
    Moneda INTEGER NOT NULL DEFAULT 0,
    CotizacionAplicada INTEGER,
    TipoMovimientoId TEXT NOT NULL,
    TipoConceptoPagoId TEXT,
    CategoriaId TEXT,
    ClienteId TEXT,
    EmpleadoId TEXT,
    TrabajoId TEXT,
    FacturaId TEXT,
    CreatedAt TEXT NOT NULL,
    UpdatedAt TEXT NOT NULL,
    DeletedAt TEXT,
    RowVersion BLOB,
    IsDeleted INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS Adjuntos (
    Id TEXT PRIMARY KEY,
    EntidadTipo TEXT NOT NULL,
    EntidadId TEXT NOT NULL,
    NombreArchivo TEXT NOT NULL,
    RutaRelativa TEXT NOT NULL,
    Mime TEXT NOT NULL,
    Tamano INTEGER NOT NULL DEFAULT 0,
    CreatedAt TEXT NOT NULL,
    UpdatedAt TEXT NOT NULL,
    DeletedAt TEXT,
    RowVersion BLOB,
    IsDeleted INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS AppMetadata (
    Key TEXT PRIMARY KEY,
    Value TEXT,
    UpdatedAt TEXT NOT NULL
);
"#;
