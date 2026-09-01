use certaro_application::AppError;
use certaro_domain::entities::{Cliente, ClienteContacto};
use certaro_domain::time;
use sea_orm::ActiveValue::Set;

use crate::persistence::mappers;
use crate::persistence::models::cliente::{ActiveModel, Model};
use crate::persistence::models::cliente_contacto;

pub fn to_domain(model: Model) -> Result<Cliente, AppError> {
    Ok(Cliente {
        id: mappers::uuid(&model.id)?,
        nombre: model.nombre,
        cuit: model.cuit,
        direccion: model.direccion,
        telefono: model.telefono,
        email: model.email,
        condicion_iva: model.condicion_iva,
        // The repository fills this in when the caller asked for the whole aggregate; a list
        // query leaves it empty rather than issuing one extra statement per row.
        contactos: Vec::new(),
        audit: mappers::audit(
            &model.created_at,
            model.updated_at.as_deref(),
            &model.row_version,
            model.is_deleted,
            model.deleted_at.as_deref(),
        )?,
    })
}

pub fn to_active(entity: &Cliente) -> ActiveModel {
    ActiveModel {
        id: Set(entity.id.to_string()),
        nombre: Set(entity.nombre.clone()),
        cuit: Set(entity.cuit.clone()),
        direccion: Set(entity.direccion.clone()),
        telefono: Set(entity.telefono.clone()),
        email: Set(entity.email.clone()),
        condicion_iva: Set(entity.condicion_iva.clone()),
        created_at: Set(time::to_storage(entity.audit.created_at)),
        updated_at: Set(entity.audit.updated_at.map(time::to_storage)),
        row_version: Set(entity.audit.row_version.as_bytes().to_vec()),
        is_deleted: Set(entity.audit.is_deleted),
        deleted_at: Set(entity.audit.deleted_at.map(time::to_storage)),
    }
}

pub fn contacto_to_domain(model: cliente_contacto::Model) -> Result<ClienteContacto, AppError> {
    Ok(ClienteContacto {
        id: mappers::uuid(&model.id)?,
        cliente_id: mappers::uuid(&model.cliente_id)?,
        etiqueta: model.etiqueta,
        email: model.email,
        nombre: model.nombre,
        telefono: model.telefono,
        es_principal: model.es_principal,
        audit: mappers::audit(
            &model.created_at,
            model.updated_at.as_deref(),
            &model.row_version,
            model.is_deleted,
            model.deleted_at.as_deref(),
        )?,
    })
}

pub fn contacto_to_active(entity: &ClienteContacto) -> cliente_contacto::ActiveModel {
    cliente_contacto::ActiveModel {
        id: Set(entity.id.to_string()),
        cliente_id: Set(entity.cliente_id.to_string()),
        etiqueta: Set(entity.etiqueta.clone()),
        email: Set(entity.email.clone()),
        nombre: Set(entity.nombre.clone()),
        telefono: Set(entity.telefono.clone()),
        es_principal: Set(entity.es_principal),
        created_at: Set(time::to_storage(entity.audit.created_at)),
        updated_at: Set(entity.audit.updated_at.map(time::to_storage)),
        row_version: Set(entity.audit.row_version.as_bytes().to_vec()),
        is_deleted: Set(entity.audit.is_deleted),
        deleted_at: Set(entity.audit.deleted_at.map(time::to_storage)),
    }
}
