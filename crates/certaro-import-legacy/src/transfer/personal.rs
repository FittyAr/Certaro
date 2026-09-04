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
pub(crate) async fn transfer_empleados(
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
        let fecha_ingreso =
            dates::business_civil(&row.try_get::<String, _>("FechaIngreso").unwrap_or_default())?;
        let sueldo_base =
            money::scale_value(row.try_get::<i64, _>("SueldoBase").unwrap_or(0), scale);
        let tarifa_diaria =
            money::scale_value(row.try_get::<i64, _>("TarifaDiaria").unwrap_or(0), scale);
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
        exec(db, &sql)
            .await
            .with_context(|| format!("inserting empleado {id}"))?;
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

#[allow(unused_variables)]
pub(crate) async fn transfer_asistencias_empleado(
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
        let fecha_civil = dates::business_civil(&fecha_raw)?
            .format("%Y-%m-%d")
            .to_string();
        groups
            .entry((empleado_id, fecha_civil))
            .or_default()
            .push(row);
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
            let created_at =
                dates::audit(&row.try_get::<String, _>("CreatedAt").unwrap_or_default())?;
            let updated_at =
                dates::audit(&row.try_get::<String, _>("UpdatedAt").unwrap_or_default())?;
            let deleted_at: Option<String> = row.try_get("DeletedAt").ok().flatten();
            let row_version: Option<Vec<u8>> = row.try_get("RowVersion").ok();

            let is_deleted = if i > 0 { 1 } else { 0 };
            let effective_deleted_at = if i > 0 {
                Some(created_at.to_rfc3339())
            } else {
                deleted_at
                    .as_deref()
                    .map(|d| dates::audit(d).map(|dt| dt.to_rfc3339()))
                    .transpose()?
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
            exec(db, &sql)
                .await
                .with_context(|| format!("inserting asistencia {id}"))?;
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

#[allow(unused_variables)]
pub(crate) async fn transfer_liquidaciones(
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
        let fecha_inicio =
            dates::business_civil(&row.try_get::<String, _>("FechaInicio").unwrap_or_default())?;
        let fecha_fin =
            dates::business_civil(&row.try_get::<String, _>("FechaFin").unwrap_or_default())?;
        let dias_trabajados =
            money::scale_value(row.try_get::<i64, _>("DiasTrabajados").unwrap_or(0), scale);
        let tarifa_aplicada =
            money::scale_value(row.try_get::<i64, _>("TarifaAplicada").unwrap_or(0), scale);
        let incluir_sabados: i64 = row.try_get("IncluirSabados").unwrap_or(0);
        let incluir_domingos: i64 = row.try_get("IncluirDomingos").unwrap_or(0);
        let incluir_feriados: i64 = row.try_get("IncluirFeriados").unwrap_or(0);
        let multiplicador_sabado = money::default_zero_to_one(money::scale_value(
            row.try_get::<i64, _>("MultiplicadorSabado").unwrap_or(0),
            scale,
        ));
        let multiplicador_domingo = money::default_zero_to_one(money::scale_value(
            row.try_get::<i64, _>("MultiplicadorDomingo").unwrap_or(0),
            scale,
        ));
        let multiplicador_feriado = money::default_zero_to_one(money::scale_value(
            row.try_get::<i64, _>("MultiplicadorFeriado").unwrap_or(0),
            scale,
        ));
        let total_bruto =
            money::scale_value(row.try_get::<i64, _>("TotalBruto").unwrap_or(0), scale);
        let total_adelantos =
            money::scale_value(row.try_get::<i64, _>("TotalAdelantos").unwrap_or(0), scale);
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
        exec(db, &sql)
            .await
            .with_context(|| format!("inserting liquidacion {id}"))?;
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
