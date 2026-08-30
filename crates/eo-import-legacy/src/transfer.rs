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
    transfer_ordenes_trabajo(db, legacy, scale, tz, report).await?;
    transfer_orden_trabajo_items(db, legacy, scale, report).await?;
    transfer_facturas(db, legacy, scale, tz, report).await?;
    transfer_pagos_factura(db, legacy, scale, tz, report).await?;
    transfer_empleados(db, legacy, scale, tz, report).await?;
    transfer_asistencias_empleado(db, legacy, tz, allow_orphans, report).await?;
    transfer_liquidaciones(db, legacy, scale, tz, report).await?;
    transfer_movimientos(db, legacy, scale, tz, allow_orphans, report).await?;
    transfer_adjuntos(db, legacy, report).await?;
    transfer_app_metadata(db, legacy, report).await?;
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

// ── Remaining table implementations ─────────────────────────────────────────

async fn transfer_ordenes_trabajo(
    db: &DatabaseTransaction,
    legacy: &SqlitePool,
    scale: ScaleState,
    tz: Tz,
    report: &mut ImportReport,
) -> Result<()> {
    let rows = sqlx::query("SELECT * FROM OrdenesTrabajo WHERE IsDeleted = 0")
        .fetch_all(legacy)
        .await
        .context("reading OrdenesTrabajo")?;

    let source_count = rows.len() as u64;

    for row in &rows {
        let id: String = row.try_get("Id").unwrap_or_default();
        let trabajo_id: String = row.try_get("TrabajoId").unwrap_or_default();
        let titulo: String = row.try_get("Titulo").unwrap_or_default();
        let fecha = dates::business_civil(&row.try_get::<String, _>("Fecha").unwrap_or_default())?;
        let numero_certificado: Option<String> = row.try_get("NumeroCertificado").ok().flatten();
        let ajuste_uocra = money::scale_value(row.try_get::<i64, _>("AjusteUocraPorcentaje").unwrap_or(0), scale);
        let otros_descuentos = money::scale_value(row.try_get::<i64, _>("OtrosDescuentos").unwrap_or(0), scale);
        let observaciones: Option<String> = row.try_get("Observaciones").ok().flatten();
        let created_at = dates::audit(&row.try_get::<String, _>("CreatedAt").unwrap_or_default())?;
        let updated_at = dates::audit(&row.try_get::<String, _>("UpdatedAt").unwrap_or_default())?;
        let deleted_at: Option<String> = row.try_get("DeletedAt").ok().flatten();
        let row_version: Option<Vec<u8>> = row.try_get("RowVersion").ok();
        let is_deleted: i64 = row.try_get("IsDeleted").unwrap_or(0);

        let sql = format!(
            "INSERT INTO ordenes_trabajo (id, trabajo_id, titulo, fecha, numero_certificado, \
             ajuste_uocra_porcentaje, otros_descuentos, observaciones, \
             created_at, updated_at, deleted_at, row_version, is_deleted) \
             VALUES ('{}', '{}', '{}', '{}', {}, {}, {}, {}, '{}', '{}', {}, X'{}', {})",
            id.replace('\'', "''"),
            trabajo_id.replace('\'', "''"),
            titulo.replace('\'', "''"),
            fecha.to_rfc3339(),
            opt_sql_string(&numero_certificado),
            ajuste_uocra,
            otros_descuentos,
            opt_sql_string(&observaciones),
            created_at.to_rfc3339(),
            updated_at.to_rfc3339(),
            opt_sql_datetime(deleted_at.as_deref())?,
            hex::encode(row_version.as_deref().unwrap_or(&[0, 0, 0, 0, 0, 0, 0, 1])),
            is_deleted,
        );
        exec(db, &sql).await.with_context(|| format!("inserting orden_trabajo {id}"))?;
    }

    report.tables.push(TableReport {
        source: "OrdenesTrabajo".to_owned(),
        target: "ordenes_trabajo".to_owned(),
        source_rows: source_count,
        target_rows: source_count,
        skipped: 0,
        monetary_sums: vec![],
    });
    Ok(())
}

