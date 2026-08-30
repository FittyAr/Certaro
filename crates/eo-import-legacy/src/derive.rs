//! Phase 5: derived data. See `docs/15-migracion-de-datos.md` §5.
//!
//! Populates three new tables (certificados, certificado_items, liquidacion_adelantos) and
//! performs additional derivations (contacts from email, holidays from config, invoice state
//! reclassification).

use anyhow::{Context, Result};
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseTransaction, Statement};
use sqlx::sqlite::SqlitePool;

use crate::report::{DerivedReport, FacturasReclasificadas, ImportReport, WarningCode};

/// Runs all derivations.
pub async fn derive_all(
    db: &DatabaseTransaction,
    legacy: &SqlitePool,
    report: &mut ImportReport,
) -> Result<()> {
    let cert_count = derive_certificados(db, report).await?;
    let adelantos_count = derive_liquidacion_adelantos(db, report).await?;
    let contactos_count = derive_contactos(db, report).await?;
    let feriados_count = derive_feriados(db, legacy, report).await?;
    let reclasificadas = reclassify_facturas(db).await?;

    report.derived = DerivedReport {
        certificados: cert_count.0,
        certificado_items: cert_count.1,
        liquidacion_adelantos: adelantos_count,
        contactos_creados: contactos_count,
        feriados_recuperados: feriados_count,
        facturas_reclasificadas: reclasificadas,
        vencimientos_estimados: 0, // Already counted during transfer.
    };

    Ok(())
}

