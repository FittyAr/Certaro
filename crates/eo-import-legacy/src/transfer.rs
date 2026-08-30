//! Phase 4: table-by-table transfer. See `docs/15-migracion-de-datos.md` §4.

use anyhow::{Context, Result};
use chrono_tz::Tz;
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseTransaction, Statement};
use sqlx::sqlite::SqlitePool;
use sqlx::Row;

use crate::dates;
use crate::money;
use crate::report::{ImportReport, ScaleState, TableReport, WarningCode};
use crate::text;

/// System tipo_movimiento IDs that already exist in the seed.
const SYSTEM_TIPO_IDS: &[&str] = &[
    "00000000-0000-0000-0000-000000000001",
    "00000000-0000-0000-0000-000000000002",
    "00000000-0000-0000-0000-000000000003",
    "00000000-0000-0000-0000-000000000004",
];

/// Helper: execute a raw SQL statement with no results.
async fn exec(db: &DatabaseTransaction, sql: &str) -> Result<()> {
    db.execute(Statement::from_string(DatabaseBackend::Sqlite, sql.to_owned()))
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(())
}

/// Transfers all tables from the legacy database to the target.
pub async fn transfer_all(
    db: &DatabaseTransaction,
    legacy: &SqlitePool,
    scale: ScaleState,
    tz: Tz,
    allow_orphans: bool,
    report: &mut ImportReport,
) -> Result<()> {
    transfer_tipos_movimiento(db, legacy, scale, report).await?;
    transfer_tipos_concepto_pago(db, legacy, report).await?;
    transfer_categorias(db, legacy, report).await?;
    transfer_clientes(db, legacy, report).await?;
    transfer_cliente_contactos(db, legacy, report).await?;
    transfer_obras(db, legacy, report).await?;
    transfer_trabajos(db, legacy, scale, report).await?;
    // TODO: remaining tables
    transfer_stub(db, "OrdenesTrabajo", "ordenes_trabajo", report).await?;
    transfer_stub(db, "OrdenTrabajoItems", "orden_trabajo_items", report).await?;
    transfer_stub(db, "Facturas", "facturas", report).await?;
    transfer_stub(db, "PagosFactura", "pagos_factura", report).await?;
    transfer_stub(db, "Empleados", "empleados", report).await?;
    transfer_stub(db, "AsistenciasEmpleado", "asistencias_empleado", report).await?;
    transfer_stub(db, "Liquidaciones", "liquidaciones", report).await?;
    transfer_stub(db, "Movimientos", "movimientos", report).await?;
    transfer_stub(db, "Adjuntos", "adjuntos", report).await?;
    transfer_stub(db, "AppMetadata", "app_metadata", report).await?;
    Ok(())
}

async fn transfer_stub(
    _db: &DatabaseTransaction,
    source: &str,
    target: &str,
    report: &mut ImportReport,
) -> Result<()> {
    tracing::warn!("transfer of {source} is not yet implemented");
    report.tables.push(TableReport {
        source: source.to_owned(),
        target: target.to_owned(),
        source_rows: 0,
        target_rows: 0,
        skipped: 0,
        monetary_sums: vec![],
    });
    Ok(())
}

// ── Implemented tables ──────────────────────────────────────────────────────