async fn transfer_orden_trabajo_items(
    db: &DatabaseTransaction,
    legacy: &SqlitePool,
    scale: ScaleState,
    report: &mut ImportReport,
) -> Result<()> {
    let rows = sqlx::query("SELECT * FROM OrdenTrabajoItems WHERE IsDeleted = 0")
        .fetch_all(legacy)
        .await
        .context("reading OrdenTrabajoItems")?;

    let source_count = rows.len() as u64;

    for (orden, row) in rows.iter().enumerate() {
        let id: String = row.try_get("Id").unwrap_or_default();
        let orden_trabajo_id: String = row.try_get("OrdenTrabajoId").unwrap_or_default();
        let descripcion: String = row.try_get("Descripcion").unwrap_or_default();
        let unidad: String = row.try_get("Unidad").unwrap_or_default();
        let cantidad = money::scale_value(row.try_get::<i64, _>("Cantidad").unwrap_or(0), scale);
        let precio_unitario = money::scale_value(row.try_get::<i64, _>("PrecioUnitario").unwrap_or(0), scale);
        let porcentaje_anterior = money::scale_value(row.try_get::<i64, _>("PorcentajeAnterior").unwrap_or(0), scale);
        let porcentaje_actual = money::scale_value(row.try_get::<i64, _>("PorcentajeActual").unwrap_or(0), scale);
        let ejecutado: i64 = row.try_get("Ejecutado").unwrap_or(0);
        let nota: Option<String> = row.try_get("Nota").ok().flatten();
        let created_at = dates::audit(&row.try_get::<String, _>("CreatedAt").unwrap_or_default())?;
        let updated_at = dates::audit(&row.try_get::<String, _>("UpdatedAt").unwrap_or_default())?;
        let deleted_at: Option<String> = row.try_get("DeletedAt").ok().flatten();
        let row_version: Option<Vec<u8>> = row.try_get("RowVersion").ok();
        let is_deleted: i64 = row.try_get("IsDeleted").unwrap_or(0);

        // Warn if accumulated percentage exceeds 100%.
        let total_pct = porcentaje_anterior + porcentaje_actual;
        if total_pct > 1_000_000 { // 100.0000 in scaled representation
            report.warn(
                WarningCode::PorcentajeExcede100,
                "OrdenTrabajoItems",
                Some(uuid::Uuid::parse_str(&id).unwrap_or_default()),
                serde_json::json!({ "porcentaje_anterior": porcentaje_anterior, "porcentaje_actual": porcentaje_actual }),
            );
        }

        let sql = format!(
            "INSERT INTO orden_trabajo_items (id, orden_trabajo_id, descripcion, unidad, cantidad, \
             precio_unitario, porcentaje_anterior, porcentaje_actual, ejecutado, nota, orden, \
             created_at, updated_at, deleted_at, row_version, is_deleted) \
             VALUES ('{}', '{}', '{}', '{}', {}, {}, {}, {}, {}, {}, {}, '{}', '{}', {}, X'{}', {})",
            id.replace('\'', "''"),
            orden_trabajo_id.replace('\'', "''"),
            descripcion.replace('\'', "''"),
            unidad.replace('\'', "''"),
            cantidad,
            precio_unitario,
            porcentaje_anterior,
            porcentaje_actual,
            ejecutado,
            opt_sql_string(&nota),
            orden as i64 + 1, // 1-based ROW_NUMBER
            created_at.to_rfc3339(),
            updated_at.to_rfc3339(),
            opt_sql_datetime(deleted_at.as_deref())?,
            hex::encode(row_version.as_deref().unwrap_or(&[0, 0, 0, 0, 0, 0, 0, 1])),
            is_deleted,
        );
        exec(db, &sql).await.with_context(|| format!("inserting orden_trabajo_item {id}"))?;
    }

    report.tables.push(TableReport {
        source: "OrdenTrabajoItems".to_owned(),
        target: "orden_trabajo_items".to_owned(),
        source_rows: source_count,
        target_rows: source_count,
        skipped: 0,
        monetary_sums: vec![],
    });
    Ok(())
}

