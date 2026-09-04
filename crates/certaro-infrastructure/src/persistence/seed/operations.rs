//! Seeding for Projects, Jobs, Attendance, Work Orders and Certificates.

use chrono::Datelike;
use certaro_application::result::AppResult;
use certaro_application::AppError;
use certaro_domain::RowVersion;
use sea_orm::{ActiveModelTrait, DatabaseTransaction, Set};
use uuid::Uuid;

use crate::persistence::models::{
    asistencia_empleado, certificado, certificado_item, orden_trabajo, orden_trabajo_item, proyecto, trabajo,
};

pub async fn seed_projects_and_jobs(
    tx: &DatabaseTransaction,
    now: &str,
    clientes_ids: &[String],
    empleados_ids: &[String],
) -> AppResult<(Vec<String>, Vec<String>, Vec<String>, Vec<String>, usize, usize, usize)> {
    // 5. Proyectos
    let proyectos_data = [
        ("Instalación Eléctrica Integral Torre Alvear", "Av. Alvear 1890", "CABA", &clientes_ids[2], 1, 1),
        ("Iluminación y Fuerza Motriz Planta del Plata", "Parque Industrial Norte", "Tigre", &clientes_ids[0], 2, 1),
        ("Cableado Estructurado Oficinas Centro", "San Martín 567", "Rosario", &clientes_ids[1], 3, 3),
        ("Refacción y Tablero Eléctrico Domiciliario", "Belgrano 432", "San Isidro", &clientes_ids[3], 4, 1),
    ];

    let mut proyectos_ids = Vec::new();
    for (nombre, dir, loc, cli_id, num, estado) in proyectos_data {
        let id = Uuid::now_v7().to_string();
        let ob = proyecto::ActiveModel {
            id: Set(id.clone()),
            numero: Set(num),
            nombre: Set(nombre.to_string()),
            direccion: Set(Some(dir.to_string())),
            localidad: Set(Some(loc.to_string())),
            cliente_id: Set(cli_id.clone()),
            estado: Set(estado),
            created_at: Set(now.to_string()),
            updated_at: Set(None),
            row_version: Set(RowVersion::INITIAL.as_bytes().to_vec()),
            is_deleted: Set(false),
            deleted_at: Set(None),
        };
        ob.insert(tx).await.map_err(AppError::persistence)?;
        proyectos_ids.push(id);
    }

    // 6. Trabajos
    let trabajos_data = [
        (&proyectos_ids[0], "Tendido de bandejas portacables en subsuelos", "2025-02-01", 18_500_000_000_i64, 2),
        (&proyectos_ids[0], "Montaje de tableros seccionales por piso", "2025-02-10", 32_000_000_000_i64, 2),
        (&proyectos_ids[1], "Iluminación perimetral LED alta potencia", "2025-01-20", 9_500_000_000_i64, 3),
        (&proyectos_ids[2], "Puestos de red Cat6 y rack central", "2025-01-10", 14_000_000_000_i64, 3),
        (&proyectos_ids[3], "Recableado completo y disyuntor diferencial", "2025-02-15", 6_500_000_000_i64, 2),
    ];

    let mut trabajos_ids = Vec::new();
    for (proyecto_id, desc, fecha_ini, presup, estado) in trabajos_data {
        let id = Uuid::now_v7().to_string();
        let trab = trabajo::ActiveModel {
            id: Set(id.clone()),
            proyecto_id: Set(proyecto_id.clone()),
            descripcion: Set(desc.to_string()),
            fecha_inicio: Set(fecha_ini.to_string()),
            fecha_fin: Set(if estado == 3 { Some("2025-02-25".to_string()) } else { None }),
            presupuesto: Set(presup),
            estado: Set(estado),
            created_at: Set(now.to_string()),
            updated_at: Set(None),
            row_version: Set(RowVersion::INITIAL.as_bytes().to_vec()),
            is_deleted: Set(false),
            deleted_at: Set(None),
        };
        trab.insert(tx).await.map_err(AppError::persistence)?;
        trabajos_ids.push(id);
    }

    // 7. Asistencias de Empleados
    let mut asistencias_count = 0;
    let feriados_set: std::collections::HashSet<String> = [
        "2025-06-16", "2025-06-20", "2025-07-09", "2025-08-17", "2025-10-12", "2025-11-24", "2025-12-08", "2025-12-25",
        "2026-01-01", "2026-03-03", "2026-03-04", "2026-03-24", "2026-04-02",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let hoy = chrono::Utc::now().date_naive();
    let desde = hoy - chrono::Duration::days(92);
    for (emp_idx, emp_id) in empleados_ids.iter().enumerate() {
        let mut fecha = desde;
        let mut dia_idx = 0;
        while fecha <= hoy {
            let fecha_asist = fecha.format("%Y-%m-%d").to_string();
            let es_feriado = feriados_set.contains(&fecha_asist);
            let tipo_jornada = if es_feriado {
                if dia_idx % 2 == 0 { 4 } else { 2 }
            } else {
                match fecha.weekday() {
                    chrono::Weekday::Sat => {
                        if dia_idx % 3 == 0 { 0 } else { 2 }
                    }
                    chrono::Weekday::Sun => 2,
                    _ => match dia_idx % 20 {
                        0..=13 => 0,
                        14..=16 => 1,
                        17..=18 => 2,
                        _ => 3,
                    },
                }
            };
            dia_idx += 1;
            let asist = asistencia_empleado::ActiveModel {
                id: Set(Uuid::now_v7().to_string()),
                empleado_id: Set(emp_id.clone()),
                fecha: Set(fecha_asist),
                tipo_jornada: Set(tipo_jornada),
                trabajo_id: Set(Some(trabajos_ids[emp_idx % trabajos_ids.len()].clone())),
                observaciones: Set(Some("Jornada cumplida en proyecto".to_string())),
                created_at: Set(now.to_string()),
                updated_at: Set(None),
                row_version: Set(RowVersion::INITIAL.as_bytes().to_vec()),
                is_deleted: Set(false),
                deleted_at: Set(None),
            };
            asist.insert(tx).await.map_err(AppError::persistence)?;
            asistencias_count += 1;
            fecha += chrono::Duration::days(1);
        }
    }

    // 8. Ordenes de Trabajo & Items
    let mut ordenes_ids = Vec::new();
    let mut items_ids = Vec::new();
    for (i, trab_id) in trabajos_ids.iter().enumerate().take(3) {
        let orden_id = Uuid::now_v7().to_string();
        let ot = orden_trabajo::ActiveModel {
            id: Set(orden_id.clone()),
            trabajo_id: Set(trab_id.clone()),
            titulo: Set(format!("Certificación de Avance Etapa {}", i + 1)),
            numero_certificado: Set(Some(format!("CERT-{:03}", i + 1))),
            fecha: Set("2025-02-20".to_string()),
            observaciones: Set(Some("Avance verificado en proyecto con dirección facultativa".to_string())),
            ajuste_uocra_porcentaje: Set(80_000),
            otros_descuentos: Set(0),
            created_at: Set(now.to_string()),
            updated_at: Set(None),
            row_version: Set(RowVersion::INITIAL.as_bytes().to_vec()),
            is_deleted: Set(false),
            deleted_at: Set(None),
        };
        ot.insert(tx).await.map_err(AppError::persistence)?;

        let item1_id = Uuid::now_v7().to_string();
        let item1 = orden_trabajo_item::ActiveModel {
            id: Set(item1_id.clone()),
            orden_trabajo_id: Set(orden_id.clone()),
            descripcion: Set("Tendido de cañería y cableado".to_string()),
            unidad: Set("MTS".to_string()),
            cantidad: Set(250_0000),
            precio_unitario: Set(45_000_000),
            porcentaje_anterior: Set(0),
            porcentaje_actual: Set(400_000),
            ejecutado: Set(false),
            nota: Set(None),
            orden: Set(1),
            created_at: Set(now.to_string()),
            updated_at: Set(None),
            row_version: Set(RowVersion::INITIAL.as_bytes().to_vec()),
            is_deleted: Set(false),
            deleted_at: Set(None),
        };
        item1.insert(tx).await.map_err(AppError::persistence)?;
        items_ids.push(item1_id);

        let item2_id = Uuid::now_v7().to_string();
        let item2 = orden_trabajo_item::ActiveModel {
            id: Set(item2_id.clone()),
            orden_trabajo_id: Set(orden_id.clone()),
            descripcion: Set("Instalación y conexión de luminarias".to_string()),
            unidad: Set("UN".to_string()),
            cantidad: Set(30_0000),
            precio_unitario: Set(120_000_000),
            porcentaje_anterior: Set(0),
            porcentaje_actual: Set(500_000),
            ejecutado: Set(false),
            nota: Set(None),
            orden: Set(2),
            created_at: Set(now.to_string()),
            updated_at: Set(None),
            row_version: Set(RowVersion::INITIAL.as_bytes().to_vec()),
            is_deleted: Set(false),
            deleted_at: Set(None),
        };
        item2.insert(tx).await.map_err(AppError::persistence)?;
        items_ids.push(item2_id);

        ordenes_ids.push(orden_id);
    }

    // 9. Certificados & Certificado Items
    let mut certificados_count = 0;
    let mut cert_items_count = 0;
    for (i, ot_id) in ordenes_ids.iter().enumerate().take(2) {
        let cert_id = Uuid::now_v7().to_string();
        let cert = certificado::ActiveModel {
            id: Set(cert_id.clone()),
            orden_trabajo_id: Set(ot_id.clone()),
            numero: Set((i + 1) as i32),
            fecha: Set("2025-02-22".to_string()),
            observaciones: Set(Some("Certificado de proyecto aprobado".to_string())),
            total_certificado: Set(18_500_000_000),
            ajuste_uocra: Set(1_480_000_000),
            otros_descuentos: Set(0),
            total_neto: Set(19_980_000_000),
            created_at: Set(now.to_string()),
            updated_at: Set(None),
            row_version: Set(RowVersion::INITIAL.as_bytes().to_vec()),
            is_deleted: Set(false),
            deleted_at: Set(None),
        };
        cert.insert(tx).await.map_err(AppError::persistence)?;
        certificados_count += 1;

        let citem = certificado_item::ActiveModel {
            id: Set(Uuid::now_v7().to_string()),
            certificado_id: Set(cert_id.clone()),
            orden_trabajo_item_id: Set(items_ids[i].clone()),
            cantidad: Set(250_0000),
            precio_unitario: Set(45_000_000),
            porcentaje_anterior: Set(0),
            porcentaje_actual: Set(400_000),
            subtotal_actual: Set(4_500_000_000),
            subtotal_acumulado: Set(4_500_000_000),
            created_at: Set(now.to_string()),
            updated_at: Set(None),
            row_version: Set(RowVersion::INITIAL.as_bytes().to_vec()),
            is_deleted: Set(false),
            deleted_at: Set(None),
        };
        citem.insert(tx).await.map_err(AppError::persistence)?;
        cert_items_count += 1;
    }

    Ok((proyectos_ids, trabajos_ids, ordenes_ids, items_ids, asistencias_count, certificados_count, cert_items_count))
}
