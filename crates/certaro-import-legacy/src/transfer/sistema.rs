use anyhow::{Context, Result};
use chrono_tz::Tz;
use sea_orm::DatabaseTransaction;
use sqlx::sqlite::SqlitePool;
use sqlx::Row;

use crate::dates;
use crate::money;
use crate::report::{ImportReport, ScaleState, TableReport, WarningCode};
use crate::text;
use super::{exec, opt_sql_datetime, opt_sql_string, SYSTEM_TIPO_IDS};

#[allow(unused_variables)]
pub(crate) async fn transfer_movimientos(
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
        let cantidad = money::default_zero_to_one(money::scale_value(
            row.try_get::<i64, _>("Cantidad").unwrap_or(0),
            scale,
        ));
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
        let tipo_concepto_pago_id: Option<String> =
            row.try_get("TipoConceptoPagoId").ok().flatten();
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
        exec(db, &sql)
            .await
            .with_context(|| format!("inserting movimiento {id}"))?;
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

pub(crate) async fn transfer_adjuntos(
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
        exec(db, &sql)
            .await
            .with_context(|| format!("inserting adjunto {id}"))?;
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

pub(crate) async fn transfer_app_metadata(
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
        exec(db, &sql)
            .await
            .with_context(|| format!("inserting app_metadata {key}"))?;
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