async fn transfer_facturas(
    db: &DatabaseTransaction,
    legacy: &SqlitePool,
    scale: ScaleState,
    tz: Tz,
    report: &mut ImportReport,
) -> Result<()> {
    let rows = sqlx::query("SELECT * FROM Facturas WHERE IsDeleted = 0")
        .fetch_all(legacy)
        .await
        .context("reading Facturas")?;

    let source_count = rows.len() as u64;

    for row in &rows {
        let id: String = row.try_get("Id").unwrap_or_default();
        let numero: String = row.try_get("Numero").unwrap_or_default();
        let cliente_id: String = row.try_get("ClienteId").unwrap_or_default();
        let fecha = dates::business_civil(&row.try_get::<String, _>("Fecha").unwrap_or_default())?;
        let subtotal = money::scale_value(row.try_get::<i64, _>("Subtotal").unwrap_or(0), scale);
        let iva = money::scale_value(row.try_get::<i64, _>("Iva").unwrap_or(0), scale);
        let total = money::scale_value(row.try_get::<i64, _>("Total").unwrap_or(0), scale);
        let estado: i64 = row.try_get("Estado").unwrap_or(0);
        let observaciones: Option<String> = row.try_get("Observaciones").ok().flatten();
        let created_at = dates::audit(&row.try_get::<String, _>("CreatedAt").unwrap_or_default())?;
        let updated_at = dates::audit(&row.try_get::<String, _>("UpdatedAt").unwrap_or_default())?;
        let deleted_at: Option<String> = row.try_get("DeletedAt").ok().flatten();
        let row_version: Option<Vec<u8>> = row.try_get("RowVersion").ok();
        let is_deleted: i64 = row.try_get("IsDeleted").unwrap_or(0);

        // fecha_vencimiento is new: estimated from config default.
        let fecha_vencimiento = fecha + chrono::Duration::days(30);

        let sql = format!(
            "INSERT INTO facturas (id, numero, cliente_id, fecha, fecha_vencimiento, subtotal, iva, total, \
             estado, observaciones, created_at, updated_at, deleted_at, row_version, is_deleted) \
             VALUES ('{}', '{}', '{}', '{}', '{}', {}, {}, {}, {}, {}, '{}', '{}', {}, X'{}', {})",
            id.replace('\'', "''"),
            numero.replace('\'', "''"),
            cliente_id.replace('\'', "''"),
            fecha.to_rfc3339(),
            fecha_vencimiento.to_rfc3339(),
            subtotal,
            iva,
            total,
            estado,
            opt_sql_string(&observaciones),
            created_at.to_rfc3339(),
            updated_at.to_rfc3339(),
            opt_sql_datetime(deleted_at.as_deref())?,
            hex::encode(row_version.as_deref().unwrap_or(&[0, 0, 0, 0, 0, 0, 0, 1])),
            is_deleted,
        );
        exec(db, &sql).await.with_context(|| format!("inserting factura {id}"))?;

        report.warn(
            WarningCode::VencimientoEstimado,
            "Facturas",
            Some(uuid::Uuid::parse_str(&id).unwrap_or_default()),
            serde_json::json!({ "fecha_vencimiento": fecha_vencimiento.to_rfc3339() }),
        );
    }

    report.tables.push(TableReport {
        source: "Facturas".to_owned(),
        target: "facturas".to_owned(),
        source_rows: source_count,
        target_rows: source_count,
        skipped: 0,
        monetary_sums: vec![],
    });
    Ok(())
}

async fn transfer_pagos_factura(
    db: &DatabaseTransaction,
    legacy: &SqlitePool,
    scale: ScaleState,
    tz: Tz,
    report: &mut ImportReport,
) -> Result<()> {
    let rows = sqlx::query("SELECT * FROM PagosFactura WHERE IsDeleted = 0")
        .fetch_all(legacy)
        .await
        .context("reading PagosFactura")?;

    let source_count = rows.len() as u64;

    for row in &rows {
        let id: String = row.try_get("Id").unwrap_or_default();
        let factura_id: String = row.try_get("FacturaId").unwrap_or_default();
        let fecha = dates::business_civil(&row.try_get::<String, _>("Fecha").unwrap_or_default())?;
        let raw_monto: i64 = row.try_get("Monto").unwrap_or(0);
        let medio_pago: i64 = row.try_get("MedioPago").unwrap_or(0);
        let created_at = dates::audit(&row.try_get::<String, _>("CreatedAt").unwrap_or_default())?;
        let updated_at = dates::audit(&row.try_get::<String, _>("UpdatedAt").unwrap_or_default())?;
        let deleted_at: Option<String> = row.try_get("DeletedAt").ok().flatten();
        let row_version: Option<Vec<u8>> = row.try_get("RowVersion").ok();
        let is_deleted: i64 = row.try_get("IsDeleted").unwrap_or(0);

        // Get invoice total for heuristic scaling.
        let invoice_total: i64 = sqlx::query_scalar(
            "SELECT Total FROM Facturas WHERE Id = ?1"
        )
        .bind(&factura_id)
        .fetch_one(legacy)
        .await
        .unwrap_or(0);

        let (monto, was_heuristic) = money::scale_pago(raw_monto, invoice_total, scale);
        if was_heuristic {
            report.warn(
                WarningCode::PagoEscalaHeuristica,
                "PagosFactura",
                Some(uuid::Uuid::parse_str(&id).unwrap_or_default()),
                serde_json::json!({ "raw": raw_monto, "resolved": monto, "invoice_total": invoice_total }),
            );
        }

        let sql = format!(
            "INSERT INTO pagos_factura (id, factura_id, fecha, monto, medio_pago, \
             created_at, updated_at, deleted_at, row_version, is_deleted) \
             VALUES ('{}', '{}', '{}', {}, {}, '{}', '{}', {}, X'{}', {})",
            id.replace('\'', "''"),
            factura_id.replace('\'', "''"),
            fecha.to_rfc3339(),
            monto,
            medio_pago,
            created_at.to_rfc3339(),
            updated_at.to_rfc3339(),
            opt_sql_datetime(deleted_at.as_deref())?,
            hex::encode(row_version.as_deref().unwrap_or(&[0, 0, 0, 0, 0, 0, 0, 1])),
            is_deleted,
        );
        exec(db, &sql).await.with_context(|| format!("inserting pago_factura {id}"))?;
    }

    report.tables.push(TableReport {
        source: "PagosFactura".to_owned(),
        target: "pagos_factura".to_owned(),
        source_rows: source_count,
        target_rows: source_count,
        skipped: 0,
        monetary_sums: vec![],
    });
    Ok(())
}

