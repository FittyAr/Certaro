//! Integration tests for eo-import-legacy. See `docs/15-migracion-de-datos.md` §8.

use sqlx::sqlite::SqliteConnectOptions;
use super::schema::CREATE_LEGACY_SCHEMA;

/// Creates a legacy SQLite database with the C# schema and populates it with test data.
pub async fn create_legacy_db(scaled: bool) -> (sqlx::SqlitePool, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("legacy.db");
    let options = SqliteConnectOptions::new()
        .filename(&path)
        .create_if_missing(true);
    let pool = sqlx::SqlitePool::connect_with(options).await.unwrap();

    // Create the legacy schema (PascalCase).
    sqlx::query(CREATE_LEGACY_SCHEMA)
        .execute(&pool)
        .await
        .unwrap();

    // Create migration history.
    sqlx::query("CREATE TABLE IF NOT EXISTS __EFMigrationsHistory (MigrationId TEXT PRIMARY KEY)")
        .execute(&pool)
        .await
        .unwrap();

    if scaled {
        sqlx::query("INSERT INTO __EFMigrationsHistory (MigrationId) VALUES ('20260828214627_RescaleMonetaryValues')")
            .execute(&pool)
            .await
            .unwrap();
    }

    (pool, dir)
}

/// Populates a legacy database with a minimal but complete dataset.
pub async fn populate_legacy(pool: &sqlx::SqlitePool, scaled: bool) {
    let factor: i64 = if scaled { 1 } else { 10_000 };

    // TiposMovimiento (4 system + 1 user).
    for i in 1..=4 {
        sqlx::query(&format!(
            "INSERT INTO TiposMovimiento (Id, Nombre, Descripcion, EsIngreso, EsSistema, CreatedAt, UpdatedAt, RowVersion, IsDeleted) \
             VALUES ('00000000-0000-0000-0000-00000000000{i}', 'Tipo {i}', NULL, {}, 1, '2026-01-01 00:00:00', '2026-01-01 00:00:00', X'0000000000000001', 0)",
            if i <= 2 { 1 } else { 0 }
        )).execute(pool).await.unwrap();
    }
    sqlx::query(
        "INSERT INTO TiposMovimiento (Id, Nombre, Descripcion, EsIngreso, EsSistema, CreatedAt, UpdatedAt, RowVersion, IsDeleted) \
         VALUES ('a0000000-0000-0000-0000-000000000001', 'Consultoría', 'Servicios', 1, 0, '2026-01-15 10:00:00', '2026-01-15 10:00:00', X'0000000000000001', 0)"
    ).execute(pool).await.unwrap();

    // TiposConceptoPago.
    sqlx::query(
        "INSERT INTO TiposConceptoPago (Id, Nombre, EsSistema, CreatedAt, UpdatedAt, RowVersion, IsDeleted) \
         VALUES ('b0000000-0000-0000-0000-000000000001', 'Efectivo', 1, '2026-01-01 00:00:00', '2026-01-01 00:00:00', X'0000000000000001', 0)"
    ).execute(pool).await.unwrap();

    // Categorias.
    sqlx::query(
        "INSERT INTO Categorias (Id, Nombre, Descripcion, ColorHex, Icono, CreatedAt, UpdatedAt, RowVersion, IsDeleted) \
         VALUES ('c0000000-0000-0000-0000-000000000001', 'Materiales', 'Cables y caños', '#FF5733', 'box', '2026-01-10 08:00:00', '2026-01-10 08:00:00', X'0000000000000001', 0)"
    ).execute(pool).await.unwrap();

    // Clientes.
    sqlx::query(
        "INSERT INTO Clientes (Id, Nombre, Cuit, Email, Telefono, Direccion, CondicionIva, CreatedAt, UpdatedAt, RowVersion, IsDeleted) \
         VALUES ('d0000000-0000-0000-0000-000000000001', 'Acme S.A.', '20-12345678-9', 'acme@example.com', '(011) 4567-8901', 'Av. Siempre Viva 742', 'Responsable Inscripto', '2026-01-05 09:00:00', '2026-01-05 09:00:00', X'0000000000000001', 0)"
    ).execute(pool).await.unwrap();

    // ClienteContactos.
    sqlx::query(
        "INSERT INTO ClienteContactos (Id, ClienteId, Email, Etiqueta, CreatedAt, UpdatedAt, RowVersion, IsDeleted) \
         VALUES ('e0000000-0000-0000-0000-000000000001', 'd0000000-0000-0000-0000-000000000001', 'ventas@acme.com', 'Ventas', '2026-01-05 09:00:00', '2026-01-05 09:00:00', X'0000000000000001', 0)"
    ).execute(pool).await.unwrap();

    // Obras.
    sqlx::query(
        "INSERT INTO Obras (Id, Numero, Nombre, Direccion, Localidad, ClienteId, Estado, CreatedAt, UpdatedAt, RowVersion, IsDeleted) \
         VALUES ('f0000000-0000-0000-0000-000000000001', 1, 'Edificio Sur', 'Calle Falsa 123', 'CABA', 'd0000000-0000-0000-0000-000000000001', 0, '2026-01-10 10:00:00', '2026-01-10 10:00:00', X'0000000000000001', 0)"
    ).execute(pool).await.unwrap();

    // Trabajos.
    sqlx::query(&format!(
        "INSERT INTO Trabajos (Id, ObraId, Descripcion, Presupuesto, FechaInicio, FechaFin, Estado, CreatedAt, UpdatedAt, RowVersion, IsDeleted) \
         VALUES ('11000000-0000-0000-0000-000000000001', 'f0000000-0000-0000-0000-000000000001', 'Tablero eléctrico', {}, '2026-02-01 00:00:00', NULL, 0, '2026-01-15 11:00:00', '2026-01-15 11:00:00', X'0000000000000001', 0)",
        500_000 * factor
    )).execute(pool).await.unwrap();

    // OrdenesTrabajo.
    sqlx::query(&format!(
        "INSERT INTO OrdenesTrabajo (Id, TrabajoId, Titulo, Fecha, NumeroCertificado, AjusteUocraPorcentaje, OtrosDescuentos, Observaciones, CreatedAt, UpdatedAt, RowVersion, IsDeleted) \
         VALUES ('21000000-0000-0000-0000-000000000001', '11000000-0000-0000-0000-000000000001', 'OT-001', '2026-02-15 00:00:00', 'C-001', {}, {}, NULL, '2026-02-15 12:00:00', '2026-02-15 12:00:00', X'0000000000000001', 0)",
        300 * factor, 5000 * factor
    )).execute(pool).await.unwrap();

    // OrdenTrabajoItems.
    sqlx::query(&format!(
        "INSERT INTO OrdenTrabajoItems (Id, OrdenTrabajoId, Descripcion, Unidad, Cantidad, PrecioUnitario, PorcentajeAnterior, PorcentajeActual, Ejecutado, Nota, CreatedAt, UpdatedAt, RowVersion, IsDeleted) \
         VALUES ('31000000-0000-0000-0000-000000000001', '21000000-0000-0000-0000-000000000001', 'Cable 2.5mm', 'metro', {}, {}, 0, {}, 0, NULL, '2026-02-15 12:00:00', '2026-02-15 12:00:00', X'0000000000000001', 0)",
        100 * factor, 150 * factor, 5000 * factor
    )).execute(pool).await.unwrap();

    // Facturas.
    sqlx::query(&format!(
        "INSERT INTO Facturas (Id, Numero, ClienteId, Fecha, Subtotal, Iva, Total, Estado, Observaciones, CreatedAt, UpdatedAt, RowVersion, IsDeleted) \
         VALUES ('41000000-0000-0000-0000-000000000001', '0001-00000001', 'd0000000-0000-0000-0000-000000000001', '2026-03-01 00:00:00', {}, {}, {}, 1, NULL, '2026-03-01 14:00:00', '2026-03-01 14:00:00', X'0000000000000001', 0)",
        100_000 * factor, 21_000 * factor, 121_000 * factor
    )).execute(pool).await.unwrap();

    // PagosFactura (partial payment).
    sqlx::query(&format!(
        "INSERT INTO PagosFactura (Id, FacturaId, Fecha, Monto, MedioPago, CreatedAt, UpdatedAt, RowVersion, IsDeleted) \
         VALUES ('51000000-0000-0000-0000-000000000001', '41000000-0000-0000-0000-000000000001', '2026-03-15 00:00:00', {}, 0, '2026-03-15 15:00:00', '2026-03-15 15:00:00', X'0000000000000001', 0)",
        50_000 * factor
    )).execute(pool).await.unwrap();

    // Empleados.
    sqlx::query(&format!(
        "INSERT INTO Empleados (Id, Nombre, Dni, Telefono, Email, Cargo, FechaIngreso, SueldoBase, TarifaDiaria, PagoFrecuencia, Activo, CreatedAt, UpdatedAt, RowVersion, IsDeleted) \
         VALUES ('61000000-0000-0000-0000-000000000001', 'Juan Pérez', '12345678', '11-5555-1234', 'juan@test.com', 'Electricista', '2025-01-01 00:00:00', {}, {}, 2, 1, '2025-01-01 00:00:00', '2025-01-01 00:00:00', X'0000000000000001', 0)",
        500_000 * factor, 25_000 * factor
    )).execute(pool).await.unwrap();

    // AsistenciasEmpleado.
    sqlx::query(
        "INSERT INTO AsistenciasEmpleado (Id, EmpleadoId, TrabajoId, Fecha, TipoJornada, Observaciones, CreatedAt, UpdatedAt, RowVersion, IsDeleted) \
         VALUES ('71000000-0000-0000-0000-000000000001', '61000000-0000-0000-0000-000000000001', '11000000-0000-0000-0000-000000000001', '2026-03-01 22:30:00', 0, NULL, '2026-03-01 22:30:00', '2026-03-01 22:30:00', X'0000000000000001', 0)"
    ).execute(pool).await.unwrap();

    // Liquidaciones.
    sqlx::query(&format!(
        "INSERT INTO Liquidaciones (Id, EmpleadoId, FechaInicio, FechaFin, DiasTrabajados, TarifaAplicada, IncluirSabados, IncluirDomingos, IncluirFeriados, MultiplicadorSabado, MultiplicadorDomingo, MultiplicadorFeriado, TotalBruto, TotalAdelantos, Observaciones, CreatedAt, UpdatedAt, RowVersion, IsDeleted) \
         VALUES ('81000000-0000-0000-0000-000000000001', '61000000-0000-0000-0000-000000000001', '2026-03-01 00:00:00', '2026-03-15 00:00:00', {}, {}, 1, 0, 0, {}, {}, {}, {}, 0, NULL, '2026-03-16 10:00:00', '2026-03-16 10:00:00', X'0000000000000001', 0)",
        10 * factor, 25_000 * factor, 12000 * factor, 10000 * factor, 10000 * factor, 250_000 * factor
    )).execute(pool).await.unwrap();

    // Movimientos.
    sqlx::query(&format!(
        "INSERT INTO Movimientos (Id, Fecha, Concepto, Monto, Cantidad, Moneda, CotizacionAplicada, TipoMovimientoId, TipoConceptoPagoId, CategoriaId, ClienteId, EmpleadoId, TrabajoId, FacturaId, CreatedAt, UpdatedAt, RowVersion, IsDeleted) \
         VALUES ('91000000-0000-0000-0000-000000000001', '2026-03-01 10:00:00', 'Cable 2.5mm', {}, {}, 0, NULL, '00000000-0000-0000-0000-000000000002', NULL, 'c0000000-0000-0000-0000-000000000001', NULL, NULL, NULL, NULL, '2026-03-01 10:00:00', '2026-03-01 10:00:00', X'0000000000000001', 0)",
        15_000 * factor, 10 * factor
    )).execute(pool).await.unwrap();

    // Adelanto movement.
    sqlx::query(&format!(
        "INSERT INTO Movimientos (Id, Fecha, Concepto, Monto, Cantidad, Moneda, CotizacionAplicada, TipoMovimientoId, TipoConceptoPagoId, CategoriaId, ClienteId, EmpleadoId, TrabajoId, FacturaId, CreatedAt, UpdatedAt, RowVersion, IsDeleted) \
         VALUES ('91000000-0000-0000-0000-000000000002', '2026-03-10 00:00:00', 'Adelanto quincena', {}, 10000, 0, NULL, '00000000-0000-0000-0000-000000000003', NULL, NULL, NULL, '61000000-0000-0000-0000-000000000001', NULL, NULL, '2026-03-10 08:00:00', '2026-03-10 08:00:00', X'0000000000000001', 0)",
        10_000 * factor
    )).execute(pool).await.unwrap();

    // Adjuntos.
    sqlx::query(
        "INSERT INTO Adjuntos (Id, EntidadTipo, EntidadId, NombreArchivo, RutaRelativa, Mime, Tamano, CreatedAt, UpdatedAt, RowVersion, IsDeleted) \
         VALUES ('a1000000-0000-0000-0000-000000000001', 'Movimiento', '91000000-0000-0000-0000-000000000001', 'factura.pdf', 'Movimiento/91000000-0000-0000-0000-000000000001/a1000000-0000-0000-0000-000000000001_factura.pdf', 'application/pdf', 12345, '2026-03-01 10:05:00', '2026-03-01 10:05:00', X'0000000000000001', 0)"
    ).execute(pool).await.unwrap();

    // AppMetadata.
    sqlx::query(
        "INSERT INTO AppMetadata (Key, Value, UpdatedAt) VALUES ('app.version', '0.9.0', '2026-01-01 00:00:00')"
    ).execute(pool).await.unwrap();
}

