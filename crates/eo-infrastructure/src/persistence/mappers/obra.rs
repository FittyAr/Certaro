use eo_application::AppError;
use eo_domain::entities::Obra;
use eo_domain::{time, EstadoObra};
use sea_orm::ActiveValue::Set;

use crate::persistence::mappers;
use crate::persistence::models::obra::{ActiveModel, Model};

pub fn to_domain(model: Model) -> Result<Obra, AppError> {
    Ok(Obra {
        id: mappers::uuid(&model.id)?,
        numero: model.numero,
        nombre: model.nombre,
        direccion: model.direccion,
        localidad: model.localidad,
        cliente_id: mappers::uuid(&model.cliente_id)?,
        estado: EstadoObra::from_i32(model.estado).map_err(|e| {
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

pub fn to_active(entity: &Obra) -> ActiveModel {
    ActiveModel {
        id: Set(entity.id.to_string()),
        numero: Set(entity.numero),
        nombre: Set(entity.nombre.clone()),
        direccion: Set(entity.direccion.clone()),
        localidad: Set(entity.localidad.clone()),
        cliente_id: Set(entity.cliente_id.to_string()),
        estado: Set(entity.estado.as_i32()),
        created_at: Set(time::to_storage(entity.audit.created_at)),
        updated_at: Set(entity.audit.updated_at.map(time::to_storage)),
        row_version: Set(entity.audit.row_version.as_bytes().to_vec()),
        is_deleted: Set(entity.audit.is_deleted),
        deleted_at: Set(entity.audit.deleted_at.map(time::to_storage)),
    }
}