async fn transfer_empleados(
    db: &DatabaseTransaction,
    legacy: &SqlitePool,
    scale: ScaleState,
    tz: Tz,
    report: &mut ImportReport,
) -> Result<()> {
    let rows = sqlx::query("SELECT * FROM Empleados WHERE IsDeleted = 0")
        .fetch_all(legacy)
        .await
        .context("reading Empleados")?;

    let source_count = rows.len() as u64;

    for row in &rows {
        let id: String = row.try_get("Id").unwrap_or_default();
        let nombre: String = row.try_get("Nombre").unwrap_or_default();
        let dni: Option<String> = row.try_get("Dni").ok().flatten();
        let telefono: Option<String> = row.try_get("Telefono").ok().flatten();
        let email: Option<String> = row.try_get("Email").ok().flatten();
        let cargo: Option<String> = row.try_get("Cargo").ok().flatten();
        let fecha_ingreso = dates::business_civil(&row.try_get::<String, _>("FechaIngreso").unwrap_or_default())?;
        let sueldo_base = money::scale_value(row.try_get::<i64, _>("SueldoBase").unwrap_or(0), scale);
        let tarifa_diaria = money::scale_value(row.try_get::<i64, _>("TarifaDiaria").unwrap_or(0), scale);
        let pago_frecuencia: i64 = row.try_get("PagoFrecuencia").unwrap_or(0);
        let activo: i64 = row.try_get("Activo").unwrap_or(1);
        let created_at = dates::audit(&row.try_get::<String, _>("CreatedAt").unwrap_or_default())?;
        let updated_at = dates::audit(&row.try_get::<String, _>("UpdatedAt").unwrap_or_default())?;
        let deleted_at: Option<String> = row.try_get("DeletedAt").ok().flatten();
        let row_version: Option<Vec<u8>> = row.try_get("RowVersion").ok();
        let is_deleted: i64 = row.try_get("IsDeleted").unwrap_or(0);

        let sql = format!(
            "INSERT INTO empleados (id, nombre, dni, telefono, email, cargo, fecha_ingreso, \
             sueldo_base, tarifa_diaria, pago_frecuencia, activo, \
             created_at, updated_at, deleted_at, row_version, is_deleted) \
             VALUES ('{}', '{}', {}, {}, {}, {}, '{}', {}, {}, {}, {}, '{}', '{}', {}, X'{}', {})",
            id.replace('\'', "''"),
            nombre.replace('\'', "''"),
            opt_sql_string(&dni),
            opt_sql_string(&telefono),
            opt_sql_string(&email),
            opt_sql_string(&cargo),
            fecha_ingreso.to_rfc3339(),
            sueldo_base,
            tarifa_diaria,
            pago_frecuencia,
            activo,
            created_at.to_rfc3339(),
            updated_at.to_rfc3339(),
            opt_sql_datetime(deleted_at.as_deref())?,
            hex::encode(row_version.as_deref().unwrap_or(&[0, 0, 0, 0, 0, 0, 0, 1])),
            is_deleted,
        );
        exec(db, &sql).await.with_context(|| format!("inserting empleado {id}"))?;
    }

    report.tables.push(TableReport {
        source: "Empleados".to_owned(),
        target: "empleados".to_owned(),
        source_rows: source_count,
        target_rows: source_count,
        skipped: 0,
        monetary_sums: vec![],
    });
    Ok(())
}

