use eo_application::AppError;
use eo_domain::entities::{OrdenTrabajo, OrdenTrabajoItem};
use eo_domain::{time, Decimal4, Money};
use sea_orm::ActiveValue::Set;

use crate::persistence::mappers;
use crate::persistence::models::orden_trabajo::{ActiveModel, Model};
use crate::persistence::models::orden_trabajo_item::{
    ActiveModel as ItemActiveModel, Model as ItemModel,
};

pub fn to_domain(model: Model) -> Result<OrdenTrabajo, AppError> {
    Ok(OrdenTrabajo {
        id: mappers::uuid(&model.id)?,
        trabajo_id: mappers::uuid(&model.trabajo_id)?,
        titulo: model.titulo,
        numero_certificado: model.numero_certificado,
        fecha: mappers::civil(&model.fecha)?,
        observaciones: model.observaciones,
        ajuste_uocra_porcentaje: Decimal4::from_raw(model.ajuste_uocra_porcentaje),
        otros_descuentos: Money::from_raw(model.otros_descuentos),
        items: Vec::new(),
        audit: mappers::audit(
            &model.created_at,
            model.updated_at.as_deref(),
            &model.row_version,
            model.is_deleted,
            model.deleted_at.as_deref(),
        )?,
    })
}

pub fn to_active(entity: &OrdenTrabajo) -> ActiveModel {
    ActiveModel {
        id: Set(entity.id.to_string()),
        trabajo_id: Set(entity.trabajo_id.to_string()),
        titulo: Set(entity.titulo.clone()),
        numero_certificado: Set(entity.numero_certificado.clone()),
        fecha: Set(mappers::civil_to_storage(entity.fecha)),
        observaciones: Set(entity.observaciones.clone()),
        ajuste_uocra_porcentaje: Set(entity.ajuste_uocra_porcentaje.raw()),
        otros_descuentos: Set(entity.otros_descuentos.raw()),
        created_at: Set(time::to_storage(entity.audit.created_at)),
        updated_at: Set(entity.audit.updated_at.map(time::to_storage)),
        row_version: Set(entity.audit.row_version.as_bytes().to_vec()),
        is_deleted: Set(entity.audit.is_deleted),
        deleted_at: Set(entity.audit.deleted_at.map(time::to_storage)),
    }
}

pub fn item_to_domain(model: ItemModel) -> Result<OrdenTrabajoItem, AppError> {
    Ok(OrdenTrabajoItem {
        id: mappers::uuid(&model.id)?,
        orden_trabajo_id: mappers::uuid(&model.orden_trabajo_id)?,
        descripcion: model.descripcion,
        unidad: model.unidad,
        cantidad: Decimal4::from_raw(model.cantidad),
        precio_unitario: Money::from_raw(model.precio_unitario),
        porcentaje_anterior: Decimal4::from_raw(model.porcentaje_anterior),
        porcentaje_actual: Decimal4::from_raw(model.porcentaje_actual),
        ejecutado: model.ejecutado,
        nota: model.nota,
        orden: model.orden,
        audit: mappers::audit(
            &model.created_at,
            model.updated_at.as_deref(),
            &model.row_version,
            model.is_deleted,
            model.deleted_at.as_deref(),
        )?,
    })
}

pub fn item_to_active(entity: &OrdenTrabajoItem) -> ItemActiveModel {
    ItemActiveModel {
        id: Set(entity.id.to_string()),
        orden_trabajo_id: Set(entity.orden_trabajo_id.to_string()),
        descripcion: Set(entity.descripcion.clone()),
        unidad: Set(entity.unidad.clone()),
        cantidad: Set(entity.cantidad.raw()),
        precio_unitario: Set(entity.precio_unitario.raw()),
        porcentaje_anterior: Set(entity.porcentaje_anterior.raw()),
        porcentaje_actual: Set(entity.porcentaje_actual.raw()),
        ejecutado: Set(entity.ejecutado),
        nota: Set(entity.nota.clone()),
        orden: Set(entity.orden),
        created_at: Set(time::to_storage(entity.audit.created_at)),
        updated_at: Set(entity.audit.updated_at.map(time::to_storage)),
        row_version: Set(entity.audit.row_version.as_bytes().to_vec()),
        is_deleted: Set(entity.audit.is_deleted),
        deleted_at: Set(entity.audit.deleted_at.map(time::to_storage)),
    }
}
