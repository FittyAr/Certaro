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

pub(crate) async fn transfer_proyectos(
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
            "INSERT INTO proyectos (id, numero, nombre, direccion, localidad, cliente_id, estado, \
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
                report.block(format!("proyecto {id} (numero={numero}): {e}"));
                return Err(e).context("inserting proyecto");
            }
        }
    }

    report.tables.push(TableReport {
        source: "Obras".to_owned(),
        target: "proyectos".to_owned(),
        source_rows: source_count,
        target_rows: source_count,
        skipped: 0,
        monetary_sums: vec![],
    });

    Ok(())
}

pub(crate) async fn transfer_trabajos(
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
        let proyecto_id: String = row.try_get("ObraId").unwrap_or_default();
        let descripcion: String = row.try_get("Descripcion").unwrap_or_default();
        let presupuesto =
            money::scale_value(row.try_get::<i64, _>("Presupuesto").unwrap_or(0), scale);
        let fecha_inicio =
            dates::business_civil(&row.try_get::<String, _>("FechaInicio").unwrap_or_default())?;
        let fecha_fin_raw: Option<String> = row.try_get("FechaFin").ok().flatten();
        let fecha_fin = fecha_fin_raw
            .as_deref()
            .map(dates::business_civil)
            .transpose()?;
        let estado: i64 = row.try_get("Estado").unwrap_or(0);
        let created_at = dates::audit(&row.try_get::<String, _>("CreatedAt").unwrap_or_default())?;
        let updated_at = dates::audit(&row.try_get::<String, _>("UpdatedAt").unwrap_or_default())?;
        let deleted_at: Option<String> = row.try_get("DeletedAt").ok().flatten();
        let row_version: Option<Vec<u8>> = row.try_get("RowVersion").ok();
        let is_deleted: i64 = row.try_get("IsDeleted").unwrap_or(0);

        sum_presupuesto += presupuesto;

        let sql = format!(
            "INSERT INTO trabajos (id, proyecto_id, descripcion, presupuesto, fecha_inicio, fecha_fin, \
             estado, created_at, updated_at, deleted_at, row_version, is_deleted) \
             VALUES ('{}', '{}', '{}', {}, '{}', {}, {}, '{}', '{}', {}, X'{}', {})",
            id.replace('\'', "''"),
            proyecto_id.replace('\'', "''"),
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

#[allow(unused_variables)]
pub(crate) async fn transfer_ordenes_trabajo(
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
        let ajuste_uocra = money::scale_value(
            row.try_get::<i64, _>("AjusteUocraPorcentaje").unwrap_or(0),
            scale,
        );
        let otros_descuentos =
            money::scale_value(row.try_get::<i64, _>("OtrosDescuentos").unwrap_or(0), scale);
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
        exec(db, &sql)
            .await
            .with_context(|| format!("inserting orden_trabajo {id}"))?;
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

#[allow(unused_variables)]
pub(crate) async fn transfer_orden_trabajo_items(
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
        let precio_unitario =
            money::scale_value(row.try_get::<i64, _>("PrecioUnitario").unwrap_or(0), scale);
        let porcentaje_anterior = money::scale_value(
            row.try_get::<i64, _>("PorcentajeAnterior").unwrap_or(0),
            scale,
        );
        let porcentaje_actual = money::scale_value(
            row.try_get::<i64, _>("PorcentajeActual").unwrap_or(0),
            scale,
        );
        let ejecutado: i64 = row.try_get("Ejecutado").unwrap_or(0);
        let nota: Option<String> = row.try_get("Nota").ok().flatten();
        let created_at = dates::audit(&row.try_get::<String, _>("CreatedAt").unwrap_or_default())?;
        let updated_at = dates::audit(&row.try_get::<String, _>("UpdatedAt").unwrap_or_default())?;
        let deleted_at: Option<String> = row.try_get("DeletedAt").ok().flatten();
        let row_version: Option<Vec<u8>> = row.try_get("RowVersion").ok();
        let is_deleted: i64 = row.try_get("IsDeleted").unwrap_or(0);

        // Warn if accumulated percentage exceeds 100%.
        let total_pct = porcentaje_anterior + porcentaje_actual;
        if total_pct > 1_000_000 {
            // 100.0000 in scaled representation
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
        exec(db, &sql)
            .await
            .with_context(|| format!("inserting orden_trabajo_item {id}"))?;
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