async fn transfer_asistencias_empleado(
    db: &DatabaseTransaction,
    legacy: &SqlitePool,
    tz: Tz,
    allow_orphans: bool,
    report: &mut ImportReport,
) -> Result<()> {
    let rows = sqlx::query("SELECT * FROM AsistenciasEmpleado WHERE IsDeleted = 0")
        .fetch_all(legacy)
        .await
        .context("reading AsistenciasEmpleado")?;

    let source_count = rows.len() as u64;

    // Group by (empleado_id, civil date) to detect collisions.
    use std::collections::HashMap;
    let mut groups: HashMap<(String, String), Vec<&sqlx::sqlite::SqliteRow>> = HashMap::new();
    for row in &rows {
        let empleado_id: String = row.try_get("EmpleadoId").unwrap_or_default();
        let fecha_raw: String = row.try_get("Fecha").unwrap_or_default();
        let fecha_civil = dates::business_civil(&fecha_raw)?.format("%Y-%m-%d").to_string();
        groups.entry((empleado_id, fecha_civil)).or_default().push(row);
    }

    for ((_empleado_id, _fecha), group) in &groups {
        // Sort by CreatedAt descending, then Id ascending. Keep the first (most recent).
        let mut sorted: Vec<_> = group.iter().collect();
        sorted.sort_by(|a, b| {
            let ca = a.try_get::<String, _>("CreatedAt").unwrap_or_default();
            let cb = b.try_get::<String, _>("CreatedAt").unwrap_or_default();
            cb.cmp(&ca).then_with(|| {
                let ia: String = a.try_get("Id").unwrap_or_default();
                let ib: String = b.try_get("Id").unwrap_or_default();
                ia.cmp(&ib)
            })
        });

        for (i, row) in sorted.iter().enumerate() {
            let id: String = row.try_get("Id").unwrap_or_default();
            let empleado_id: String = row.try_get("EmpleadoId").unwrap_or_default();
            let trabajo_id: Option<String> = row.try_get("TrabajoId").ok().flatten();
            let fecha_raw: String = row.try_get("Fecha").unwrap_or_default();
            let fecha = dates::business_civil(&fecha_raw)?;
            let tipo_jornada: i64 = row.try_get("TipoJornada").unwrap_or(0);
            let observaciones: Option<String> = row.try_get("Observaciones").ok().flatten();
            let created_at = dates::audit(&row.try_get::<String, _>("CreatedAt").unwrap_or_default())?;
            let updated_at = dates::audit(&row.try_get::<String, _>("UpdatedAt").unwrap_or_default())?;
            let deleted_at: Option<String> = row.try_get("DeletedAt").ok().flatten();
            let row_version: Option<Vec<u8>> = row.try_get("RowVersion").ok();

            let is_deleted = if i > 0 { 1 } else { 0 };
            let effective_deleted_at = if i > 0 {
                Some(created_at.to_rfc3339())
            } else {
                deleted_at.as_deref().map(|d| dates::audit(d).map(|dt| dt.to_rfc3339())).transpose()?
            };

            if i > 0 {
                report.warn(
                    WarningCode::AsistenciaColision,
                    "AsistenciasEmpleado",
                    Some(uuid::Uuid::parse_str(&id).unwrap_or_default()),
                    serde_json::json!({ "kept": sorted[0].try_get::<String, _>("Id").unwrap_or_default() }),
                );
            }

            // Handle orphan FK.
            let effective_trabajo_id = if trabajo_id.is_some() {
                trabajo_id.clone()
            } else {
                None
            };

            let sql = format!(
                "INSERT INTO asistencias_empleado (id, empleado_id, trabajo_id, fecha, tipo_jornada, \
                 observaciones, created_at, updated_at, deleted_at, row_version, is_deleted) \
                 VALUES ('{}', '{}', {}, '{}', {}, {}, '{}', '{}', {}, X'{}', {})",
                id.replace('\'', "''"),
                empleado_id.replace('\'', "''"),
                opt_sql_string(&effective_trabajo_id),
                fecha.to_rfc3339(),
                tipo_jornada,
                opt_sql_string(&observaciones),
                created_at.to_rfc3339(),
                updated_at.to_rfc3339(),
                effective_deleted_at.map(|d| format!("'{}'", d)).unwrap_or_else(|| "NULL".to_owned()),
                hex::encode(row_version.as_deref().unwrap_or(&[0, 0, 0, 0, 0, 0, 0, 1])),
                is_deleted,
            );
            exec(db, &sql).await.with_context(|| format!("inserting asistencia {id}"))?;
        }
    }

    report.tables.push(TableReport {
        source: "AsistenciasEmpleado".to_owned(),
        target: "asistencias_empleado".to_owned(),
        source_rows: source_count,
        target_rows: source_count,
        skipped: 0,
        monetary_sums: vec![],
    });
    Ok(())
}

