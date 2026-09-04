//! Phase 4: table-by-table transfer. See `docs/15-migracion-de-datos.md` §4.

use anyhow::Result;
use chrono_tz::Tz;
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseTransaction, Statement};
use sqlx::sqlite::SqlitePool;

use crate::dates;
use crate::report::{ImportReport, ScaleState, TableReport};

pub mod catalogs;
pub mod comercial;
pub mod personal;
pub mod proyectos;
pub mod sistema;

pub(crate) use catalogs::*;
pub(crate) use comercial::*;
pub(crate) use personal::*;
pub(crate) use proyectos::*;
pub(crate) use sistema::*;

/// System tipo_movimiento IDs that already exist in the seed.
pub(crate) const SYSTEM_TIPO_IDS: &[&str] = &[
    "00000000-0000-0000-0000-000000000001",
    "00000000-0000-0000-0000-000000000002",
    "00000000-0000-0000-0000-000000000003",
    "00000000-0000-0000-0000-000000000004",
];

/// Helper: execute a raw SQL statement with no results.
pub(crate) async fn exec(db: &DatabaseTransaction, sql: &str) -> Result<()> {
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        sql.to_owned(),
    ))
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
    transfer_proyectos(db, legacy, report).await?;
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

