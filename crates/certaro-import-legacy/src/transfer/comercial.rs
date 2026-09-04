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

pub(crate) async fn transfer_clientes(
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

pub(crate) async fn transfer_cliente_contactos(
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

#[allow(unused_variables)]
pub(crate) async fn transfer_facturas(
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
        exec(db, &sql)
            .await
            .with_context(|| format!("inserting factura {id}"))?;

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

#[allow(unused_variables)]
pub(crate) async fn transfer_pagos_factura(
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
        let invoice_total: i64 = sqlx::query_scalar("SELECT Total FROM Facturas WHERE Id = ?1")
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
        exec(db, &sql)
            .await
            .with_context(|| format!("inserting pago_factura {id}"))?;
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