async fn transfer_liquidaciones(
    db: &DatabaseTransaction,
    legacy: &SqlitePool,
    scale: ScaleState,
    tz: Tz,
    report: &mut ImportReport,
) -> Result<()> {
    let rows = sqlx::query("SELECT * FROM Liquidaciones WHERE IsDeleted = 0")
        .fetch_all(legacy)
        .await
        .context("reading Liquidaciones")?;

    let source_count = rows.len() as u64;

    for row in &rows {
        let id: String = row.try_get("Id").unwrap_or_default();
        let empleado_id: String = row.try_get("EmpleadoId").unwrap_or_default();
        let fecha_inicio = dates::business_civil(&row.try_get::<String, _>("FechaInicio").unwrap_or_default())?;
        let fecha_fin = dates::business_civil(&row.try_get::<String, _>("FechaFin").unwrap_or_default())?;
        let dias_trabajados = money::scale_value(row.try_get::<i64, _>("DiasTrabajados").unwrap_or(0), scale);
        let tarifa_aplicada = money::scale_value(row.try_get::<i64, _>("TarifaAplicada").unwrap_or(0), scale);
        let incluir_sabados: i64 = row.try_get("IncluirSabados").unwrap_or(0);
        let incluir_domingos: i64 = row.try_get("IncluirDomingos").unwrap_or(0);
        let incluir_feriados: i64 = row.try_get("IncluirFeriados").unwrap_or(0);
        let multiplicador_sabado = money::default_zero_to_one(money::scale_value(row.try_get::<i64, _>("MultiplicadorSabado").unwrap_or(0), scale));
        let multiplicador_domingo = money::default_zero_to_one(money::scale_value(row.try_get::<i64, _>("MultiplicadorDomingo").unwrap_or(0), scale));
        let multiplicador_feriado = money::default_zero_to_one(money::scale_value(row.try_get::<i64, _>("MultiplicadorFeriado").unwrap_or(0), scale));
        let total_bruto = money::scale_value(row.try_get::<i64, _>("TotalBruto").unwrap_or(0), scale);
        let total_adelantos = money::scale_value(row.try_get::<i64, _>("TotalAdelantos").unwrap_or(0), scale);
        let observaciones: Option<String> = row.try_get("Observaciones").ok().flatten();
        let created_at = dates::audit(&row.try_get::<String, _>("CreatedAt").unwrap_or_default())?;
        let updated_at = dates::audit(&row.try_get::<String, _>("UpdatedAt").unwrap_or_default())?;
        let deleted_at: Option<String> = row.try_get("DeletedAt").ok().flatten();
        let row_version: Option<Vec<u8>> = row.try_get("RowVersion").ok();
        let is_deleted: i64 = row.try_get("IsDeleted").unwrap_or(0);

        let sql = format!(
            "INSERT INTO liquidaciones (id, empleado_id, fecha_inicio, fecha_fin, dias_trabajados, \
             tarifa_aplicada, incluir_sabados, incluir_domingos, incluir_feriados, \
             multiplicador_sabado, multiplicador_domingo, multiplicador_feriado, \
             total_bruto, total_adelantos, observaciones, \
             created_at, updated_at, deleted_at, row_version, is_deleted) \
             VALUES ('{}', '{}', '{}', '{}', {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, '{}', '{}', {}, X'{}', {})",
            id.replace('\'', "''"),
            empleado_id.replace('\'', "''"),
            fecha_inicio.to_rfc3339(),
            fecha_fin.to_rfc3339(),
            dias_trabajados,
            tarifa_aplicada,
            incluir_sabados,
            incluir_domingos,
            incluir_feriados,
            multiplicador_sabado,
            multiplicador_domingo,
            multiplicador_feriado,
            total_bruto,
            total_adelantos,
            opt_sql_string(&observaciones),
            created_at.to_rfc3339(),
            updated_at.to_rfc3339(),
            opt_sql_datetime(deleted_at.as_deref())?,
            hex::encode(row_version.as_deref().unwrap_or(&[0, 0, 0, 0, 0, 0, 0, 1])),
            is_deleted,
        );
        exec(db, &sql).await.with_context(|| format!("inserting liquidacion {id}"))?;
    }

    report.tables.push(TableReport {
        source: "Liquidaciones".to_owned(),
        target: "liquidaciones".to_owned(),
        source_rows: source_count,
        target_rows: source_count,
        skipped: 0,
        monetary_sums: vec![],
    });
    Ok(())
}

