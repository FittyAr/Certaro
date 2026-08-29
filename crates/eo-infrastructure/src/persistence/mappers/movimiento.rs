use eo_application::AppError;
use eo_domain::entities::Movimiento;
use eo_domain::{time, Decimal4, Moneda, Money};
use sea_orm::ActiveValue::Set;

use crate::persistence::mappers;
use crate::persistence::models::movimiento::{ActiveModel, Model};

pub fn to_domain(model: Model) -> Result<Movimiento, AppError> {
    Ok(Movimiento {
        id: mappers::uuid(&model.id)?,
        fecha: mappers::instant(&model.fecha)?,
        concepto: model.concepto,
        monto: Money::from_raw(model.monto),
        cantidad: Decimal4::from_raw(model.cantidad),
        tipo_movimiento_id: mappers::uuid(&model.tipo_movimiento_id)?,
        // A currency outside the enum is corrupt data; mapping it to pesos would silently
        // reinterpret the amount.
        moneda: Moneda::from_i32(model.moneda).map_err(|e| {
            AppError::persistence(anyhow::anyhow!("invalid moneda {}: {e}", model.moneda))
        })?,
        cotizacion_aplicada: model.cotizacion_aplicada.map(Money::from_raw),
        tipo_concepto_pago_id: mappers::uuid_opt(model.tipo_concepto_pago_id.as_deref())?,
        categoria_id: mappers::uuid_opt(model.categoria_id.as_deref())?,
        cliente_id: mappers::uuid_opt(model.cliente_id.as_deref())?,
        trabajo_id: mappers::uuid_opt(model.trabajo_id.as_deref())?,
        empleado_id: mappers::uuid_opt(model.empleado_id.as_deref())?,
        factura_id: mappers::uuid_opt(model.factura_id.as_deref())?,
        audit: mappers::audit(
            &model.created_at,
            model.updated_at.as_deref(),
            &model.row_version,
            model.is_deleted,
            model.deleted_at.as_deref(),
        )?,
    })
}

pub fn to_active(entity: &Movimiento) -> ActiveModel {
    ActiveModel {
        id: Set(entity.id.to_string()),
        fecha: Set(time::to_storage(entity.fecha)),
        concepto: Set(entity.concepto.clone()),
        monto: Set(entity.monto.raw()),
        cantidad: Set(entity.cantidad.raw()),
        tipo_movimiento_id: Set(entity.tipo_movimiento_id.to_string()),
        moneda: Set(entity.moneda.as_i32()),
        cotizacion_aplicada: Set(entity.cotizacion_aplicada.map(Money::raw)),
        tipo_concepto_pago_id: Set(entity.tipo_concepto_pago_id.map(|id| id.to_string())),
        categoria_id: Set(entity.categoria_id.map(|id| id.to_string())),
        cliente_id: Set(entity.cliente_id.map(|id| id.to_string())),
        trabajo_id: Set(entity.trabajo_id.map(|id| id.to_string())),
        empleado_id: Set(entity.empleado_id.map(|id| id.to_string())),
        factura_id: Set(entity.factura_id.map(|id| id.to_string())),
        created_at: Set(time::to_storage(entity.audit.created_at)),
        updated_at: Set(entity.audit.updated_at.map(time::to_storage)),
        row_version: Set(entity.audit.row_version.as_bytes().to_vec()),
        is_deleted: Set(entity.audit.is_deleted),
        deleted_at: Set(entity.audit.deleted_at.map(time::to_storage)),
    }
}
