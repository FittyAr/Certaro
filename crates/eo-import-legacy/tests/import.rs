//! Integration tests for eo-import-legacy. See `docs/15-migracion-de-datos.md` §8.

use sqlx::sqlite::SqliteConnectOptions;

/// Creates a legacy SQLite database with the C# schema and populates it with test data.
async fn create_legacy_db(scaled: bool) -> (sqlx::SqlitePool, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("legacy.db");
    let options = SqliteConnectOptions::new()
        .filename(&path)
        .create_if_missing(true);
    let pool = sqlx::SqlitePool::connect_with(options).await.unwrap();

    // Create the legacy schema (PascalCase).
    sqlx::query(CREATE_LEGACY_SCHEMA).execute(&pool).await.unwrap();

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
async fn populate_legacy(pool: &sqlx::SqlitePool, scaled: bool) {
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
fn run_import(
    source: &std::path::Path,
    target: &std::path::Path,
    scaled: bool,
    dry_run: bool,
    allow_orphans: bool,
) -> serde_json::Value {
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_eo-import-legacy"));
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
    cmd.arg("--report").arg(target.parent().unwrap().join("import_report.json"));

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
        eprintln!("blocking issues: {}", serde_json::to_string_pretty(&report["blockingIssues"]).unwrap());
        eprintln!("warnings: {}", serde_json::to_string_pretty(&report["warnings"]).unwrap());
    }
    report
}

/// Opens the target database and counts rows in a table.
async fn count_rows(target: &std::path::Path, table: &str) -> i64 {
    let options = SqliteConnectOptions::new().filename(target);
    let pool = sqlx::SqlitePool::connect_with(options).await.unwrap();
    sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
        .fetch_one(&pool)
        .await
        .unwrap()
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn import_base_vacia() {
    let (_legacy, dir) = create_legacy_db(true).await;
    // Don't populate: empty database.
    let target = dir.path().join("target.db");

    let report = run_import(
        &dir.path().join("legacy.db"),
        &target,
        true,
        false,
        false,
    );

    assert_eq!(report["outcome"], "Success");
    // Only seed rows should exist.
    let tipos: i64 = count_rows(&target, "tipos_movimiento").await;
    assert!(tipos >= 4, "expected at least 4 system tipos, got {tipos}");
}

#[tokio::test]
async fn import_base_escalada() {
    let (legacy, dir) = create_legacy_db(true).await;
    populate_legacy(&legacy, true).await;
    let target = dir.path().join("target.db");

    let report = run_import(
        &dir.path().join("legacy.db"),
        &target,
        true,
        false,
        false,
    );

    assert_eq!(report["outcome"], "SuccessWithWarnings");
    // Verify movimientos count.
    let movs: i64 = count_rows(&target, "movimientos").await;
    assert_eq!(movs, 2);
}

#[tokio::test]
async fn import_base_sin_escalar() {
    let (legacy, dir) = create_legacy_db(false).await;
    populate_legacy(&legacy, false).await;
    let target = dir.path().join("target.db");

    let report = run_import(
        &dir.path().join("legacy.db"),
        &target,
        false,
        false,
        false,
    );

    assert_eq!(report["outcome"], "SuccessWithWarnings");
    // Check that ESCALA_SIN_DECIMALES warning exists.
    let warnings = report["warnings"].as_array().unwrap();
    assert!(warnings.iter().any(|w| w["code"] == "ESCALA_SIN_DECIMALES"));
}

#[tokio::test]
async fn pago_escala_mixta() {
    let (legacy, dir) = create_legacy_db(true).await;
    populate_legacy(&legacy, true).await;

    // Add an unscaled payment (raw=4500 against invoice total=121000).
    sqlx::query(
        "INSERT INTO PagosFactura (Id, FacturaId, Fecha, Monto, MedioPago, CreatedAt, UpdatedAt, RowVersion, IsDeleted) \
         VALUES ('51000000-0000-0000-0000-000000000002', '41000000-0000-0000-0000-000000000001', '2026-03-20 00:00:00', 4500, 0, '2026-03-20 15:00:00', '2026-03-20 15:00:00', X'0000000000000001', 0)"
    ).execute(&legacy).await.unwrap();

    let target = dir.path().join("target.db");
    let report = run_import(
        &dir.path().join("legacy.db"),
        &target,
        true,
        false,
        false,
    );

    let warnings = report["warnings"].as_array().unwrap();
    assert!(warnings.iter().any(|w| w["code"] == "PAGO_ESCALA_HEURISTICA"));
}

#[tokio::test]
async fn cotizacion_cero_se_importa_null() {
    let (legacy, dir) = create_legacy_db(true).await;
    populate_legacy(&legacy, true).await;

    // Add a movement with cotizacion = 0.
    sqlx::query(
        "INSERT INTO Movimientos (Id, Fecha, Concepto, Monto, Cantidad, Moneda, CotizacionAplicada, TipoMovimientoId, CreatedAt, UpdatedAt, RowVersion, IsDeleted) \
         VALUES ('91000000-0000-0000-0000-000000000003', '2026-03-05 10:00:00', 'USD test', 100000, 10000, 1, 0, 'a0000000-0000-0000-0000-000000000001', '2026-03-05 10:00:00', '2026-03-05 10:00:00', X'0000000000000001', 0)"
    ).execute(&legacy).await.unwrap();

    let target = dir.path().join("target.db");
    let report = run_import(
        &dir.path().join("legacy.db"),
        &target,
        true,
        false,
        false,
    );

    let warnings = report["warnings"].as_array().unwrap();
    assert!(warnings.iter().any(|w| w["code"] == "COTIZACION_CERO_DESCARTADA"));
}

#[tokio::test]
async fn fecha_civil_no_cambia_de_dia() {
    let (legacy, dir) = create_legacy_db(true).await;
    populate_legacy(&legacy, true).await;
    let target = dir.path().join("target.db");

    run_import(
        &dir.path().join("legacy.db"),
        &target,
        true,
        false,
        false,
    );

    // The asistencia was at 22:30 local. It should be midnight UTC of the same civil day.
    let options = SqliteConnectOptions::new().filename(&target);
    let pool = sqlx::SqlitePool::connect_with(options).await.unwrap();
    let fecha: String = sqlx::query_scalar("SELECT fecha FROM asistencias_empleado WHERE id = '71000000-0000-0000-0000-000000000001'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(fecha.starts_with("2026-03-01"), "expected 2026-03-01, got {fecha}");
    assert!(fecha.contains("00:00:00"), "expected midnight, got {fecha}");
}

#[tokio::test]
async fn fecha_negocio_con_hora_se_convierte() {
    let (legacy, dir) = create_legacy_db(true).await;
    populate_legacy(&legacy, true).await;
    let target = dir.path().join("target.db");

    run_import(
        &dir.path().join("legacy.db"),
        &target,
        true,
        false,
        false,
    );

    // The movimiento was at 10:00 local (UTC-3) = 13:00 UTC.
    let options = SqliteConnectOptions::new().filename(&target);
    let pool = sqlx::SqlitePool::connect_with(options).await.unwrap();
    let fecha: String = sqlx::query_scalar("SELECT fecha FROM movimientos WHERE id = '91000000-0000-0000-0000-000000000001'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(fecha.contains("13:00"), "expected 13:00 UTC, got {fecha}");
}

#[tokio::test]
async fn auditoria_no_se_desplaza() {
    let (legacy, dir) = create_legacy_db(true).await;
    populate_legacy(&legacy, true).await;
    let target = dir.path().join("target.db");

    run_import(
        &dir.path().join("legacy.db"),
        &target,
        true,
        false,
        false,
    );

    // CreatedAt should be preserved as-is (it was already UTC).
    let options = SqliteConnectOptions::new().filename(&target);
    let pool = sqlx::SqlitePool::connect_with(options).await.unwrap();
    let created: String = sqlx::query_scalar("SELECT created_at FROM movimientos WHERE id = '91000000-0000-0000-0000-000000000001'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(created.contains("10:00"), "expected 10:00 UTC, got {created}");
}

#[tokio::test]
async fn cantidad_cero_se_vuelve_uno() {
    let (legacy, dir) = create_legacy_db(true).await;
    populate_legacy(&legacy, true).await;

    // Add a movement with cantidad = 0.
    sqlx::query(
        "INSERT INTO Movimientos (Id, Fecha, Concepto, Monto, Cantidad, Moneda, TipoMovimientoId, CreatedAt, UpdatedAt, RowVersion, IsDeleted) \
         VALUES ('91000000-0000-0000-0000-000000000004', '2026-03-06 10:00:00', 'Test cantidad 0', 50000, 0, 0, 'a0000000-0000-0000-0000-000000000001', '2026-03-06 10:00:00', '2026-03-06 10:00:00', X'0000000000000001', 0)"
    ).execute(&legacy).await.unwrap();

    let target = dir.path().join("target.db");
    run_import(
        &dir.path().join("legacy.db"),
        &target,
        true,
        false,
        false,
    );

    let options = SqliteConnectOptions::new().filename(&target);
    let pool = sqlx::SqlitePool::connect_with(options).await.unwrap();
    let cantidad: i64 = sqlx::query_scalar("SELECT cantidad FROM movimientos WHERE id = '91000000-0000-0000-0000-000000000004'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(cantidad, 10_000, "expected 10000 (=1.0), got {cantidad}");
}

#[tokio::test]
async fn multiplicador_cero_se_vuelve_uno() {
    let (legacy, dir) = create_legacy_db(true).await;
    populate_legacy(&legacy, true).await;

    // Add a liquidation with zero multipliers.
    sqlx::query(
        "INSERT INTO Liquidaciones (Id, EmpleadoId, FechaInicio, FechaFin, DiasTrabajados, TarifaAplicada, IncluirSabados, IncluirDomingos, IncluirFeriados, MultiplicadorSabado, MultiplicadorDomingo, MultiplicadorFeriado, TotalBruto, TotalAdelantos, CreatedAt, UpdatedAt, RowVersion, IsDeleted) \
         VALUES ('81000000-0000-0000-0000-000000000002', '61000000-0000-0000-0000-000000000001', '2026-04-01 00:00:00', '2026-04-15 00:00:00', 100000, 250000, 1, 0, 0, 0, 0, 0, 2500000, 0, '2026-04-16 10:00:00', '2026-04-16 10:00:00', X'0000000000000001', 0)"
    ).execute(&legacy).await.unwrap();

    let target = dir.path().join("target.db");
    run_import(
        &dir.path().join("legacy.db"),
        &target,
        true,
        false,
        false,
    );

    let options = SqliteConnectOptions::new().filename(&target);
    let pool = sqlx::SqlitePool::connect_with(options).await.unwrap();
    let sab: i64 = sqlx::query_scalar("SELECT multiplicador_sabado FROM liquidaciones WHERE id = '81000000-0000-0000-0000-000000000002'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(sab, 10_000, "expected 10000 (=1.0), got {sab}");
}

#[tokio::test]
async fn factura_con_pago_parcial_queda_pagada_parcial() {
    let (legacy, dir) = create_legacy_db(true).await;
    populate_legacy(&legacy, true).await;
    let target = dir.path().join("target.db");

    let _report = run_import(
        &dir.path().join("legacy.db"),
        &target,
        true,
        false,
        false,
    );

    // The invoice had estado=1 (Emitida), total=121000, payment=50000.
    // After reclassification: PagadaParcial (5).
    let options = SqliteConnectOptions::new().filename(&target);
    let pool = sqlx::SqlitePool::connect_with(options).await.unwrap();
    let estado: i64 = sqlx::query_scalar("SELECT estado FROM facturas WHERE id = '41000000-0000-0000-0000-000000000001'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(estado, 5, "expected PagadaParcial (5), got {estado}");
}

#[tokio::test]
async fn email_cliente_se_vuelve_contacto_principal() {
    let (legacy, dir) = create_legacy_db(true).await;
    populate_legacy(&legacy, true).await;
    let target = dir.path().join("target.db");

    run_import(
        &dir.path().join("legacy.db"),
        &target,
        true,
        false,
        false,
    );

    let options = SqliteConnectOptions::new().filename(&target);
    let pool = sqlx::SqlitePool::connect_with(options).await.unwrap();
    // The client had email='acme@example.com'. A principal contact should exist.
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM cliente_contactos WHERE cliente_id = 'd0000000-0000-0000-0000-000000000001' AND es_principal = 1"
    ).fetch_one(&pool).await.unwrap();
    assert!(count >= 1, "expected at least 1 principal contact, got {count}");
}

#[tokio::test]
async fn dry_run_no_escribe() {
    let (legacy, dir) = create_legacy_db(true).await;
    populate_legacy(&legacy, true).await;
    let target = dir.path().join("target.db");

    let report = run_import(
        &dir.path().join("legacy.db"),
        &target,
        true,
        true, // dry_run
        false,
    );

    // The report should say dry_run.
    assert_eq!(report["dryRun"], true);
    // The target should not exist or be empty.
    if target.exists() {
        let movs = count_rows(&target, "movimientos").await;
        assert_eq!(movs, 0, "dry run should not write data");
    }
}

#[tokio::test]
async fn reejecucion_sobre_destino_con_datos_aborta() {
    let (legacy, dir) = create_legacy_db(true).await;
    populate_legacy(&legacy, true).await;
    let target = dir.path().join("target.db");

    // First import.
    run_import(
        &dir.path().join("legacy.db"),
        &target,
        true,
        false,
        false,
    );

    // Second import should fail because target has data.
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_eo-import-legacy"));
    cmd.arg("--source").arg(dir.path().join("legacy.db"));
    cmd.arg("--target").arg(&target);
    cmd.arg("--assume-scaled");
    cmd.arg("--report").arg(target.parent().unwrap().join("import_report2.json"));

    let output = cmd.output().unwrap();
    assert!(!output.status.success(), "second import should fail");
}

#[tokio::test]
async fn tipo_de_sistema_no_se_duplica() {
    let (legacy, dir) = create_legacy_db(true).await;
    populate_legacy(&legacy, true).await;
    let target = dir.path().join("target.db");

    run_import(
        &dir.path().join("legacy.db"),
        &target,
        true,
        false,
        false,
    );

    // System tipos should not be duplicated.
    let options = SqliteConnectOptions::new().filename(&target);
    let pool = sqlx::SqlitePool::connect_with(options).await.unwrap();
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM tipos_movimiento WHERE id = '00000000-0000-0000-0000-000000000001'"
    ).fetch_one(&pool).await.unwrap();
    assert_eq!(count, 1, "system tipo should exist exactly once");
}

#[tokio::test]
async fn cuit_se_normaliza() {
    let (legacy, dir) = create_legacy_db(true).await;
    populate_legacy(&legacy, true).await;
    let target = dir.path().join("target.db");

    run_import(
        &dir.path().join("legacy.db"),
        &target,
        true,
        false,
        false,
    );

    let options = SqliteConnectOptions::new().filename(&target);
    let pool = sqlx::SqlitePool::connect_with(options).await.unwrap();
    let cuit: String = sqlx::query_scalar("SELECT cuit FROM clientes WHERE id = 'd0000000-0000-0000-0000-000000000001'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(cuit, "20123456789", "CUIT should be normalized");
}

#[tokio::test]
async fn color_hex_invalido_se_marca() {
    let (legacy, dir) = create_legacy_db(true).await;
    populate_legacy(&legacy, true).await;

    // Add a category with invalid color.
    sqlx::query(
        "INSERT INTO Categorias (Id, Nombre, Descripcion, ColorHex, Icono, CreatedAt, UpdatedAt, RowVersion, IsDeleted) \
         VALUES ('c0000000-0000-0000-0000-000000000002', 'Invalida', NULL, 'rojo', NULL, '2026-01-10 08:00:00', '2026-01-10 08:00:00', X'0000000000000001', 0)"
    ).execute(&legacy).await.unwrap();

    let target = dir.path().join("target.db");
    let report = run_import(
        &dir.path().join("legacy.db"),
        &target,
        true,
        false,
        false,
    );

    let warnings = report["warnings"].as_array().unwrap();
    assert!(warnings.iter().any(|w| w["code"] == "COLOR_HEX_INVALIDO"));
}

#[tokio::test]
async fn reporte_tiene_estructura_correcta() {
    let (legacy, dir) = create_legacy_db(true).await;
    populate_legacy(&legacy, true).await;
    let target = dir.path().join("target.db");

    let report = run_import(
        &dir.path().join("legacy.db"),
        &target,
        true,
        false,
        false,
    );

    // Verify report structure.
    assert!(report["toolVersion"].is_string());
    assert!(report["startedAt"].is_string());
    assert!(report["finishedAt"].is_string());
    assert!(report["source"].is_object());
    assert!(report["target"].is_object());
    assert!(report["tables"].is_array());
    assert!(report["derived"].is_object());
    assert!(report["warnings"].is_array());
    assert!(report["blockingIssues"].is_array());
    assert!(report["attachments"].is_object());
}

// ── Legacy schema DDL ──────────────────────────────────────────────────────

const CREATE_LEGACY_SCHEMA: &str = r#"
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