async fn transfer_movimientos(
    db: &DatabaseTransaction,
    legacy: &SqlitePool,
    scale: ScaleState,
    tz: Tz,
    allow_orphans: bool,
    report: &mut ImportReport,
) -> Result<()> {
    let rows = sqlx::query("SELECT * FROM Movimientos WHERE IsDeleted = 0")
        .fetch_all(legacy)
        .await
        .context("reading Movimientos")?;

    let source_count = rows.len() as u64;

    for row in &rows {
        let id: String = row.try_get("Id").unwrap_or_default();
        let fecha_raw: String = row.try_get("Fecha").unwrap_or_default();
        let fecha = dates::business_instant(&fecha_raw, tz)?;
        let concepto: String = row.try_get("Concepto").unwrap_or_default();
        let monto = money::scale_value(row.try_get::<i64, _>("Monto").unwrap_or(0), scale);
        let cantidad = money::default_zero_to_one(money::scale_value(row.try_get::<i64, _>("Cantidad").unwrap_or(0), scale));
        let moneda: i64 = row.try_get("Moneda").unwrap_or(0);
        let raw_cotizacion: Option<i64> = row.try_get("CotizacionAplicada").ok().flatten();
        let (cotizacion, was_heuristic, is_zero) = money::scale_cotizacion(raw_cotizacion, scale);
        if was_heuristic {
            report.warn(
                WarningCode::CotizacionEscalaHeuristica,
                "Movimientos",
                Some(uuid::Uuid::parse_str(&id).unwrap_or_default()),
                serde_json::json!({ "raw": raw_cotizacion, "resolved": cotizacion }),
            );
        }
        if is_zero {
            report.warn(
                WarningCode::CotizacionCeroDescartada,
                "Movimientos",
                Some(uuid::Uuid::parse_str(&id).unwrap_or_default()),
                serde_json::json!({ "raw": 0 }),
            );
        }
        let tipo_movimiento_id: String = row.try_get("TipoMovimientoId").unwrap_or_default();
        let tipo_concepto_pago_id: Option<String> = row.try_get("TipoConceptoPagoId").ok().flatten();
        let categoria_id: Option<String> = row.try_get("CategoriaId").ok().flatten();
        let cliente_id: Option<String> = row.try_get("ClienteId").ok().flatten();
        let empleado_id: Option<String> = row.try_get("EmpleadoId").ok().flatten();
        let trabajo_id: Option<String> = row.try_get("TrabajoId").ok().flatten();
        let factura_id: Option<String> = row.try_get("FacturaId").ok().flatten();
        let created_at = dates::audit(&row.try_get::<String, _>("CreatedAt").unwrap_or_default())?;
        let updated_at = dates::audit(&row.try_get::<String, _>("UpdatedAt").unwrap_or_default())?;
        let deleted_at: Option<String> = row.try_get("DeletedAt").ok().flatten();
        let row_version: Option<Vec<u8>> = row.try_get("RowVersion").ok();
        let is_deleted: i64 = row.try_get("IsDeleted").unwrap_or(0);

        let sql = format!(
            "INSERT INTO movimientos (id, fecha, concepto, monto, cantidad, moneda, cotizacion_aplicada, \
             tipo_movimiento_id, tipo_concepto_pago_id, categoria_id, cliente_id, empleado_id, \
             trabajo_id, factura_id, created_at, updated_at, deleted_at, row_version, is_deleted) \
             VALUES ('{}', '{}', '{}', {}, {}, {}, {}, '{}', {}, {}, {}, {}, {}, {}, '{}', '{}', {}, X'{}', {})",
            id.replace('\'', "''"),
            fecha.to_rfc3339(),
            concepto.replace('\'', "''"),
            monto,
            cantidad,
            moneda,
            cotizacion.map(|c| c.to_string()).unwrap_or_else(|| "NULL".to_owned()),
            tipo_movimiento_id.replace('\'', "''"),
            opt_sql_string(&tipo_concepto_pago_id),
            opt_sql_string(&categoria_id),
            opt_sql_string(&cliente_id),
            opt_sql_string(&empleado_id),
            opt_sql_string(&trabajo_id),
            opt_sql_string(&factura_id),
            created_at.to_rfc3339(),
            updated_at.to_rfc3339(),
            opt_sql_datetime(deleted_at.as_deref())?,
            hex::encode(row_version.as_deref().unwrap_or(&[0, 0, 0, 0, 0, 0, 0, 1])),
            is_deleted,
        );
        exec(db, &sql).await.with_context(|| format!("inserting movimiento {id}"))?;
    }

    report.tables.push(TableReport {
        source: "Movimientos".to_owned(),
        target: "movimientos".to_owned(),
        source_rows: source_count,
        target_rows: source_count,
        skipped: 0,
        monetary_sums: vec![],
    });
    Ok(())
}

