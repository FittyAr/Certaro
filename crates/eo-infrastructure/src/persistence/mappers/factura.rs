use eo_application::AppError;
use eo_domain::entities::{Factura, PagoFactura};
use eo_domain::{time, EstadoFactura, Money};
use sea_orm::ActiveValue::Set;

use crate::persistence::mappers;
use crate::persistence::models::factura::{ActiveModel, Model};
use crate::persistence::models::pago_factura;

pub fn to_domain(model: Model) -> Result<Factura, AppError> {
    Ok(Factura {
        id: mappers::uuid(&model.id)?,
        numero: model.numero,
        fecha: mappers::civil(&model.fecha)?,
        fecha_vencimiento: mappers::civil_opt(model.fecha_vencimiento.as_deref())?,
        cliente_id: mappers::uuid(&model.cliente_id)?,
        estado: EstadoFactura::from_i32(model.estado).map_err(|e| {
            AppError::persistence(anyhow::anyhow!("invalid estado {}: {e}", model.estado))
        })?,
        subtotal: Money::from_raw(model.subtotal),
        iva: Money::from_raw(model.iva),
        total: Money::from_raw(model.total),
        observaciones: model.observaciones,
        // Filled in by the repository when the whole aggregate was asked for.
        pagos: Vec::new(),
        audit: mappers::audit(
            &model.created_at,
            model.updated_at.as_deref(),
            &model.row_version,
            model.is_deleted,
            model.deleted_at.as_deref(),
        )?,
    })
}

pub fn to_active(entity: &Factura) -> ActiveModel {
    ActiveModel {
        id: Set(entity.id.to_string()),
        numero: Set(entity.numero.clone()),
        fecha: Set(mappers::civil_to_storage(entity.fecha)),
        fecha_vencimiento: Set(entity.fecha_vencimiento.map(mappers::civil_to_storage)),
        cliente_id: Set(entity.cliente_id.to_string()),
        estado: Set(entity.estado.as_i32()),
        subtotal: Set(entity.subtotal.raw()),
        iva: Set(entity.iva.raw()),
        total: Set(entity.total.raw()),
        observaciones: Set(entity.observaciones.clone()),
        created_at: Set(time::to_storage(entity.audit.created_at)),
        updated_at: Set(entity.audit.updated_at.map(time::to_storage)),
        row_version: Set(entity.audit.row_version.as_bytes().to_vec()),
        is_deleted: Set(entity.audit.is_deleted),
        deleted_at: Set(entity.audit.deleted_at.map(time::to_storage)),
    }
}

pub fn pago_to_domain(model: pago_factura::Model) -> Result<PagoFactura, AppError> {
    Ok(PagoFactura {
        id: mappers::uuid(&model.id)?,
        factura_id: mappers::uuid(&model.factura_id)?,
        fecha: mappers::civil(&model.fecha)?,
        monto: Money::from_raw(model.monto),
        medio_pago: model.medio_pago,
        audit: mappers::audit(
            &model.created_at,
            model.updated_at.as_deref(),
            &model.row_version,
            model.is_deleted,
            model.deleted_at.as_deref(),
        )?,
    })
}

pub fn pago_to_active(entity: &PagoFactura) -> pago_factura::ActiveModel {
    pago_factura::ActiveModel {
        id: Set(entity.id.to_string()),
        factura_id: Set(entity.factura_id.to_string()),
        fecha: Set(mappers::civil_to_storage(entity.fecha)),
        monto: Set(entity.monto.raw()),
        medio_pago: Set(entity.medio_pago.clone()),
        created_at: Set(time::to_storage(entity.audit.created_at)),
        updated_at: Set(entity.audit.updated_at.map(time::to_storage)),
        row_version: Set(entity.audit.row_version.as_bytes().to_vec()),
        is_deleted: Set(entity.audit.is_deleted),
        deleted_at: Set(entity.audit.deleted_at.map(time::to_storage)),
    }
}
