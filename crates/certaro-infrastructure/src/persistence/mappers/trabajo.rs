use certaro_application::AppError;
use certaro_domain::entities::Trabajo;
use certaro_domain::{time, EstadoTrabajo, Money};
use sea_orm::ActiveValue::Set;

use crate::persistence::mappers;
use crate::persistence::models::trabajo::{ActiveModel, Model};

pub fn to_domain(model: Model) -> Result<Trabajo, AppError> {
    Ok(Trabajo {
        id: mappers::uuid(&model.id)?,
        obra_id: mappers::uuid(&model.obra_id)?,
        descripcion: model.descripcion,
        fecha_inicio: mappers::civil(&model.fecha_inicio)?,
        fecha_fin: mappers::civil_opt(model.fecha_fin.as_deref())?,
        presupuesto: Money::from_raw(model.presupuesto),
        estado: EstadoTrabajo::from_i32(model.estado).map_err(|e| {
            AppError::persistence(anyhow::anyhow!("invalid estado {}: {e}", model.estado))
        })?,
        audit: mappers::audit(
            &model.created_at,
            model.updated_at.as_deref(),
            &model.row_version,
            model.is_deleted,
            model.deleted_at.as_deref(),
        )?,
    })
}

pub fn to_active(entity: &Trabajo) -> ActiveModel {
    ActiveModel {
        id: Set(entity.id.to_string()),
        obra_id: Set(entity.obra_id.to_string()),
        descripcion: Set(entity.descripcion.clone()),
        fecha_inicio: Set(mappers::civil_to_storage(entity.fecha_inicio)),
        fecha_fin: Set(entity.fecha_fin.map(mappers::civil_to_storage)),
        presupuesto: Set(entity.presupuesto.raw()),
        estado: Set(entity.estado.as_i32()),
        created_at: Set(time::to_storage(entity.audit.created_at)),
        updated_at: Set(entity.audit.updated_at.map(time::to_storage)),
        row_version: Set(entity.audit.row_version.as_bytes().to_vec()),
        is_deleted: Set(entity.audit.is_deleted),
        deleted_at: Set(entity.audit.deleted_at.map(time::to_storage)),
    }
}