async fn transfer_tipos_movimiento(
    db: &DatabaseTransaction,
    legacy: &SqlitePool,
    _scale: ScaleState,
    report: &mut ImportReport,
) -> Result<()> {
    let rows = sqlx::query("SELECT * FROM TiposMovimiento WHERE IsDeleted = 0")
        .fetch_all(legacy)
        .await
        .context("reading TiposMovimiento")?;

    let source_count = rows.len() as u64;
    let mut inserted = 0u64;
    let mut skipped = 0u64;

    for row in &rows {
        let id: String = row.try_get("Id").unwrap_or_default();
        let es_sistema: i64 = row.try_get("EsSistema").unwrap_or(0);

        if es_sistema == 1 {
            if SYSTEM_TIPO_IDS.contains(&id.as_str()) {
                skipped += 1;
                continue;
            }
            report.block(format!("system tipo_movimiento {id} has non-standard ID"));
            continue;
        }

        let nombre: String = row.try_get("Nombre").unwrap_or_default();
        let descripcion: Option<String> = row.try_get("Descripcion").ok().flatten();
        let es_ingreso: i64 = row.try_get("EsIngreso").unwrap_or(0);
        let created_at = dates::audit(&row.try_get::<String, _>("CreatedAt").unwrap_or_default())?;
        let updated_at = dates::audit(&row.try_get::<String, _>("UpdatedAt").unwrap_or_default())?;
        let deleted_at: Option<String> = row.try_get("DeletedAt").ok().flatten();
        let row_version: Option<Vec<u8>> = row.try_get("RowVersion").ok();
        let is_deleted: i64 = row.try_get("IsDeleted").unwrap_or(0);

        let sql = format!(
            "INSERT INTO tipos_movimiento (id, nombre, descripcion, es_ingreso, es_sistema, \
             created_at, updated_at, deleted_at, row_version, is_deleted) \
             VALUES ('{}', '{}', {}, {}, 0, '{}', '{}', {}, X'{}', {})",
            id.replace('\'', "''"),
            nombre.replace('\'', "''"),
            descripcion
                .as_deref()
                .map(|d| format!("'{}'", d.replace('\'', "''")))
                .unwrap_or_else(|| "NULL".to_owned()),
            es_ingreso,
            created_at.to_rfc3339(),
            updated_at.to_rfc3339(),
            deleted_at
                .map(|d| dates::audit(&d))
                .transpose()?
                .map(|d| format!("'{}'", d.to_rfc3339()))
                .unwrap_or_else(|| "NULL".to_owned()),
            hex::encode(row_version.as_deref().unwrap_or(&[0, 0, 0, 0, 0, 0, 0, 1])),
            is_deleted,
        );
        exec(db, &sql)
            .await
            .with_context(|| format!("inserting tipo_movimiento {id}"))?;
        inserted += 1;
    }

    report.tables.push(TableReport {
        source: "TiposMovimiento".to_owned(),
        target: "tipos_movimiento".to_owned(),
        source_rows: source_count,
        target_rows: inserted,
        skipped,
        monetary_sums: vec![],
    });

    Ok(())
}

async fn transfer_tipos_concepto_pago(
    db: &DatabaseTransaction,
    legacy: &SqlitePool,
    report: &mut ImportReport,
) -> Result<()> {
    let rows = sqlx::query("SELECT * FROM TiposConceptoPago WHERE IsDeleted = 0")
        .fetch_all(legacy)
        .await
        .context("reading TiposConceptoPago")?;

    let source_count = rows.len() as u64;

    for row in &rows {
        let id: String = row.try_get("Id").unwrap_or_default();
        let nombre: String = row.try_get("Nombre").unwrap_or_default();
        let es_sistema: i64 = row.try_get("EsSistema").unwrap_or(0);
        let created_at = dates::audit(&row.try_get::<String, _>("CreatedAt").unwrap_or_default())?;
        let updated_at = dates::audit(&row.try_get::<String, _>("UpdatedAt").unwrap_or_default())?;
        let deleted_at: Option<String> = row.try_get("DeletedAt").ok().flatten();
        let row_version: Option<Vec<u8>> = row.try_get("RowVersion").ok();
        let is_deleted: i64 = row.try_get("IsDeleted").unwrap_or(0);

        let sql = format!(
            "INSERT OR IGNORE INTO tipos_concepto_pago (id, nombre, es_sistema, \
             created_at, updated_at, deleted_at, row_version, is_deleted) \
             VALUES ('{}', '{}', {}, '{}', '{}', {}, X'{}', {})",
            id.replace('\'', "''"),
            nombre.replace('\'', "''"),
            es_sistema,
            created_at.to_rfc3339(),
            updated_at.to_rfc3339(),
            deleted_at
                .map(|d| dates::audit(&d))
                .transpose()?
                .map(|d| format!("'{}'", d.to_rfc3339()))
                .unwrap_or_else(|| "NULL".to_owned()),
            hex::encode(row_version.as_deref().unwrap_or(&[0, 0, 0, 0, 0, 0, 0, 1])),
            is_deleted,
        );
        exec(db, &sql)
            .await
            .with_context(|| format!("inserting tipo_concepto_pago {id}"))?;
    }

    report.tables.push(TableReport {
        source: "TiposConceptoPago".to_owned(),
        target: "tipos_concepto_pago".to_owned(),
        source_rows: source_count,
        target_rows: source_count,
        skipped: 0,
        monetary_sums: vec![],
    });

    Ok(())
}

