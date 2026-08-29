use eo_application::AppError;
use eo_domain::entities::TipoMovimiento;
use eo_domain::time;
use sea_orm::ActiveValue::Set;

use crate::persistence::mappers;
use crate::persistence::models::tipo_movimiento::{ActiveModel, Model};

pub fn to_domain(model: Model) -> Result<TipoMovimiento, AppError> {
    Ok(TipoMovimiento {
        id: mappers::uuid(&model.id)?,
        nombre: model.nombre,
        descripcion: model.descripcion,
        es_ingreso: model.es_ingreso,
        es_sistema: model.es_sistema,
        audit: mappers::audit(
            &model.created_at,
            model.updated_at.as_deref(),
            &model.row_version,
            model.is_deleted,
            model.deleted_at.as_deref(),
        )?,
    })
}

pub fn to_active(entity: &TipoMovimiento) -> ActiveModel {
    ActiveModel {
        id: Set(entity.id.to_string()),
        nombre: Set(entity.nombre.clone()),
        descripcion: Set(entity.descripcion.clone()),
        es_ingreso: Set(entity.es_ingreso),
        es_sistema: Set(entity.es_sistema),
        created_at: Set(time::to_storage(entity.audit.created_at)),
        updated_at: Set(entity.audit.updated_at.map(time::to_storage)),
        row_version: Set(entity.audit.row_version.as_bytes().to_vec()),
        is_deleted: Set(entity.audit.is_deleted),
        deleted_at: Set(entity.audit.deleted_at.map(time::to_storage)),
    }
}
