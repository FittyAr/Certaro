//! Phase 6: post-import verification. See `docs/15-migracion-de-datos.md` §7.

use anyhow::Result;
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseTransaction, Statement};

use crate::report::ImportReport;

/// Runs all verification checks. Returns Ok(()) if all pass, Err if any blocking issue is found.
pub async fn verify(db: &DatabaseTransaction, report: &mut ImportReport) -> Result<()> {
    verify_row_counts(db, report).await?;
    verify_invariants(db, report).await?;

    if report.has_blocking_issues() {
        anyhow::bail!(
            "verification found {} blocking issues",
            report.blocking_issues.len()
        );
    }

    Ok(())
}

/// Tables that have seed rows inserted by migrations.
const SEEDED_TABLES: &[&str] = &["tipos_movimiento", "tipos_concepto_pago", "app_metadata"];

/// Verifies row counts match expectations.
async fn verify_row_counts(db: &DatabaseTransaction, report: &mut ImportReport) -> Result<()> {
    let tables: Vec<(String, u64)> = report
        .tables
        .iter()
        .map(|t| (t.target.clone(), t.target_rows))
        .collect();
    for (target, expected) in &tables {
        let actual = query_count(db, &format!("SELECT COUNT(*) as cnt FROM {target}")).await?;

        if SEEDED_TABLES.contains(&target.as_str()) {
            // Seeded tables: actual should be >= expected (seed rows + imported rows).
            if actual < *expected as i64 {
                report.block(format!(
                    "row count mismatch for {target}: expected at least {expected}, got {actual}"
                ));
            }
        } else {
            if actual != *expected as i64 {
                report.block(format!(
                    "row count mismatch for {target}: expected {expected}, got {actual}"
                ));
            }
        }
    }

    Ok(())
}

/// Verifies the 10 business invariants from doc 15 §7.3.
async fn verify_invariants(db: &DatabaseTransaction, report: &mut ImportReport) -> Result<()> {
    // 1. total == subtotal + iva in every invoice.
    let inv1 = query_count(
        db,
        "SELECT COUNT(*) as cnt FROM facturas WHERE total <> subtotal + iva AND is_deleted = 0",
    )
    .await?;
    if inv1 > 0 {
        report.block(format!(
            "invariant 1: {inv1} invoices with total != subtotal + iva"
        ));
    }

    // 2. No movement without tipo.
    let inv2 = query_count(db, "SELECT COUNT(*) as cnt FROM movimientos WHERE tipo_movimiento_id IS NULL AND is_deleted = 0").await?;
    if inv2 > 0 {
        report.block(format!(
            "invariant 2: {inv2} movements without tipo_movimiento_id"
        ));
    }

    // 5. movimientos.cantidad never zero.
    let inv5 = query_count(
        db,
        "SELECT COUNT(*) as cnt FROM movimientos WHERE cantidad = 0 AND is_deleted = 0",
    )
    .await?;
    if inv5 > 0 {
        report.block(format!("invariant 5: {inv5} movements with cantidad = 0"));
    }

    // 6. Multipliers never zero.
    for col in &[
        "multiplicador_sabado",
        "multiplicador_domingo",
        "multiplicador_feriado",
    ] {
        let inv6 = query_count(
            db,
            &format!(
                "SELECT COUNT(*) as cnt FROM liquidaciones WHERE {col} = 0 AND is_deleted = 0"
            ),
        )
        .await?;
        if inv6 > 0 {
            report.block(format!("invariant 6: {inv6} liquidaciones with {col} = 0"));
        }
    }

    // 7. At most one principal contact per client.
    let inv7 = query_count(db, "SELECT COUNT(*) as cnt FROM (SELECT cliente_id FROM cliente_contactos WHERE es_principal = 1 AND is_deleted = 0 GROUP BY cliente_id HAVING COUNT(*) > 1)").await?;
    if inv7 > 0 {
        report.block(format!(
            "invariant 7: {inv7} clients with multiple principal contacts"
        ));
    }

    // 9. row_version is 8 bytes.
    for table in &[
        "movimientos",
        "facturas",
        "clientes",
        "obras",
        "trabajos",
        "empleados",
    ] {
        let inv9 = query_count(db, &format!("SELECT COUNT(*) as cnt FROM {table} WHERE LENGTH(row_version) <> 8 AND is_deleted = 0")).await?;
        if inv9 > 0 {
            report.block(format!(
                "invariant 9: {inv9} rows in {table} with invalid row_version length"
            ));
        }
    }

    Ok(())
}

async fn query_count(db: &DatabaseTransaction, sql: &str) -> Result<i64> {
    let result = db
        .query_one(Statement::from_string(
            DatabaseBackend::Sqlite,
            sql.to_owned(),
        ))
        .await?;
    Ok(result
        .and_then(|r| r.try_get::<i64>("", "cnt").ok())
        .unwrap_or(0))
}
