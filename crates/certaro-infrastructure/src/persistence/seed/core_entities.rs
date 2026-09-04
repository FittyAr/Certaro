//! Seeding for Catalogs, Employees and Clients.

use certaro_application::result::AppResult;
use certaro_application::AppError;
use certaro_domain::RowVersion;
use sea_orm::{ActiveModelTrait, DatabaseTransaction, Set};
use uuid::Uuid;

use crate::persistence::models::{
    categoria, cliente, cliente_contacto, empleado, tipo_movimiento,
};
use super::data::{CATEGORIAS_DATA, CLIENTES_DATA, CUSTOM_TIPOS, EMPLEADOS_DATA};

pub async fn seed_catalogs_and_people(
    tx: &DatabaseTransaction,
    now: &str,
) -> AppResult<(Vec<String>, Vec<String>, Vec<String>, Vec<String>, usize)> {
    // 1. Categorías
    let mut categorias_ids = Vec::new();
    for (nombre, color, icono, padre_idx) in CATEGORIAS_DATA {
        let id = Uuid::now_v7().to_string();
        let cat = categoria::ActiveModel {
            id: Set(id.clone()),
            nombre: Set(nombre.to_string()),
            descripcion: Set(Some(format!("Insumos y gastos de {nombre}"))),
            color_hex: Set(Some(color.to_string())),
            icono: Set(Some(icono.to_string())),
            categoria_padre_id: Set(padre_idx.and_then(|idx| categorias_ids.get(idx)).cloned()),
            created_at: Set(now.to_string()),
            updated_at: Set(None),
            row_version: Set(RowVersion::INITIAL.as_bytes().to_vec()),
            is_deleted: Set(false),
            deleted_at: Set(None),
        };
        cat.insert(tx).await.map_err(AppError::persistence)?;
        categorias_ids.push(id);
    }

    // 2. Tipos de Movimiento personalizados
    let mut tipos_ids = Vec::new();
    for (nombre, es_ingreso) in CUSTOM_TIPOS {
        let id = Uuid::now_v7().to_string();
        let tipo = tipo_movimiento::ActiveModel {
            id: Set(id.clone()),
            nombre: Set(nombre.to_string()),
            descripcion: Set(None),
            es_ingreso: Set(es_ingreso),
            es_sistema: Set(false),
            created_at: Set(now.to_string()),
            updated_at: Set(None),
            row_version: Set(RowVersion::INITIAL.as_bytes().to_vec()),
            is_deleted: Set(false),
            deleted_at: Set(None),
        };
        tipo.insert(tx).await.map_err(AppError::persistence)?;
        tipos_ids.push(id);
    }

    // 3. Empleados
    let mut empleados_ids = Vec::new();
    for (nombre, dni, cargo, sueldo, tarifa, tel, mail) in EMPLEADOS_DATA {
        let id = Uuid::now_v7().to_string();
        let emp = empleado::ActiveModel {
            id: Set(id.clone()),
            nombre: Set(nombre.to_string()),
            dni: Set(Some(dni.to_string())),
            cargo: Set(Some(cargo.to_string())),
            sueldo_base: Set(sueldo),
            pago_frecuencia: Set(1),
            tarifa_diaria: Set(tarifa),
            multiplicador_sabado: Set(15_000),
            multiplicador_domingo: Set(20_000),
            multiplicador_feriado: Set(20_000),
            email: Set(Some(mail.to_string())),
            telefono: Set(Some(tel.to_string())),
            fecha_ingreso: Set("2025-01-15".to_string()),
            fecha_egreso: Set(None),
            activo: Set(true),
            created_at: Set(now.to_string()),
            updated_at: Set(None),
            row_version: Set(RowVersion::INITIAL.as_bytes().to_vec()),
            is_deleted: Set(false),
            deleted_at: Set(None),
        };
        emp.insert(tx).await.map_err(AppError::persistence)?;
        empleados_ids.push(id);
    }

    // 4. Clientes & Contactos
    let mut clientes_ids = Vec::new();
    let mut contactos_count = 0;
    for (nombre, cuit, dir, tel, mail, iva) in CLIENTES_DATA {
        let id = Uuid::now_v7().to_string();
        let cli = cliente::ActiveModel {
            id: Set(id.clone()),
            nombre: Set(nombre.to_string()),
            cuit: Set(Some(cuit.to_string())),
            direccion: Set(Some(dir.to_string())),
            telefono: Set(Some(tel.to_string())),
            email: Set(Some(mail.to_string())),
            condicion_iva: Set(Some(iva.to_string())),
            created_at: Set(now.to_string()),
            updated_at: Set(None),
            row_version: Set(RowVersion::INITIAL.as_bytes().to_vec()),
            is_deleted: Set(false),
            deleted_at: Set(None),
        };
        cli.insert(tx).await.map_err(AppError::persistence)?;

        let contacto = cliente_contacto::ActiveModel {
            id: Set(Uuid::now_v7().to_string()),
            cliente_id: Set(id.clone()),
            nombre: Set(Some(format!("Contacto {}", nombre.split_whitespace().next().unwrap_or("Principal")))),
            email: Set(mail.to_string()),
            telefono: Set(Some(tel.to_string())),
            etiqueta: Set("Administración / Pagos".to_string()),
            es_principal: Set(true),
            created_at: Set(now.to_string()),
            updated_at: Set(None),
            row_version: Set(RowVersion::INITIAL.as_bytes().to_vec()),
            is_deleted: Set(false),
            deleted_at: Set(None),
        };
        contacto.insert(tx).await.map_err(AppError::persistence)?;
        contactos_count += 1;

        clientes_ids.push(id);
    }

    Ok((categorias_ids, tipos_ids, empleados_ids, clientes_ids, contactos_count))
}
