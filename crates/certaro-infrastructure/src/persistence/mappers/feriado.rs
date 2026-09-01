use certaro_application::AppError;
use certaro_domain::entities::{Feriado, OrigenFeriado};
use certaro_domain::time;
use sea_orm::ActiveValue::Set;

use crate::persistence::mappers;
use crate::persistence::models::feriado::{ActiveModel, Model};

pub fn to_domain(model: Model) -> Result<Feriado, AppError> {
    Ok(Feriado {
        fecha: mappers::civil(&model.fecha)?,
        nombre: model.nombre,
        tipo: model.tipo,
        origen: OrigenFeriado::parse(&model.origen).map_err(AppError::from)?,
        created_at: mappers::instant(&model.created_at)?,
        updated_at: mappers::instant_opt(model.updated_at.as_deref())?,
    })
}

pub fn to_active(entity: &Feriado) -> ActiveModel {
    ActiveModel {
        fecha: Set(mappers::civil_to_storage(entity.fecha)),
        nombre: Set(entity.nombre.clone()),
        tipo: Set(entity.tipo.clone()),
        origen: Set(entity.origen.as_str().to_owned()),
        created_at: Set(time::to_storage(entity.created_at)),
        updated_at: Set(entity.updated_at.map(time::to_storage)),
    }
}
