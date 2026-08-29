use eo_application::AppError;
use eo_domain::entities::Categoria;
use eo_domain::time;
use sea_orm::ActiveValue::Set;

use crate::persistence::mappers;
use crate::persistence::models::categoria::{ActiveModel, Model};

pub fn to_domain(model: Model) -> Result<Categoria, AppError> {
    Ok(Categoria {
        id: mappers::uuid(&model.id)?,
        nombre: model.nombre,
        descripcion: model.descripcion,
        color_hex: model.color_hex,
        icono: model.icono,
        categoria_padre_id: mappers::uuid_opt(model.categoria_padre_id.as_deref())?,
        audit: mappers::audit(
            &model.created_at,
            model.updated_at.as_deref(),
            &model.row_version,
            model.is_deleted,
            model.deleted_at.as_deref(),
        )?,
    })
}

pub fn to_active(entity: &Categoria) -> ActiveModel {
    ActiveModel {
        id: Set(entity.id.to_string()),
        nombre: Set(entity.nombre.clone()),
        descripcion: Set(entity.descripcion.clone()),
        color_hex: Set(entity.color_hex.clone()),
        icono: Set(entity.icono.clone()),
        categoria_padre_id: Set(entity.categoria_padre_id.map(|id| id.to_string())),
        created_at: Set(time::to_storage(entity.audit.created_at)),
        updated_at: Set(entity.audit.updated_at.map(time::to_storage)),
        row_version: Set(entity.audit.row_version.as_bytes().to_vec()),
        is_deleted: Set(entity.audit.is_deleted),
        deleted_at: Set(entity.audit.deleted_at.map(time::to_storage)),
    }
}
