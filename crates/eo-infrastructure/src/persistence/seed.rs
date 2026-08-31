//! Seeding engine for development and testing demo data.
//!
//! Generates a rich, realistic dataset including categories, custom movement types,
//! employees, clients with contacts, sites (obras), jobs (trabajos), work orders,
//! cash movements, invoices, and payroll settlements.

use chrono::Utc;
use eo_application::result::AppResult;
use eo_application::AppError;
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set, TransactionTrait};
use serde::Serialize;
use uuid::Uuid;

use crate::persistence::models::{
    categoria, cliente, cliente_contacto, empleado, factura, liquidacion,
    movimiento, obra, orden_trabajo, orden_trabajo_item, tipo_movimiento, trabajo,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeedSummary {
    pub categorias: usize,
    pub tipos_movimiento: usize,
    pub empleados: usize,
    pub clientes: usize,
    pub obras: usize,
    pub trabajos: usize,
    pub ordenes_trabajo: usize,
    pub movimientos: usize,
    pub facturas: usize,
    pub liquidaciones: usize,
}

pub async fn seed_demo_data(db: &DatabaseConnection) -> AppResult<SeedSummary> {
    let tx = db.begin().await.map_err(AppError::persistence)?;
    let now = Utc::now().to_rfc3339();

    // 1. Categorías
    let categorias_data = [
        ("Materiales", "#3B82F6", "package"),
        ("Herramientas", "#F59E0B", "wrench"),
        ("Servicios", "#10B981", "briefcase"),
        ("Impuestos", "#EF4444", "receipt"),
        ("Varios", "#8B5CF6", "layers"),
        ("Viáticos", "#06B6D4", "truck"),
        ("Publicidad", "#EC4899", "megaphone"),
        ("Alquiler", "#6366F1", "building"),
    ];

    let mut categorias_ids = Vec::new();
    for (nombre, color, icono) in categorias_data {
        let id = Uuid::now_v7().to_string();
        let cat = categoria::ActiveModel {
            id: Set(id.clone()),
            nombre: Set(nombre.to_string()),
            descripcion: Set(Some(format!("Gastos e insumos de {nombre}"))),
            color_hex: Set(Some(color.to_string())),
            icono: Set(Some(icono.to_string())),
            categoria_padre_id: Set(None),
            created_at: Set(now.clone()),
            updated_at: Set(None),
            row_version: Set(Uuid::now_v7().as_bytes().to_vec()),
            is_deleted: Set(false),
            deleted_at: Set(None),
        };
        cat.insert(&tx).await.map_err(AppError::persistence)?;
        categorias_ids.push(id);
    }

    // 2. Tipos de Movimiento personalizados
    let custom_tipos = [
        ("Venta de chatarra / sobrantes", true),
        ("Alquiler de andamios y equipos", true),
        ("Honorarios asesoría técnica", false),
    ];
    let mut tipos_ids = Vec::new();
    for (nombre, es_ingreso) in custom_tipos {
        let id = Uuid::now_v7().to_string();
        let tipo = tipo_movimiento::ActiveModel {
            id: Set(id.clone()),
            nombre: Set(nombre.to_string()),
            descripcion: Set(None),
            es_ingreso: Set(es_ingreso),
            es_sistema: Set(false),
            created_at: Set(now.clone()),
            updated_at: Set(None),
            row_version: Set(Uuid::now_v7().as_bytes().to_vec()),
            is_deleted: Set(false),
            deleted_at: Set(None),
        };
        tipo.insert(&tx).await.map_err(AppError::persistence)?;
        tipos_ids.push(id);
    }

    // Fetch seeded system tipo movimiento IDs
    let ingreso_sistema = eo_domain::constants::tipos_movimiento::INGRESO.to_string();
    let egreso_sistema = eo_domain::constants::tipos_movimiento::GASTO.to_string();
    let adelanto_sistema = eo_domain::constants::tipos_movimiento::ADELANTO.to_string();

    // 3. Empleados (valores Money en escala 4: $450.000 = 4_500_000_000)
    let empleados_data = [
        ("Ricardo Darín", "20.123.456", "Operario Electricista", 4_500_000_000_i64, 450_000_000_i64, "1145678901", "ricardo.darin@obra.com"),
        ("Guillermo Francella", "22.345.678", "Capataz de Obra", 5_500_000_000_i64, 550_000_000_i64, "1145678902", "guillermo.francella@obra.com"),
        ("Natalia Oreiro", "25.678.901", "Técnica Instaladora", 4_800_000_000_i64, 480_000_000_i64, "1145678903", "natalia.oreiro@obra.com"),
        ("Diego Peretti", "18.901.234", "Ayudante Práctico", 3_800_000_000_i64, 380_000_000_i64, "1145678904", "diego.peretti@obra.com"),
        ("Érica Rivas", "27.234.567", "Administrativa de Obra", 4_200_000_000_i64, 420_000_000_i64, "1145678905", "erica.rivas@obra.com"),
    ];

    let mut empleados_ids = Vec::new();
    for (nombre, dni, cargo, sueldo, tarifa, tel, mail) in empleados_data {
        let id = Uuid::now_v7().to_string();
        let emp = empleado::ActiveModel {
            id: Set(id.clone()),
            nombre: Set(nombre.to_string()),
            dni: Set(Some(dni.to_string())),
            cargo: Set(Some(cargo.to_string())),
            sueldo_base: Set(sueldo),
            pago_frecuencia: Set(1), // Quincenal
            tarifa_diaria: Set(tarifa),
            multiplicador_sabado: Set(15_000), // 1.5
            multiplicador_domingo: Set(20_000), // 2.0
            multiplicador_feriado: Set(20_000), // 2.0
            email: Set(Some(mail.to_string())),
            telefono: Set(Some(tel.to_string())),
            fecha_ingreso: Set("2025-01-15".to_string()),
            fecha_egreso: Set(None),
            activo: Set(true),
            created_at: Set(now.clone()),
            updated_at: Set(None),
            row_version: Set(Uuid::now_v7().as_bytes().to_vec()),
            is_deleted: Set(false),
            deleted_at: Set(None),
        };
        emp.insert(&tx).await.map_err(AppError::persistence)?;
        empleados_ids.push(id);
    }

    // 4. Clientes & Contactos
    let clientes_data = [
        ("Constructora del Plata S.A.", "30-71234567-9", "Av. del Libertador 1234, CABA", "011-4567-8900", "info@constructoradelplata.com", "Responsable Inscripto"),
        ("Desarrollos Urbanos SRL", "30-79876543-1", "San Martín 567, Piso 4, Rosario", "0341-423-4567", "administracion@desarrollosurbanos.com", "Responsable Inscripto"),
        ("Consorcio Torre Alvear", "30-65432109-8", "Av. Alvear 1890, CABA", "011-4812-3456", "consorcio@torrealvear.com", "Consumidor Final"),
        ("Juan Carlos Pérez", "20-28123456-3", "Belgrano 432, San Isidro", "011-15-5432-1098", "jcperez@gmail.com", "Consumidor Final"),
    ];

    let mut clientes_ids = Vec::new();
    for (nombre, cuit, dir, tel, mail, iva) in clientes_data {
        let id = Uuid::now_v7().to_string();
        let cli = cliente::ActiveModel {
            id: Set(id.clone()),
            nombre: Set(nombre.to_string()),
            cuit: Set(Some(cuit.to_string())),
            direccion: Set(Some(dir.to_string())),
            telefono: Set(Some(tel.to_string())),
            email: Set(Some(mail.to_string())),
            condicion_iva: Set(Some(iva.to_string())),
            created_at: Set(now.clone()),
            updated_at: Set(None),
            row_version: Set(Uuid::now_v7().as_bytes().to_vec()),
            is_deleted: Set(false),
            deleted_at: Set(None),
        };
        cli.insert(&tx).await.map_err(AppError::persistence)?;

        // Contactos
        let contacto = cliente_contacto::ActiveModel {
            id: Set(Uuid::now_v7().to_string()),
            cliente_id: Set(id.clone()),
            nombre: Set(Some(format!("Contacto {}", nombre.split_whitespace().next().unwrap_or("Principal")))),
            email: Set(mail.to_string()),
            telefono: Set(Some(tel.to_string())),
            etiqueta: Set("Administración / Pagos".to_string()),
            es_principal: Set(true),
            created_at: Set(now.clone()),
            updated_at: Set(None),
            row_version: Set(Uuid::now_v7().as_bytes().to_vec()),
            is_deleted: Set(false),
            deleted_at: Set(None),
        };
        contacto.insert(&tx).await.map_err(AppError::persistence)?;

        clientes_ids.push(id);
    }

    // 5. Obras
    let obras_data = [
        ("Instalación Eléctrica Integral Torre Alvear", "Av. Alvear 1890", "CABA", &clientes_ids[2], 1, 1), // Activa
        ("Iluminación y Fuerza Motriz Planta del Plata", "Parque Industrial Norte", "Tigre", &clientes_ids[0], 2, 1), // Activa
        ("Cableado Estructurado Oficinas Centro", "San Martín 567", "Rosario", &clientes_ids[1], 3, 3), // Finalizada
        ("Refacción y Tablero Eléctrico Domiciliario", "Belgrano 432", "San Isidro", &clientes_ids[3], 4, 1), // Activa
    ];

    let mut obras_ids = Vec::new();
    for (nombre, dir, loc, cli_id, num, estado) in obras_data {
        let id = Uuid::now_v7().to_string();
        let ob = obra::ActiveModel {
            id: Set(id.clone()),
            numero: Set(num),
            nombre: Set(nombre.to_string()),
            direccion: Set(Some(dir.to_string())),
            localidad: Set(Some(loc.to_string())),
            cliente_id: Set(cli_id.clone()),
            estado: Set(estado),
            created_at: Set(now.clone()),
            updated_at: Set(None),
            row_version: Set(Uuid::now_v7().as_bytes().to_vec()),
            is_deleted: Set(false),
            deleted_at: Set(None),
        };
        ob.insert(&tx).await.map_err(AppError::persistence)?;
        obras_ids.push(id);
    }

    // 6. Trabajos
    let trabajos_data = [
        (&obras_ids[0], "Tendido de bandejas portacables en subsuelos", "2025-02-01", 18_500_000_000_i64, 2), // EnProceso
        (&obras_ids[0], "Montaje de tableros seccionales por piso", "2025-02-10", 32_000_000_000_i64, 2), // EnProceso
        (&obras_ids[1], "Iluminación perimetral LED alta potencia", "2025-01-20", 9_500_000_000_i64, 3), // Finalizado
        (&obras_ids[2], "Puestos de red Cat6 y rack central", "2025-01-10", 14_000_000_000_i64, 3), // Finalizado
        (&obras_ids[3], "Recableado completo y disyuntor diferencial", "2025-02-15", 6_500_000_000_i64, 2), // EnProceso
    ];

    let mut trabajos_ids = Vec::new();
    for (obra_id, desc, fecha_ini, presup, estado) in trabajos_data {
        let id = Uuid::now_v7().to_string();
        let trab = trabajo::ActiveModel {
            id: Set(id.clone()),
            obra_id: Set(obra_id.clone()),
            descripcion: Set(desc.to_string()),
            fecha_inicio: Set(fecha_ini.to_string()),
            fecha_fin: Set(if estado == 3 { Some("2025-02-25".to_string()) } else { None }),
            presupuesto: Set(presup),
            estado: Set(estado),
            created_at: Set(now.clone()),
            updated_at: Set(None),
            row_version: Set(Uuid::now_v7().as_bytes().to_vec()),
            is_deleted: Set(false),
            deleted_at: Set(None),
        };
        trab.insert(&tx).await.map_err(AppError::persistence)?;
        trabajos_ids.push(id);
    }

    // 7. Ordenes de Trabajo & Items
    let mut ordenes_ids = Vec::new();
    for (i, trab_id) in trabajos_ids.iter().enumerate().take(3) {
        let orden_id = Uuid::now_v7().to_string();
        let ot = orden_trabajo::ActiveModel {
            id: Set(orden_id.clone()),
            trabajo_id: Set(trab_id.clone()),
            titulo: Set(format!("Certificación de Avance Etapa {}", i + 1)),
            numero_certificado: Set(Some(format!("CERT-{:03}", i + 1))),
            fecha: Set("2025-02-20".to_string()),
            observaciones: Set(Some("Avance verificado en obra con dirección facultativa".to_string())),
            ajuste_uocra_porcentaje: Set(80_000), // 8.0%
            otros_descuentos: Set(0),
            created_at: Set(now.clone()),
            updated_at: Set(None),
            row_version: Set(Uuid::now_v7().as_bytes().to_vec()),
            is_deleted: Set(false),
            deleted_at: Set(None),
        };
        ot.insert(&tx).await.map_err(AppError::persistence)?;

        // Items
        let item1 = orden_trabajo_item::ActiveModel {
            id: Set(Uuid::now_v7().to_string()),
            orden_trabajo_id: Set(orden_id.clone()),
            descripcion: Set("Tendido de cañería y cableado".to_string()),
            unidad: Set("MTS".to_string()),
            cantidad: Set(250_0000), // 250
            precio_unitario: Set(45_000_000), // $4.500
            porcentaje_anterior: Set(0),
            porcentaje_actual: Set(400_000), // 40%
            ejecutado: Set(false),
            nota: Set(None),
            orden: Set(1),
            created_at: Set(now.clone()),
            updated_at: Set(None),
            row_version: Set(Uuid::now_v7().as_bytes().to_vec()),
            is_deleted: Set(false),
            deleted_at: Set(None),
        };
        item1.insert(&tx).await.map_err(AppError::persistence)?;

        let item2 = orden_trabajo_item::ActiveModel {
            id: Set(Uuid::now_v7().to_string()),
            orden_trabajo_id: Set(orden_id.clone()),
            descripcion: Set("Instalación y conexión de luminarias".to_string()),
            unidad: Set("UN".to_string()),
            cantidad: Set(30_0000), // 30
            precio_unitario: Set(120_000_000), // $12.000
            porcentaje_anterior: Set(0),
            porcentaje_actual: Set(500_000), // 50%
            ejecutado: Set(false),
            nota: Set(None),
            orden: Set(2),
            created_at: Set(now.clone()),
            updated_at: Set(None),
            row_version: Set(Uuid::now_v7().as_bytes().to_vec()),
            is_deleted: Set(false),
            deleted_at: Set(None),
        };
        item2.insert(&tx).await.map_err(AppError::persistence)?;

        ordenes_ids.push(orden_id);
    }

    // 8. Facturas
    let mut facturas_ids = Vec::new();
    let facturas_data = [
        ("0001-00000101", &clientes_ids[0], 2, 8_500_000_000_i64, 1_785_000_000_i64, 10_285_000_000_i64), // Emitida
        ("0001-00000102", &clientes_ids[2], 3, 12_000_000_000_i64, 2_520_000_000_i64, 14_520_000_000_i64), // Pagada
        ("0001-00000103", &clientes_ids[1], 1, 6_200_000_000_i64, 1_302_000_000_i64, 7_502_000_000_i64), // Borrador
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
            created_at: Set(now.clone()),
            updated_at: Set(None),
            row_version: Set(Uuid::now_v7().as_bytes().to_vec()),
            is_deleted: Set(false),
            deleted_at: Set(None),
        };
        f.insert(&tx).await.map_err(AppError::persistence)?;
        facturas_ids.push(fact_id);
    }

    // 9. Movimientos de Caja
    let movimientos_data = [
        // Ingresos
        ("Cobro Certificado N.º 1 Torre Alvear", 14_520_000_000_i64, 10_000_i64, &ingreso_sistema, Some(&categorias_ids[2]), Some(&clientes_ids[2]), Some(&trabajos_ids[0]), None, Some(&facturas_ids[1])),
        ("Anticipo Obra Planta del Plata", 5_000_000_000_i64, 10_000_i64, &ingreso_sistema, Some(&categorias_ids[2]), Some(&clientes_ids[0]), Some(&trabajos_ids[2]), None, None),
        ("Venta de cables sobrantes de cobre", 850_000_000_i64, 10_000_i64, &tipos_ids[0], Some(&categorias_ids[4]), None, None, None, None),
        // Gastos
        ("Compra de cables sintetizados y termomagnéticas", 3_400_000_000_i64, 10_000_i64, &egreso_sistema, Some(&categorias_ids[0]), None, Some(&trabajos_ids[0]), None, None),
        ("Adquisición de pinza amperimétrica True RMS", 1_250_000_000_i64, 10_000_i64, &egreso_sistema, Some(&categorias_ids[1]), None, None, None, None),
        ("Combustible y peajes traslados a Tigre", 450_000_000_i64, 10_000_i64, &egreso_sistema, Some(&categorias_ids[5]), None, Some(&trabajos_ids[2]), None, None),
        ("Pago de Monotributo / IIBB mensual", 620_000_000_i64, 10_000_i64, &egreso_sistema, Some(&categorias_ids[3]), None, None, None, None),
        // Adelantos a empleados
        ("Adelanto quincenal Ricardo Darín", 500_000_000_i64, 10_000_i64, &adelanto_sistema, None, None, None, Some(&empleados_ids[0]), None),
        ("Adelanto quincenal Natalia Oreiro", 400_000_000_i64, 10_000_i64, &adelanto_sistema, None, None, None, Some(&empleados_ids[2]), None),
    ];

    let mut movimientos_count = 0;
    for (conc, monto, cant, tipo_id, cat_id, cli_id, trab_id, emp_id, fact_id) in movimientos_data {
        let mov = movimiento::ActiveModel {
            id: Set(Uuid::now_v7().to_string()),
            fecha: Set("2025-02-18T14:30:00Z".to_string()),
            concepto: Set(conc.to_string()),
            monto: Set(monto),
            cantidad: Set(cant),
            tipo_movimiento_id: Set(tipo_id.to_string()),
            moneda: Set(0), // ARS
            cotizacion_aplicada: Set(None),
            tipo_concepto_pago_id: Set(None),
            categoria_id: Set(cat_id.cloned()),
            cliente_id: Set(cli_id.cloned()),
            trabajo_id: Set(trab_id.cloned()),
            empleado_id: Set(emp_id.cloned()),
            factura_id: Set(fact_id.cloned()),
            created_at: Set(now.clone()),
            updated_at: Set(None),
            row_version: Set(Uuid::now_v7().as_bytes().to_vec()),
            is_deleted: Set(false),
            deleted_at: Set(None),
        };
        mov.insert(&tx).await.map_err(AppError::persistence)?;
        movimientos_count += 1;
    }

    // 10. Liquidaciones
    let mut liquidaciones_count = 0;
    for (i, emp_id) in empleados_ids.iter().enumerate().take(2) {
        let liq = liquidacion::ActiveModel {
            id: Set(Uuid::now_v7().to_string()),
            empleado_id: Set(emp_id.clone()),
            fecha_inicio: Set("2025-02-01".to_string()),
            fecha_fin: Set("2025-02-15".to_string()),
            dias_trabajados: Set(110_000), // 11 días
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
            created_at: Set(now.clone()),
            updated_at: Set(None),
            row_version: Set(Uuid::now_v7().as_bytes().to_vec()),
            is_deleted: Set(false),
            deleted_at: Set(None),
        };
        liq.insert(&tx).await.map_err(AppError::persistence)?;
        liquidaciones_count += 1;
    }

    tx.commit().await.map_err(AppError::persistence)?;

    Ok(SeedSummary {
        categorias: categorias_ids.len(),
        tipos_movimiento: tipos_ids.len(),
        empleados: empleados_ids.len(),
        clientes: clientes_ids.len(),
        obras: obras_ids.len(),
        trabajos: trabajos_ids.len(),
        ordenes_trabajo: ordenes_ids.len(),
        movimientos: movimientos_count,
        facturas: facturas_ids.len(),
        liquidaciones: liquidaciones_count,
    })
}