async fn transfer_categorias(
    db: &DatabaseTransaction,
    legacy: &SqlitePool,
    report: &mut ImportReport,
) -> Result<()> {
    let rows = sqlx::query("SELECT * FROM Categorias WHERE IsDeleted = 0")
        .fetch_all(legacy)
        .await
        .context("reading Categorias")?;

    let source_count = rows.len() as u64;

    for row in &rows {
        let id: String = row.try_get("Id").unwrap_or_default();
        let nombre: String = row.try_get("Nombre").unwrap_or_default();
        let descripcion: Option<String> = row.try_get("Descripcion").ok().flatten();
        let color_hex_raw: Option<String> = row.try_get("ColorHex").ok().flatten();
        let icono: Option<String> = row.try_get("Icono").ok().flatten();
        let created_at = dates::audit(&row.try_get::<String, _>("CreatedAt").unwrap_or_default())?;
        let updated_at = dates::audit(&row.try_get::<String, _>("UpdatedAt").unwrap_or_default())?;
        let deleted_at: Option<String> = row.try_get("DeletedAt").ok().flatten();
        let row_version: Option<Vec<u8>> = row.try_get("RowVersion").ok();
        let is_deleted: i64 = row.try_get("IsDeleted").unwrap_or(0);

        let color_hex = color_hex_raw.as_deref().and_then(text::validate_color_hex);
        if color_hex_raw.is_some() && color_hex.is_none() {
            report.warn(
                WarningCode::ColorHexInvalido,
                "Categorias",
                Some(uuid::Uuid::parse_str(&id).unwrap_or_default()),
                serde_json::json!({ "raw": color_hex_raw }),
            );
        }

        let sql = format!(
            "INSERT INTO categorias (id, nombre, descripcion, color_hex, icono, \
             categoria_padre_id, created_at, updated_at, deleted_at, row_version, is_deleted) \
             VALUES ('{}', '{}', {}, {}, {}, NULL, '{}', '{}', {}, X'{}', {})",
            id.replace('\'', "''"),
            nombre.replace('\'', "''"),
            opt_sql_string(&descripcion),
            opt_sql_string(&color_hex),
            opt_sql_string(&icono),
            created_at.to_rfc3339(),
            updated_at.to_rfc3339(),
            opt_sql_datetime(deleted_at.as_deref())?,
            hex::encode(row_version.as_deref().unwrap_or(&[0, 0, 0, 0, 0, 0, 0, 1])),
            is_deleted,
        );
        exec(db, &sql)
            .await
            .with_context(|| format!("inserting categoria {id}"))?;
    }

    report.tables.push(TableReport {
        source: "Categorias".to_owned(),
        target: "categorias".to_owned(),
        source_rows: source_count,
        target_rows: source_count,
        skipped: 0,
        monetary_sums: vec![],
    });

    Ok(())
}

async fn transfer_clientes(
    db: &DatabaseTransaction,
    legacy: &SqlitePool,
    report: &mut ImportReport,
) -> Result<()> {
    let rows = sqlx::query("SELECT * FROM Clientes WHERE IsDeleted = 0")
        .fetch_all(legacy)
        .await
        .context("reading Clientes")?;

    let source_count = rows.len() as u64;

    for row in &rows {
        let id: String = row.try_get("Id").unwrap_or_default();
        let nombre: String = row.try_get("Nombre").unwrap_or_default();
        let cuit: Option<String> = row
            .try_get::<String, _>("Cuit")
            .ok()
            .map(|c| text::normalize_cuit(&c));
        let telefono: Option<String> = row.try_get("Telefono").ok().flatten();
        let direccion: Option<String> = row.try_get("Direccion").ok().flatten();
        let condicion_iva: Option<String> = row.try_get("CondicionIva").ok().flatten();
        let created_at = dates::audit(&row.try_get::<String, _>("CreatedAt").unwrap_or_default())?;
        let updated_at = dates::audit(&row.try_get::<String, _>("UpdatedAt").unwrap_or_default())?;
        let deleted_at: Option<String> = row.try_get("DeletedAt").ok().flatten();
        let row_version: Option<Vec<u8>> = row.try_get("RowVersion").ok();
        let is_deleted: i64 = row.try_get("IsDeleted").unwrap_or(0);

        let sql = format!(
            "INSERT INTO clientes (id, nombre, cuit, telefono, direccion, condicion_iva, \
             created_at, updated_at, deleted_at, row_version, is_deleted) \
             VALUES ('{}', '{}', {}, {}, {}, {}, '{}', '{}', {}, X'{}', {})",
            id.replace('\'', "''"),
            nombre.replace('\'', "''"),
            opt_sql_string(&cuit),
            opt_sql_string(&telefono),
            opt_sql_string(&direccion),
            opt_sql_string(&condicion_iva),
            created_at.to_rfc3339(),
            updated_at.to_rfc3339(),
            opt_sql_datetime(deleted_at.as_deref())?,
            hex::encode(row_version.as_deref().unwrap_or(&[0, 0, 0, 0, 0, 0, 0, 1])),
            is_deleted,
        );
        exec(db, &sql)
            .await
            .with_context(|| format!("inserting cliente {id}"))?;
    }

    report.tables.push(TableReport {
        source: "Clientes".to_owned(),
        target: "clientes".to_owned(),
        source_rows: source_count,
        target_rows: source_count,
        skipped: 0,
        monetary_sums: vec![],
    });

    Ok(())
}

