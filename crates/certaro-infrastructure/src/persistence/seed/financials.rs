//! Seeding for Invoices, Payments, Movements, Payroll and Attachments.

use certaro_application::result::AppResult;
use certaro_application::AppError;
use certaro_domain::RowVersion;
use sea_orm::{ActiveModelTrait, DatabaseTransaction, Set};
use uuid::Uuid;

use crate::persistence::models::{
    adjunto, factura, feriado, liquidacion, liquidacion_adelanto, movimiento, pago_factura,
};
use super::data::FERIADOS_DATA;

pub async fn seed_financials_and_attachments(
    tx: &DatabaseTransaction,
    now: &str,
    clientes_ids: &[String],
    empleados_ids: &[String],
    proyectos_ids: &[String],
    trabajos_ids: &[String],
    categorias_ids: &[String],
    tipos_ids: &[String],
) -> AppResult<(Vec<String>, usize, usize, usize, usize, usize, usize)> {
    // 10. Facturas & Pagos
    let mut facturas_ids = Vec::new();
    let mut pagos_count = 0;
    let facturas_data = [
        ("0001-00000101", &clientes_ids[0], 2, 8_500_000_000_i64, 1_785_000_000_i64, 10_285_000_000_i64),
        ("0001-00000102", &clientes_ids[2], 3, 12_000_000_000_i64, 2_520_000_000_i64, 14_520_000_000_i64),
        ("0001-00000103", &clientes_ids[1], 1, 6_200_000_000_i64, 1_302_000_000_i64, 7_502_000_000_i64),
    ];
    for (num, cli_id, estado, sub, iva, tot) in facturas_data {
        let fact_id = Uuid::now_v7().to_string();
        let f = factura::ActiveModel {
            id: Set(fact_id.clone()),
            numero: Set(num.to_string()),
            fecha: Set("2025-02-10".to_string()),
            fecha_vencimiento: Set(Some("2025-03-10".to_string())),
            cliente_id: Set(cli_id.clone()),
            estado: Set(estado),
            subtotal: Set(sub),
            iva: Set(iva),
            total: Set(tot),
            observaciones: Set(Some("Facturación de servicios eléctricos".to_string())),
            created_at: Set(now.to_string()),
            updated_at: Set(None),
            row_version: Set(RowVersion::INITIAL.as_bytes().to_vec()),
            is_deleted: Set(false),
            deleted_at: Set(None),
        };
        f.insert(tx).await.map_err(AppError::persistence)?;

        if estado == 3 {
            let pago = pago_factura::ActiveModel {
                id: Set(Uuid::now_v7().to_string()),
                factura_id: Set(fact_id.clone()),
                fecha: Set("2025-02-18".to_string()),
                monto: Set(tot),
                medio_pago: Set("Transferencia Bancaria".to_string()),
                created_at: Set(now.to_string()),
                updated_at: Set(None),
                row_version: Set(RowVersion::INITIAL.as_bytes().to_vec()),
                is_deleted: Set(false),
                deleted_at: Set(None),
            };
            pago.insert(tx).await.map_err(AppError::persistence)?;
            pagos_count += 1;
        }

        facturas_ids.push(fact_id);
    }

    // 11. Movimientos de Caja
    let ingreso_sistema = certaro_domain::constants::tipos_movimiento::INGRESO.to_string();
    let egreso_sistema = certaro_domain::constants::tipos_movimiento::GASTO.to_string();
    let adelanto_sistema = certaro_domain::constants::tipos_movimiento::ADELANTO.to_string();

    let movimientos_data = [
        ("Cobro Certificado N.º 1 Torre Alvear", 14_520_000_000_i64, 10_000_i64, &ingreso_sistema, Some(&categorias_ids[4]), Some(&clientes_ids[2]), Some(&trabajos_ids[0]), None, Some(&facturas_ids[1])),
        ("Anticipo Proyecto Planta del Plata", 5_000_000_000_i64, 10_000_i64, &ingreso_sistema, Some(&categorias_ids[4]), Some(&clientes_ids[0]), Some(&trabajos_ids[2]), None, None),
        ("Venta de cables sobrantes de cobre", 850_000_000_i64, 10_000_i64, &tipos_ids[0], Some(&categorias_ids[0]), None, None, None, None),
        ("Compra de cables sintetizados y termomagnéticas", 3_400_000_000_i64, 10_000_i64, &egreso_sistema, Some(&categorias_ids[1]), None, Some(&trabajos_ids[0]), None, None),
        ("Adquisición de pinza amperimétrica True RMS", 1_250_000_000_i64, 10_000_i64, &egreso_sistema, Some(&categorias_ids[3]), None, None, None, None),
        ("Combustible y peajes traslados a Tigre", 450_000_000_i64, 10_000_i64, &egreso_sistema, Some(&categorias_ids[6]), None, Some(&trabajos_ids[2]), None, None),
        ("Pago de Monotributo / IIBB mensual", 620_000_000_i64, 10_000_i64, &egreso_sistema, Some(&categorias_ids[5]), None, None, None, None),
        ("Adelanto quincenal Ricardo Darín", 500_000_000_i64, 10_000_i64, &adelanto_sistema, None, None, None, Some(&empleados_ids[0]), None),
        ("Adelanto quincenal Natalia Oreiro", 400_000_000_i64, 10_000_i64, &adelanto_sistema, None, None, None, Some(&empleados_ids[2]), None),
    ];

    let mut movimientos_count = 0;
    let mut adelanto_movimiento_id = String::new();
    for (conc, monto, cant, tipo_id, cat_id, cli_id, trab_id, emp_id, fact_id) in movimientos_data {
        let mov_id = Uuid::now_v7().to_string();
        if conc.contains("Ricardo Darín") {
            adelanto_movimiento_id = mov_id.clone();
        }
        let mov = movimiento::ActiveModel {
            id: Set(mov_id),
            fecha: Set("2025-02-18T14:30:00Z".to_string()),
            concepto: Set(conc.to_string()),
            monto: Set(monto),
            cantidad: Set(cant),
            tipo_movimiento_id: Set(tipo_id.to_string()),
            moneda: Set(0),
            cotizacion_aplicada: Set(None),
            tipo_concepto_pago_id: Set(None),
            categoria_id: Set(cat_id.cloned()),
            cliente_id: Set(cli_id.cloned()),
            trabajo_id: Set(trab_id.cloned()),
            empleado_id: Set(emp_id.cloned()),
            factura_id: Set(fact_id.cloned()),
            created_at: Set(now.to_string()),
            updated_at: Set(None),
            row_version: Set(RowVersion::INITIAL.as_bytes().to_vec()),
            is_deleted: Set(false),
            deleted_at: Set(None),
        };
        mov.insert(tx).await.map_err(AppError::persistence)?;
        movimientos_count += 1;
    }

    // 12. Liquidaciones & Descuento de Adelantos
    let mut liquidaciones_count = 0;
    let mut liq_adelantos_count = 0;
    for (i, emp_id) in empleados_ids.iter().enumerate().take(2) {
        let liq_id = Uuid::now_v7().to_string();
        let liq = liquidacion::ActiveModel {
            id: Set(liq_id.clone()),
            empleado_id: Set(emp_id.clone()),
            fecha_inicio: Set("2025-02-01".to_string()),
            fecha_fin: Set("2025-02-15".to_string()),
            dias_trabajados: Set(110_000),
            tarifa_aplicada: Set(if i == 0 { 450_000_000 } else { 550_000_000 }),
            incluir_sabados: Set(true),
            incluir_domingos: Set(false),
            incluir_feriados: Set(false),
            multiplicador_sabado: Set(15_000),
            multiplicador_domingo: Set(20_000),
            multiplicador_feriado: Set(20_000),
            total_bruto: Set(if i == 0 { 4_950_000_000 } else { 6_050_000_000 }),
            total_adelantos: Set(if i == 0 { 500_000_000 } else { 0 }),
            observaciones: Set(Some("Liquidación 1ra Quincena Febrero 2025".to_string())),
            pdf_generado_at: Set(None),
            created_at: Set(now.to_string()),
            updated_at: Set(None),
            row_version: Set(RowVersion::INITIAL.as_bytes().to_vec()),
            is_deleted: Set(false),
            deleted_at: Set(None),
        };
        liq.insert(tx).await.map_err(AppError::persistence)?;
        liquidaciones_count += 1;

        if i == 0 && !adelanto_movimiento_id.is_empty() {
            let liq_ad = liquidacion_adelanto::ActiveModel {
                id: Set(Uuid::now_v7().to_string()),
                liquidacion_id: Set(liq_id.clone()),
                movimiento_id: Set(adelanto_movimiento_id.clone()),
                monto: Set(500_000_000),
                fecha: Set("2025-02-05".to_string()),
                concepto: Set("Adelanto quincenal Ricardo Darín".to_string()),
                created_at: Set(now.to_string()),
                updated_at: Set(None),
                row_version: Set(RowVersion::INITIAL.as_bytes().to_vec()),
                is_deleted: Set(false),
                deleted_at: Set(None),
            };
            liq_ad.insert(tx).await.map_err(AppError::persistence)?;
            liq_adelantos_count += 1;
        }
    }

    // 13. Feriados
    let mut feriados_count = 0;
    for (fecha_fer, nom, tip) in FERIADOS_DATA {
        let f = feriado::ActiveModel {
            fecha: Set(fecha_fer.to_string()),
            nombre: Set(nom.to_string()),
            tipo: Set(Some(tip.to_string())),
            origen: Set("Api".to_string()),
            created_at: Set(now.to_string()),
            updated_at: Set(None),
        };
        let _ = f.insert(tx).await;
        feriados_count += 1;
    }

    // 14. Adjuntos de Prueba
    let adjuntos_data = [
        ("Proyecto", &proyectos_ids[0], "plano_unifilar_torre_alvear.pdf", "proyectos/plano_unifilar.pdf", "application/pdf", 1_048_576),
        ("Factura", &facturas_ids[1], "comprobante_transferencia_102.pdf", "facturas/comprobante_102.pdf", "application/pdf", 256_000),
        ("Empleado", &empleados_ids[0], "constancia_alta_afip_darin.pdf", "empleados/alta_afip_darin.pdf", "application/pdf", 512_000),
    ];

    let mut adjuntos_count = 0;
    for (tipo_ent, ent_id, nom_arch, ruta, mime, tam) in adjuntos_data {
        let adj = adjunto::ActiveModel {
            id: Set(Uuid::now_v7().to_string()),
            entidad_tipo: Set(tipo_ent.to_string()),
            entidad_id: Set(ent_id.clone()),
            nombre_archivo: Set(nom_arch.to_string()),
            ruta_relativa: Set(ruta.to_string()),
            mime: Set(mime.to_string()),
            tamano: Set(tam),
            created_at: Set(now.to_string()),
            updated_at: Set(None),
            row_version: Set(RowVersion::INITIAL.as_bytes().to_vec()),
            is_deleted: Set(false),
            deleted_at: Set(None),
        };
        adj.insert(tx).await.map_err(AppError::persistence)?;
        adjuntos_count += 1;
    }

    Ok((facturas_ids, pagos_count, movimientos_count, liquidaciones_count, liq_adelantos_count, feriados_count, adjuntos_count))
}
