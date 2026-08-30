use eo_application::AppError;
use eo_domain::entities::AsistenciaEmpleado;
use eo_domain::{time, TipoJornada};
use sea_orm::ActiveValue::Set;

use crate::persistence::mappers;
use crate::persistence::models::asistencia_empleado::{ActiveModel, Model};

pub fn to_domain(model: Model) -> Result<AsistenciaEmpleado, AppError> {
    Ok(AsistenciaEmpleado {
        id: mappers::uuid(&model.id)?,
        empleado_id: mappers::uuid(&model.empleado_id)?,
        fecha: mappers::civil(&model.fecha)?,
        tipo_jornada: TipoJornada::from_i32(model.tipo_jornada).map_err(AppError::from)?,
        trabajo_id: mappers::uuid_opt(model.trabajo_id.as_deref())?,
        observaciones: model.observaciones,
        audit: mappers::audit(
            &model.created_at,
            model.updated_at.as_deref(),
            &model.row_version,
            model.is_deleted,
            model.deleted_at.as_deref(),
        )?,
    })
}

pub fn to_active(entity: &AsistenciaEmpleado) -> ActiveModel {
    ActiveModel {
        id: Set(entity.id.to_string()),
        empleado_id: Set(entity.empleado_id.to_string()),
        fecha: Set(mappers::civil_to_storage(entity.fecha)),
        tipo_jornada: Set(entity.tipo_jornada.as_i32()),
        trabajo_id: Set(entity.trabajo_id.map(|id| id.to_string())),
        observaciones: Set(entity.observaciones.clone()),
        created_at: Set(time::to_storage(entity.audit.created_at)),
        updated_at: Set(entity.audit.updated_at.map(time::to_storage)),
        row_version: Set(entity.audit.row_version.as_bytes().to_vec()),
        is_deleted: Set(entity.audit.is_deleted),
        deleted_at: Set(entity.audit.deleted_at.map(time::to_storage)),
    }
}
