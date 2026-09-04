use anyhow::{Context, Result};
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseTransaction, Statement};
use crate::report::ImportReport;

/// Derives one certificate per work order that has items with porcentaje_actual > 0.
/// Returns (certificados, certificado_items).
#[allow(unused_variables)]
pub async fn derive_certificados(
    db: &DatabaseTransaction,
    report: &mut ImportReport,
) -> Result<(u64, u64)> {
    // Find work orders with at least one item with porcentaje_actual > 0.
    let ordenes = db
        .query_all(Statement::from_string(
            DatabaseBackend::Sqlite,
            "SELECT DISTINCT o.id, o.fecha, o.ajuste_uocra_porcentaje, o.otros_descuentos              FROM ordenes_trabajo o              JOIN orden_trabajo_items i ON i.orden_trabajo_id = o.id              WHERE i.porcentaje_actual > 0 AND o.is_deleted = 0 AND i.is_deleted = 0"
                .to_owned(),
        ))
        .await
        .context("finding work orders for certificate derivation")?;

    let mut cert_count = 0u64;
    let mut item_count = 0u64;

    for orden in ordenes {
        let orden_id: String = orden.try_get("", "id").unwrap_or_default();
        let fecha: String = orden.try_get("", "fecha").unwrap_or_default();
        let ajuste_uocra: i64 = orden.try_get("", "ajuste_uocra_porcentaje").unwrap_or(0);
        let otros_descuentos: i64 = orden.try_get("", "otros_descuentos").unwrap_or(0);

        // Generate a UUID v7 for the certificate.
        let cert_id = uuid::Uuid::now_v7().to_string();

        // Get items for this order.
        let items = db
            .query_all(Statement::from_string(
                DatabaseBackend::Sqlite,
                format!(
                    "SELECT id, cantidad, precio_unitario, porcentaje_anterior, porcentaje_actual                      FROM orden_trabajo_items                      WHERE orden_trabajo_id = '{}' AND is_deleted = 0                      ORDER BY orden",
                    orden_id.replace('\'', "''")
                ),
            ))
            .await
            .context("getting order items for certificate")?;

        // First pass: calculate totals.
        let mut total_certificado = 0i64;
        let mut item_data = Vec::new();
        for item in &items {
            let cantidad: i64 = item.try_get("", "cantidad").unwrap_or(0);
            let precio_unitario: i64 = item.try_get("", "precio_unitario").unwrap_or(0);
            let porcentaje_anterior: i64 = item.try_get("", "porcentaje_anterior").unwrap_or(0);
            let porcentaje_actual: i64 = item.try_get("", "porcentaje_actual").unwrap_or(0);
            // Avoid overflow: divide early. All values are scaled ×10_000.
            // subtotal = cantidad × precio_unitario × porcentaje / 10_000_000
            // = (cantidad / 10_000) × precio_unitario × (porcentaje / 10_000) / 100
            // But simpler: use i128 for the intermediate product.
            let subtotal_actual =
                (cantidad as i128 * precio_unitario as i128 * porcentaje_actual as i128
                    / 10_000_000) as i64;
            let subtotal_acumulado = (cantidad as i128
                * precio_unitario as i128
                * (porcentaje_anterior + porcentaje_actual) as i128
                / 10_000_000) as i64;
            total_certificado += subtotal_actual;
            item_data.push((
                item.try_get::<String>("", "id").unwrap_or_default(),
                cantidad,
                precio_unitario,
                porcentaje_anterior,
                porcentaje_actual,
                subtotal_actual,
                subtotal_acumulado,
            ));
        }

        // Insert the certificate FIRST (FK target for items).
        let ajuste_uocra_monto =
            (total_certificado as i128 * ajuste_uocra as i128 / 10_000_000) as i64;
        let total_neto = total_certificado - ajuste_uocra_monto - otros_descuentos;

        let sql = format!(
            "INSERT INTO certificados (id, orden_trabajo_id, numero, fecha, observaciones,              total_certificado, ajuste_uocra, otros_descuentos, total_neto, created_at)              VALUES ('{}', '{}', 1, '{}', 'Importado del sistema anterior. Certificado único reconstruido.',              {}, {}, {}, {}, '{}')",
            cert_id,
            orden_id,
            fecha,
            total_certificado,
            ajuste_uocra_monto,
            otros_descuentos,
            total_neto,
            chrono::Utc::now().to_rfc3339(),
        );
        db.execute(Statement::from_string(DatabaseBackend::Sqlite, sql))
            .await
            .context("inserting certificado")?;
        cert_count += 1;

        // Second pass: insert items (FK to certificado now exists).
        for (
            item_id_orig,
            cantidad,
            precio_unitario,
            porcentaje_anterior,
            porcentaje_actual,
            subtotal_actual,
            subtotal_acumulado,
        ) in &item_data
        {
            let item_id = uuid::Uuid::now_v7().to_string();
            let sql = format!(
                "INSERT INTO certificado_items (id, certificado_id, orden_trabajo_item_id,                  cantidad, precio_unitario, porcentaje_anterior, porcentaje_actual,                  subtotal_actual, subtotal_acumulado, created_at)                  VALUES ('{}', '{}', '{}', {}, {}, {}, {}, {}, {}, '{}')",
                item_id,
                cert_id,
                item_id_orig,
                cantidad,
                precio_unitario,
                porcentaje_anterior,
                porcentaje_actual,
                subtotal_actual,
                subtotal_acumulado,
                chrono::Utc::now().to_rfc3339(),
            );
            db.execute(Statement::from_string(DatabaseBackend::Sqlite, sql))
                .await
                .context("inserting certificado_item")?;
            item_count += 1;
        }
    }

    Ok((cert_count, item_count))
}