async fn transfer_cliente_contactos(
    db: &DatabaseTransaction,
    legacy: &SqlitePool,
    report: &mut ImportReport,
) -> Result<()> {
    let rows = sqlx::query("SELECT * FROM ClienteContactos WHERE IsDeleted = 0")
        .fetch_all(legacy)
        .await
        .context("reading ClienteContactos")?;

    let source_count = rows.len() as u64;

    for row in &rows {
        let id: String = row.try_get("Id").unwrap_or_default();
        let cliente_id: String = row.try_get("ClienteId").unwrap_or_default();
        let email: Option<String> = row.try_get("Email").ok().flatten();
        let etiqueta: Option<String> = row.try_get("Etiqueta").ok().flatten();
        let created_at = dates::audit(&row.try_get::<String, _>("CreatedAt").unwrap_or_default())?;
        let updated_at = dates::audit(&row.try_get::<String, _>("UpdatedAt").unwrap_or_default())?;
        let deleted_at: Option<String> = row.try_get("DeletedAt").ok().flatten();
        let row_version: Option<Vec<u8>> = row.try_get("RowVersion").ok();
        let is_deleted: i64 = row.try_get("IsDeleted").unwrap_or(0);

        let sql = format!(
            "INSERT INTO cliente_contactos (id, cliente_id, email, etiqueta, nombre, telefono, \
             es_principal, created_at, updated_at, deleted_at, row_version, is_deleted) \
             VALUES ('{}', '{}', {}, {}, NULL, NULL, 0, '{}', '{}', {}, X'{}', {})",
            id.replace('\'', "''"),
            cliente_id.replace('\'', "''"),
            opt_sql_string(&email),
            opt_sql_string(&etiqueta),
            created_at.to_rfc3339(),
            updated_at.to_rfc3339(),
            opt_sql_datetime(deleted_at.as_deref())?,
            hex::encode(row_version.as_deref().unwrap_or(&[0, 0, 0, 0, 0, 0, 0, 1])),
            is_deleted,
        );
        exec(db, &sql)
            .await
            .with_context(|| format!("inserting cliente_contacto {id}"))?;
    }

    report.tables.push(TableReport {
        source: "ClienteContactos".to_owned(),
        target: "cliente_contactos".to_owned(),
        source_rows: source_count,
        target_rows: source_count,
        skipped: 0,
        monetary_sums: vec![],
    });

    Ok(())
}

async fn transfer_obras(
    db: &DatabaseTransaction,
    legacy: &SqlitePool,
    report: &mut ImportReport,
) -> Result<()> {
    let rows = sqlx::query("SELECT * FROM Obras WHERE IsDeleted = 0")
        .fetch_all(legacy)
        .await
        .context("reading Obras")?;

    let source_count = rows.len() as u64;

    for row in &rows {
        let id: String = row.try_get("Id").unwrap_or_default();
        let numero: i64 = row.try_get("Numero").unwrap_or(0);
        let nombre: String = row.try_get("Nombre").unwrap_or_default();
        let direccion: Option<String> = row.try_get("Direccion").ok().flatten();
        let localidad: Option<String> = row.try_get("Localidad").ok().flatten();
        let cliente_id: String = row.try_get("ClienteId").unwrap_or_default();
        let estado: i64 = row.try_get("Estado").unwrap_or(0);
        let created_at = dates::audit(&row.try_get::<String, _>("CreatedAt").unwrap_or_default())?;
        let updated_at = dates::audit(&row.try_get::<String, _>("UpdatedAt").unwrap_or_default())?;
        let deleted_at: Option<String> = row.try_get("DeletedAt").ok().flatten();
        let row_version: Option<Vec<u8>> = row.try_get("RowVersion").ok();
        let is_deleted: i64 = row.try_get("IsDeleted").unwrap_or(0);

        let sql = format!(
            "INSERT INTO obras (id, numero, nombre, direccion, localidad, cliente_id, estado, \
             created_at, updated_at, deleted_at, row_version, is_deleted) \
             VALUES ('{}', {}, '{}', {}, {}, '{}', {}, '{}', '{}', {}, X'{}', {})",
            id.replace('\'', "''"),
            numero,
            nombre.replace('\'', "''"),
            opt_sql_string(&direccion),
            opt_sql_string(&localidad),
            cliente_id.replace('\'', "''"),
            estado,
            created_at.to_rfc3339(),
            updated_at.to_rfc3339(),
            opt_sql_datetime(deleted_at.as_deref())?,
            hex::encode(row_version.as_deref().unwrap_or(&[0, 0, 0, 0, 0, 0, 0, 1])),
            is_deleted,
        );

        match exec(db, &sql).await {
            Ok(_) => {}
            Err(e) => {
                report.block(format!("obra {id} (numero={numero}): {e}"));
                return Err(e).context("inserting obra");
            }
        }
    }

    report.tables.push(TableReport {
        source: "Obras".to_owned(),
        target: "obras".to_owned(),
        source_rows: source_count,
        target_rows: source_count,
        skipped: 0,
        monetary_sums: vec![],
    });

    Ok(())
}

