use certaro_application::AppError;
use certaro_domain::entities::{Certificado, CertificadoItem};
use certaro_domain::{time, Decimal4, Money};
use sea_orm::ActiveValue::Set;

use crate::persistence::mappers;
use crate::persistence::models::certificado::{ActiveModel, Model};
use crate::persistence::models::certificado_item::{
    ActiveModel as ItemActiveModel, Model as ItemModel,
};

pub fn to_domain(model: Model) -> Result<Certificado, AppError> {
    Ok(Certificado {
        id: mappers::uuid(&model.id)?,
        orden_trabajo_id: mappers::uuid(&model.orden_trabajo_id)?,
        numero: model.numero,
        fecha: mappers::civil(&model.fecha)?,
        observaciones: model.observaciones,
        total_certificado: Money::from_raw(model.total_certificado),
        ajuste_uocra: Money::from_raw(model.ajuste_uocra),
        otros_descuentos: Money::from_raw(model.otros_descuentos),
        total_neto: Money::from_raw(model.total_neto),
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

pub fn to_active(entity: &Certificado) -> ActiveModel {
    ActiveModel {
        id: Set(entity.id.to_string()),
        orden_trabajo_id: Set(entity.orden_trabajo_id.to_string()),
        numero: Set(entity.numero),
        fecha: Set(mappers::civil_to_storage(entity.fecha)),
        observaciones: Set(entity.observaciones.clone()),
        total_certificado: Set(entity.total_certificado.raw()),
        ajuste_uocra: Set(entity.ajuste_uocra.raw()),
        otros_descuentos: Set(entity.otros_descuentos.raw()),
        total_neto: Set(entity.total_neto.raw()),
        created_at: Set(time::to_storage(entity.audit.created_at)),
        updated_at: Set(entity.audit.updated_at.map(time::to_storage)),
        row_version: Set(entity.audit.row_version.as_bytes().to_vec()),
        is_deleted: Set(entity.audit.is_deleted),
        deleted_at: Set(entity.audit.deleted_at.map(time::to_storage)),
    }
}

pub fn item_to_domain(model: ItemModel) -> Result<CertificadoItem, AppError> {
    Ok(CertificadoItem {
        id: mappers::uuid(&model.id)?,
        certificado_id: mappers::uuid(&model.certificado_id)?,
        orden_trabajo_item_id: mappers::uuid(&model.orden_trabajo_item_id)?,
        cantidad: Decimal4::from_raw(model.cantidad),
        precio_unitario: Money::from_raw(model.precio_unitario),
        porcentaje_anterior: Decimal4::from_raw(model.porcentaje_anterior),
        porcentaje_actual: Decimal4::from_raw(model.porcentaje_actual),
        subtotal_actual: Money::from_raw(model.subtotal_actual),
        subtotal_acumulado: Money::from_raw(model.subtotal_acumulado),
        audit: mappers::audit(
            &model.created_at,
            model.updated_at.as_deref(),
            &model.row_version,
            model.is_deleted,
            model.deleted_at.as_deref(),
        )?,
    })
}

pub fn item_to_active(entity: &CertificadoItem) -> ItemActiveModel {
    ItemActiveModel {
        id: Set(entity.id.to_string()),
        certificado_id: Set(entity.certificado_id.to_string()),
        orden_trabajo_item_id: Set(entity.orden_trabajo_item_id.to_string()),
        cantidad: Set(entity.cantidad.raw()),
        precio_unitario: Set(entity.precio_unitario.raw()),
        porcentaje_anterior: Set(entity.porcentaje_anterior.raw()),
        porcentaje_actual: Set(entity.porcentaje_actual.raw()),
        subtotal_actual: Set(entity.subtotal_actual.raw()),
        subtotal_acumulado: Set(entity.subtotal_acumulado.raw()),
        created_at: Set(time::to_storage(entity.audit.created_at)),
        updated_at: Set(entity.audit.updated_at.map(time::to_storage)),
        row_version: Set(entity.audit.row_version.as_bytes().to_vec()),
        is_deleted: Set(entity.audit.is_deleted),
        deleted_at: Set(entity.audit.deleted_at.map(time::to_storage)),
    }
}
