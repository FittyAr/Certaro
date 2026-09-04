use std::collections::HashSet;
use anyhow::{Context, Result};
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseTransaction, Statement};
use crate::report::{ImportReport, WarningCode};

/// Derives liquidacion_adelantos from movements of type Adelanto.
/// Returns the count of derived advance records.
pub async fn derive_liquidacion_adelantos(
    db: &DatabaseTransaction,
    report: &mut ImportReport,
) -> Result<u64> {
    // Get all liquidations ordered by fecha_inicio, then created_at.
    let liquidaciones = db
        .query_all(Statement::from_string(
            DatabaseBackend::Sqlite,
            "SELECT id, empleado_id, fecha_inicio, fecha_fin, total_adelantos              FROM liquidaciones WHERE is_deleted = 0              ORDER BY fecha_inicio, created_at"
                .to_owned(),
        ))
        .await
        .context("getting liquidations for advance derivation")?;

    let mut count = 0u64;
    let mut used_movements: HashSet<String> = HashSet::new();

    for liq in &liquidaciones {
        let liq_id: String = liq.try_get("", "id").unwrap_or_default();
        let empleado_id: String = liq.try_get("", "empleado_id").unwrap_or_default();
        let fecha_inicio: String = liq.try_get("", "fecha_inicio").unwrap_or_default();
        let fecha_fin: String = liq.try_get("", "fecha_fin").unwrap_or_default();
        let total_adelantos: i64 = liq.try_get("", "total_adelantos").unwrap_or(0);

        // Find advance movements for this employee in this period.
        let candidatos = db
            .query_all(Statement::from_string(
                DatabaseBackend::Sqlite,
                format!(
                    "SELECT id, fecha, monto, concepto FROM movimientos                      WHERE tipo_movimiento_id = '00000000-0000-0000-0000-000000000003'                      AND empleado_id = '{}'                      AND fecha >= '{}' AND fecha <= '{}'                      AND is_deleted = 0                      ORDER BY fecha",
                    empleado_id.replace('\'', "''"),
                    fecha_inicio.replace('\'', "''"),
                    fecha_fin.replace('\'', "''"),
                ),
            ))
            .await
            .context("finding advance movements")?;

        let mut suma_derivada = 0i64;

        for candidato in &candidatos {
            let mov_id: String = candidato.try_get("", "id").unwrap_or_default();
            if used_movements.contains(&mov_id) {
                continue;
            }

            let fecha: String = candidato.try_get("", "fecha").unwrap_or_default();
            let monto: i64 = candidato.try_get("", "monto").unwrap_or(0);
            let concepto: String = candidato.try_get("", "concepto").unwrap_or_default();

            let adelanto_id = uuid::Uuid::now_v7().to_string();
            let sql = format!(
                "INSERT INTO liquidacion_adelantos (id, liquidacion_id, movimiento_id,                  monto, fecha, concepto, created_at)                  VALUES ('{}', '{}', '{}', {}, '{}', '{}', '{}')",
                adelanto_id,
                liq_id,
                mov_id,
                monto,
                fecha,
                concepto.replace('\'', "''"),
                chrono::Utc::now().to_rfc3339(),
            );
            db.execute(Statement::from_string(DatabaseBackend::Sqlite, sql))
                .await
                .context("inserting liquidacion_adelanto")?;

            used_movements.insert(mov_id);
            suma_derivada += monto;
            count += 1;
        }

        // Warn if derived sum differs from stored total.
        if suma_derivada != total_adelantos {
            report.warn(
                WarningCode::AdelantoSumaDifiere,
                "Liquidaciones",
                Some(uuid::Uuid::parse_str(&liq_id).unwrap_or_default()),
                serde_json::json!({ "derived": suma_derivada, "stored": total_adelantos }),
            );
        }
    }

    Ok(count)
}
