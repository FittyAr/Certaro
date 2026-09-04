use anyhow::{Context, Result};
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseTransaction, Statement};
use crate::report::FacturasReclasificadas;

/// Reclassifies invoice states based on payments.
pub async fn reclassify_facturas(db: &DatabaseTransaction) -> Result<FacturasReclasificadas> {
    let mut result = FacturasReclasificadas::default();

    let facturas = db
        .query_all(Statement::from_string(
            DatabaseBackend::Sqlite,
            "SELECT id, estado, total, fecha_vencimiento FROM facturas              WHERE estado IN (1, 2) AND is_deleted = 0"
                .to_owned(),
        ))
        .await
        .context("getting invoices for reclassification")?;

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    for factura in &facturas {
        let id: String = factura.try_get("", "id").unwrap_or_default();
        let estado: i64 = factura.try_get("", "estado").unwrap_or(0);
        let total: i64 = factura.try_get("", "total").unwrap_or(0);
        let fecha_vencimiento: String =
            factura.try_get("", "fecha_vencimiento").unwrap_or_default();

        let total_pagado: i64 = db
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                format!(
                    "SELECT COALESCE(SUM(monto), 0) as total_pagado FROM pagos_factura                      WHERE factura_id = '{}' AND is_deleted = 0",
                    id.replace('\'', "''")
                ),
            ))
            .await
            .context("summing payments")?
            .map(|r| r.try_get::<i64>("", "total_pagado").unwrap_or(0))
            .unwrap_or(0);

        let new_estado = if total_pagado >= total {
            3 // Pagada
        } else if total_pagado > 0 {
            5 // PagadaParcial
        } else if fecha_vencimiento < today {
            2 // Vencida
        } else {
            continue; // No change.
        };

        if new_estado != estado {
            db.execute(Statement::from_string(
                DatabaseBackend::Sqlite,
                format!(
                    "UPDATE facturas SET estado = {} WHERE id = '{}'",
                    new_estado,
                    id.replace('\'', "''")
                ),
            ))
            .await
            .context("reclassifying invoice")?;

            match new_estado {
                3 => result.pagada += 1,
                5 => result.pagada_parcial += 1,
                2 => result.vencida += 1,
                _ => {}
            }
        }
    }

    Ok(result)
}