/// Runs the import and returns the report.
pub fn run_import(
    source: &std::path::Path,
    target: &std::path::Path,
    scaled: bool,
    dry_run: bool,
    allow_orphans: bool,
) -> serde_json::Value {
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_certaro-import-legacy"));
    cmd.arg("--source").arg(source);
    cmd.arg("--target").arg(target);
    if scaled {
        cmd.arg("--assume-scaled");
    } else {
        cmd.arg("--assume-unscaled");
    }
    if dry_run {
        cmd.arg("--dry-run");
    }
    if allow_orphans {
        cmd.arg("--allow-orphans");
    }
    cmd.arg("--report")
        .arg(target.parent().unwrap().join("import_report.json"));

    let output = cmd.output().expect("failed to run eo-import-legacy");
    let report_path = target.parent().unwrap().join("import_report.json");
    let report_json = std::fs::read_to_string(&report_path).unwrap_or_else(|_| {
        panic!(
            "report not found at {}. stdout: {}, stderr: {}",
            report_path.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    });
    let report: serde_json::Value = serde_json::from_str(&report_json).unwrap();
    if report["outcome"] == "Rollback" || report["outcome"] == "Aborted" {
        eprintln!("import failed with outcome: {}", report["outcome"]);
        eprintln!(
            "blocking issues: {}",
            serde_json::to_string_pretty(&report["blockingIssues"]).unwrap()
        );
        eprintln!(
            "warnings: {}",
            serde_json::to_string_pretty(&report["warnings"]).unwrap()
        );
    }
    report
}

/// Opens the target database and counts rows in a table.
pub async fn count_rows(target: &std::path::Path, table: &str) -> i64 {
    let options = SqliteConnectOptions::new().filename(target);
    let pool = sqlx::SqlitePool::connect_with(options).await.unwrap();
    sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
        .fetch_one(&pool)
        .await
        .unwrap()
}

// ── Tests ───────────────────────────────────────────────────────────────────