/// Derives one certificate per work order that has items with porcentaje_actual > 0.
/// Returns (certificados, certificado_items).
#[allow(unused_variables)]
async fn derive_certificados(
    db: &DatabaseTransaction,
    report: &mut ImportReport,
) -> Result<(u64, u64)> {
    // Find work orders with at least one item with porcentaje_actual > 0.
    let ordenes = db
        .query_all(Statement::from_string(
            DatabaseBackend::Sqlite,
            "SELECT DISTINCT o.id, o.fecha, o.ajuste_uocra_porcentaje, o.otros_descuentos \
             FROM ordenes_trabajo o \
             JOIN orden_trabajo_items i ON i.orden_trabajo_id = o.id \
             WHERE i.porcentaje_actual > 0 AND o.is_deleted = 0 AND i.is_deleted = 0"
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
                    "SELECT id, cantidad, precio_unitario, porcentaje_anterior, porcentaje_actual \
                     FROM orden_trabajo_items \
                     WHERE orden_trabajo_id = '{}' AND is_deleted = 0 \
                     ORDER BY orden",
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
            let subtotal_actual = (cantidad as i128 * precio_unitario as i128 * porcentaje_actual as i128 / 10_000_000) as i64;
            let subtotal_acumulado = (cantidad as i128 * precio_unitario as i128 * (porcentaje_anterior + porcentaje_actual) as i128 / 10_000_000) as i64;
            total_certificado += subtotal_actual;
            item_data.push((item.try_get::<String>("", "id").unwrap_or_default(), cantidad, precio_unitario, porcentaje_anterior, porcentaje_actual, subtotal_actual, subtotal_acumulado));
        }

        // Insert the certificate FIRST (FK target for items).
        let ajuste_uocra_monto = (total_certificado as i128 * ajuste_uocra as i128 / 10_000_000) as i64;
        let total_neto = total_certificado - ajuste_uocra_monto - otros_descuentos;

        let sql = format!(
            "INSERT INTO certificados (id, orden_trabajo_id, numero, fecha, observaciones, \
             total_certificado, ajuste_uocra, otros_descuentos, total_neto, created_at) \
             VALUES ('{}', '{}', 1, '{}', 'Importado del sistema anterior. Certificado único reconstruido.', \
             {}, {}, {}, {}, '{}')",
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
        for (item_id_orig, cantidad, precio_unitario, porcentaje_anterior, porcentaje_actual, subtotal_actual, subtotal_acumulado) in &item_data {
            let item_id = uuid::Uuid::now_v7().to_string();
            let sql = format!(
                "INSERT INTO certificado_items (id, certificado_id, orden_trabajo_item_id, \
                 cantidad, precio_unitario, porcentaje_anterior, porcentaje_actual, \
                 subtotal_actual, subtotal_acumulado, created_at) \
                 VALUES ('{}', '{}', '{}', {}, {}, {}, {}, {}, {}, '{}')",
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

/// Derives liquidacion_adelantos from movements of type Adelanto.
/// Returns the count of derived advance records.
async fn derive_liquidacion_adelantos(
    db: &DatabaseTransaction,
    report: &mut ImportReport,
) -> Result<u64> {
    // Get all liquidations ordered by fecha_inicio, then created_at.
    let liquidaciones = db
        .query_all(Statement::from_string(
            DatabaseBackend::Sqlite,
            "SELECT id, empleado_id, fecha_inicio, fecha_fin, total_adelantos \
             FROM liquidaciones WHERE is_deleted = 0 \
             ORDER BY fecha_inicio, created_at"
                .to_owned(),
        ))
        .await
        .context("getting liquidations for advance derivation")?;

    let mut count = 0u64;
    let mut used_movements: std::collections::HashSet<String> = std::collections::HashSet::new();

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
                    "SELECT id, fecha, monto, concepto FROM movimientos \
                     WHERE tipo_movimiento_id = '00000000-0000-0000-0000-000000000003' \
                     AND empleado_id = '{}' \
                     AND fecha >= '{}' AND fecha <= '{}' \
                     AND is_deleted = 0 \
                     ORDER BY fecha",
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
                "INSERT INTO liquidacion_adelantos (id, liquidacion_id, movimiento_id, \
                 monto, fecha, concepto, created_at) \
                 VALUES ('{}', '{}', '{}', {}, '{}', '{}', '{}')",
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

/// Derives cliente_contactos from Clientes.Email.
/// Returns the count of new contacts created.
#[allow(unused_variables)]
async fn derive_contactos(
    db: &DatabaseTransaction,
    report: &mut ImportReport,
) -> Result<u64> {
    // Find clients with an email that doesn't already have a matching contact.
    let clientes = db
        .query_all(Statement::from_string(
            DatabaseBackend::Sqlite,
            "SELECT c.id, c.email, c.created_at FROM clientes c \
             WHERE c.email IS NOT NULL AND c.email != '' AND c.is_deleted = 0"
                .to_owned(),
        ))
        .await
        .context("getting clients with email")?;

    let mut count = 0u64;

    for cliente in &clientes {
        let cliente_id: String = cliente.try_get("", "id").unwrap_or_default();
        let email: String = cliente.try_get("", "email").unwrap_or_default();
        let created_at: String = cliente.try_get("", "created_at").unwrap_or_default();

        // Check if a contact with this email already exists.
        let existing = db
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                format!(
                    "SELECT id FROM cliente_contactos \
                     WHERE cliente_id = '{}' AND LOWER(email) = LOWER('{}') AND is_deleted = 0",
                    cliente_id.replace('\'', "''"),
                    email.replace('\'', "''"),
                ),
            ))
            .await
            .context("checking existing contact")?;

        if existing.is_some() {
            // Mark existing as principal.
            db.execute(Statement::from_string(
                DatabaseBackend::Sqlite,
                format!(
                    "UPDATE cliente_contactos SET es_principal = 1 \
                     WHERE cliente_id = '{}' AND LOWER(email) = LOWER('{}') AND is_deleted = 0",
                    cliente_id.replace('\'', "''"),
                    email.replace('\'', "''"),
                ),
            ))
            .await
            .context("marking contact as principal")?;
        } else {
            // Create new contact.
            let contacto_id = uuid::Uuid::now_v7().to_string();
            let sql = format!(
                "INSERT INTO cliente_contactos (id, cliente_id, email, etiqueta, nombre, telefono, \
                 es_principal, created_at, updated_at, row_version, is_deleted) \
                 VALUES ('{}', '{}', '{}', 'Principal', NULL, NULL, 1, '{}', '{}', X'0000000000000001', 0)",
                contacto_id,
                cliente_id.replace('\'', "''"),
                email.replace('\'', "''"),
                created_at,
                chrono::Utc::now().to_rfc3339(),
            );
            db.execute(Statement::from_string(DatabaseBackend::Sqlite, sql))
                .await
                .context("inserting derived contact")?;
            count += 1;
        }
    }

    // Ensure every client with contacts has at least one principal.
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        "UPDATE cliente_contactos SET es_principal = 1 \
         WHERE rowid IN ( \
             SELECT cc.rowid FROM cliente_contactos cc \
             WHERE cc.is_deleted = 0 AND cc.es_principal = 0 \
             AND NOT EXISTS ( \
                 SELECT 1 FROM cliente_contactos cc2 \
                 WHERE cc2.cliente_id = cc.cliente_id AND cc2.es_principal = 1 AND cc2.is_deleted = 0 \
             ) \
             AND cc.created_at = ( \
                 SELECT MIN(cc3.created_at) FROM cliente_contactos cc3 \
                 WHERE cc3.cliente_id = cc.cliente_id AND cc3.is_deleted = 0 \
             ) \
         )"
            .to_owned(),
    ))
    .await
    .context("ensuring principal contacts")?;

    Ok(count)
}

/// Derives feriados from the legacy config file.
///
/// The old system stored holidays in `appsettings.json` under `Application:Settlement:Holidays`
/// with two incompatible serializations. We try both.
/// Returns the count of recovered holidays.
#[allow(unused_variables)]
async fn derive_feriados(
    db: &DatabaseTransaction,
    legacy: &SqlitePool,
    report: &mut ImportReport,
) -> Result<u64> {
    // Read the legacy config from AppMetadata if available.
    let config_json: Option<String> = db
        .query_one(Statement::from_string(
            DatabaseBackend::Sqlite,
            "SELECT value FROM app_metadata WHERE key = 'config.json'".to_owned(),
        ))
        .await?
        .and_then(|r| r.try_get::<String>("", "value").ok());

    let Some(json_str) = config_json else {
        return Ok(0);
    };

    let parsed: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(_) => return Ok(0),
    };

    // Navigate to Application.Settlement.Holidays.
    let holidays = parsed
        .get("Application")
        .and_then(|a| a.get("Settlement"))
        .and_then(|s| s.get("Holidays"));

    let Some(holidays) = holidays else {
        return Ok(0);
    };

    let mut count = 0u64;

    // Try format 1: [{"Date": "...", "Name": "..."}]
    if let Some(arr) = holidays.as_array() {
        for item in arr {
            let fecha = if let Some(date) = item.get("Date").and_then(|d| d.as_str()) {
                date.to_owned()
            } else if let Some(date) = item.as_str() {
                // Format 2: ["2026-01-01T00:00:00"]
                date.split('T').next().unwrap_or(date).to_owned()
            } else {
                report.warn(
                    WarningCode::FeriadoNoParseable,
                    "Feriados",
                    None,
                    serde_json::json!({ "raw": item }),
                );
                continue;
            };

            let nombre = item
                .get("Name")
                .and_then(|n| n.as_str())
                .unwrap_or("Feriado")
                .to_owned();

            // Normalize date to YYYY-MM-DD.
            let fecha_normalized = fecha.split('T').next().unwrap_or(&fecha);

            let sql = format!(
                "INSERT OR IGNORE INTO feriados (fecha, nombre, tipo, origen, created_at) \
                 VALUES ('{}', '{}', NULL, 'Manual', '{}')",
                fecha_normalized.replace('\'', "''"),
                nombre.replace('\'', "''"),
                chrono::Utc::now().to_rfc3339(),
            );
            db.execute(Statement::from_string(DatabaseBackend::Sqlite, sql))
                .await
                .context("inserting derived feriado")?;
            count += 1;
        }
    }

    Ok(count)
}

/// Reclassifies invoice states based on payments.
async fn reclassify_facturas(db: &DatabaseTransaction) -> Result<FacturasReclasificadas> {
    let mut result = FacturasReclasificadas::default();

    let facturas = db
        .query_all(Statement::from_string(
            DatabaseBackend::Sqlite,
            "SELECT id, estado, total, fecha_vencimiento FROM facturas \
             WHERE estado IN (1, 2) AND is_deleted = 0"
                .to_owned(),
        ))
        .await
        .context("getting invoices for reclassification")?;

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    for factura in &facturas {
        let id: String = factura.try_get("", "id").unwrap_or_default();
        let estado: i64 = factura.try_get("", "estado").unwrap_or(0);
        let total: i64 = factura.try_get("", "total").unwrap_or(0);
        let fecha_vencimiento: String = factura.try_get("", "fecha_vencimiento").unwrap_or_default();

        let total_pagado: i64 = db
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                format!(
                    "SELECT COALESCE(SUM(monto), 0) as total_pagado FROM pagos_factura \
                     WHERE factura_id = '{}' AND is_deleted = 0",
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