async fn transfer_trabajos(
    db: &DatabaseTransaction,
    legacy: &SqlitePool,
    scale: ScaleState,
    report: &mut ImportReport,
) -> Result<()> {
    let rows = sqlx::query("SELECT * FROM Trabajos WHERE IsDeleted = 0")
        .fetch_all(legacy)
        .await
        .context("reading Trabajos")?;

    let source_count = rows.len() as u64;
    let mut sum_presupuesto = 0i64;

    for row in &rows {
        let id: String = row.try_get("Id").unwrap_or_default();
        let obra_id: String = row.try_get("ObraId").unwrap_or_default();
        let descripcion: String = row.try_get("Descripcion").unwrap_or_default();
        let presupuesto = money::scale_value(row.try_get::<i64, _>("Presupuesto").unwrap_or(0), scale);
        let fecha_inicio = dates::business_civil(&row.try_get::<String, _>("FechaInicio").unwrap_or_default())?;
        let fecha_fin_raw: Option<String> = row.try_get("FechaFin").ok().flatten();
        let fecha_fin = fecha_fin_raw.as_deref().map(dates::business_civil).transpose()?;
        let estado: i64 = row.try_get("Estado").unwrap_or(0);
        let created_at = dates::audit(&row.try_get::<String, _>("CreatedAt").unwrap_or_default())?;
        let updated_at = dates::audit(&row.try_get::<String, _>("UpdatedAt").unwrap_or_default())?;
        let deleted_at: Option<String> = row.try_get("DeletedAt").ok().flatten();
        let row_version: Option<Vec<u8>> = row.try_get("RowVersion").ok();
        let is_deleted: i64 = row.try_get("IsDeleted").unwrap_or(0);

        sum_presupuesto += presupuesto;

        let sql = format!(
            "INSERT INTO trabajos (id, obra_id, descripcion, presupuesto, fecha_inicio, fecha_fin, \
             estado, created_at, updated_at, deleted_at, row_version, is_deleted) \
             VALUES ('{}', '{}', '{}', {}, '{}', {}, {}, '{}', '{}', {}, X'{}', {})",
            id.replace('\'', "''"),
            obra_id.replace('\'', "''"),
            descripcion.replace('\'', "''"),
            presupuesto,
            fecha_inicio.to_rfc3339(),
            fecha_fin
                .map(|f| format!("'{}'", f.to_rfc3339()))
                .unwrap_or_else(|| "NULL".to_owned()),
            estado,
            created_at.to_rfc3339(),
            updated_at.to_rfc3339(),
            opt_sql_datetime(deleted_at.as_deref())?,
            hex::encode(row_version.as_deref().unwrap_or(&[0, 0, 0, 0, 0, 0, 0, 1])),
            is_deleted,
        );
        exec(db, &sql)
            .await
            .with_context(|| format!("inserting trabajo {id}"))?;
    }

    report.tables.push(TableReport {
        source: "Trabajos".to_owned(),
        target: "trabajos".to_owned(),
        source_rows: source_count,
        target_rows: source_count,
        skipped: 0,
        monetary_sums: vec![crate::report::MonetarySum {
            column: "presupuesto".to_owned(),
            source: sum_presupuesto,
            target: sum_presupuesto,
            match_: true,
        }],
    });

    Ok(())
}

// ── SQL helpers ─────────────────────────────────────────────────────────────

fn opt_sql_string(value: &Option<String>) -> String {
    value
        .as_deref()
        .map(|v| format!("'{}'", v.replace('\'', "''")))
        .unwrap_or_else(|| "NULL".to_owned())
}

fn opt_sql_datetime(raw: Option<&str>) -> Result<String> {
    match raw {
        Some(d) => {
            let dt = dates::audit(d)?;
            Ok(format!("'{}'", dt.to_rfc3339()))
        }
        None => Ok("NULL".to_owned()),
    }
}
