use certaro_application::AppError;
use certaro_domain::entities::{Adjunto, EntidadAdjunto};
use certaro_domain::time;
use sea_orm::ActiveValue::Set;

use crate::persistence::mappers;
use crate::persistence::models::adjunto::{ActiveModel, Model};

pub fn to_domain(model: Model) -> Result<Adjunto, AppError> {
    Ok(Adjunto {
        id: mappers::uuid(&model.id)?,
        entidad_tipo: EntidadAdjunto::parse(&model.entidad_tipo).map_err(AppError::from)?,
        entidad_id: mappers::uuid(&model.entidad_id)?,
        nombre_archivo: model.nombre_archivo,
        ruta_relativa: model.ruta_relativa,
        mime: model.mime,
        // The column is signed because SQLite integers are; a negative size is corruption, and
        // clamping is better than a panic on a value nobody can act on.
        tamano: model.tamano.max(0) as u64,
        audit: mappers::audit(
            &model.created_at,
            model.updated_at.as_deref(),
            &model.row_version,
            model.is_deleted,
            model.deleted_at.as_deref(),
        )?,
    })
}

pub fn to_active(entity: &Adjunto) -> ActiveModel {
    ActiveModel {
        id: Set(entity.id.to_string()),
        entidad_tipo: Set(entity.entidad_tipo.as_str().to_owned()),
        entidad_id: Set(entity.entidad_id.to_string()),
        nombre_archivo: Set(entity.nombre_archivo.clone()),
        ruta_relativa: Set(entity.ruta_relativa.clone()),
        mime: Set(entity.mime.clone()),
        tamano: Set(entity.tamano as i64),
        created_at: Set(time::to_storage(entity.audit.created_at)),
        updated_at: Set(entity.audit.updated_at.map(time::to_storage)),
        row_version: Set(entity.audit.row_version.as_bytes().to_vec()),
        is_deleted: Set(entity.audit.is_deleted),
        deleted_at: Set(entity.audit.deleted_at.map(time::to_storage)),
    }
}