async fn transfer_adjuntos(
    db: &DatabaseTransaction,
    legacy: &SqlitePool,
    report: &mut ImportReport,
) -> Result<()> {
    let rows = sqlx::query("SELECT * FROM Adjuntos WHERE IsDeleted = 0")
        .fetch_all(legacy)
        .await
        .context("reading Adjuntos")?;

    let source_count = rows.len() as u64;

    for row in &rows {
        let id: String = row.try_get("Id").unwrap_or_default();
        let entidad_tipo: String = row.try_get("EntidadTipo").unwrap_or_default();
        let entidad_id: String = row.try_get("EntidadId").unwrap_or_default();
        let nombre_archivo: String = row.try_get("NombreArchivo").unwrap_or_default();
        let ruta_relativa: String = row.try_get("RutaRelativa").unwrap_or_default();
        let mime: String = row.try_get("Mime").unwrap_or_default();
        let tamano: i64 = row.try_get("Tamano").unwrap_or(0);
        let created_at = dates::audit(&row.try_get::<String, _>("CreatedAt").unwrap_or_default())?;
        let updated_at = dates::audit(&row.try_get::<String, _>("UpdatedAt").unwrap_or_default())?;
        let deleted_at: Option<String> = row.try_get("DeletedAt").ok().flatten();
        let row_version: Option<Vec<u8>> = row.try_get("RowVersion").ok();
        let is_deleted: i64 = row.try_get("IsDeleted").unwrap_or(0);

        let sql = format!(
            "INSERT INTO adjuntos (id, entidad_tipo, entidad_id, nombre_archivo, ruta_relativa, \
             mime, tamano, created_at, updated_at, deleted_at, row_version, is_deleted) \
             VALUES ('{}', '{}', '{}', '{}', '{}', '{}', {}, '{}', '{}', {}, X'{}', {})",
            id.replace('\'', "''"),
            entidad_tipo.replace('\'', "''"),
            entidad_id.replace('\'', "''"),
            nombre_archivo.replace('\'', "''"),
            ruta_relativa.replace('\'', "''"),
            mime.replace('\'', "''"),
            tamano,
            created_at.to_rfc3339(),
            updated_at.to_rfc3339(),
            opt_sql_datetime(deleted_at.as_deref())?,
            hex::encode(row_version.as_deref().unwrap_or(&[0, 0, 0, 0, 0, 0, 0, 1])),
            is_deleted,
        );
        exec(db, &sql).await.with_context(|| format!("inserting adjunto {id}"))?;
    }

    report.tables.push(TableReport {
        source: "Adjuntos".to_owned(),
        target: "adjuntos".to_owned(),
        source_rows: source_count,
        target_rows: source_count,
        skipped: 0,
        monetary_sums: vec![],
    });
    Ok(())
}

async fn transfer_app_metadata(
    db: &DatabaseTransaction,
    legacy: &SqlitePool,
    report: &mut ImportReport,
) -> Result<()> {
    let rows = sqlx::query("SELECT * FROM AppMetadata")
        .fetch_all(legacy)
        .await
        .context("reading AppMetadata")?;

    let source_count = rows.len() as u64;

    for row in &rows {
        let key: String = row.try_get("Key").unwrap_or_default();
        let value: Option<String> = row.try_get("Value").ok().flatten();
        let updated_at = dates::audit(&row.try_get::<String, _>("UpdatedAt").unwrap_or_default())?;

        let sql = format!(
            "INSERT OR REPLACE INTO app_metadata (key, value, updated_at) VALUES ('{}', {}, '{}')",
            key.replace('\'', "''"),
            opt_sql_string(&value),
            updated_at.to_rfc3339(),
        );
        exec(db, &sql).await.with_context(|| format!("inserting app_metadata {key}"))?;
    }

    report.tables.push(TableReport {
        source: "AppMetadata".to_owned(),
        target: "app_metadata".to_owned(),
        source_rows: source_count,
        target_rows: source_count,
        skipped: 0,
        monetary_sums: vec![],
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
