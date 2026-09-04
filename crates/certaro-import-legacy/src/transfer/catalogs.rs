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

// ── Implemented tables ──────────────────────────────────────────────────────

pub(crate) async fn transfer_tipos_movimiento(
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

pub(crate) async fn transfer_tipos_concepto_pago(
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

pub(crate) async fn transfer_categorias(
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
