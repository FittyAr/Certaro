use certaro_application::AppError;
use certaro_domain::entities::calendario::{
    CalendarioEvento, CalendarioGrupoRecurso, CalendarioRecurso, TipoEvento, TipoRecurso,
};
use certaro_domain::time;
use sea_orm::ActiveValue::Set;

use crate::persistence::mappers;
use crate::persistence::models::{
    calendario_evento, calendario_grupo_recurso, calendario_recurso,
};

// --- Grupo Recurso ---

pub fn grupo_recurso_to_domain(
    model: calendario_grupo_recurso::Model,
) -> Result<CalendarioGrupoRecurso, AppError> {
    Ok(CalendarioGrupoRecurso {
        id: mappers::uuid(&model.id)?,
        nombre: model.nombre,
        color: model.color,
        audit: mappers::audit(
            &model.created_at,
            model.updated_at.as_deref(),
            &model.row_version,
            model.is_deleted,
            model.deleted_at.as_deref(),
        )?,
    })
}

pub fn grupo_recurso_to_active(
    entity: &CalendarioGrupoRecurso,
) -> calendario_grupo_recurso::ActiveModel {
    calendario_grupo_recurso::ActiveModel {
        id: Set(entity.id.to_string()),
        nombre: Set(entity.nombre.clone()),
        color: Set(entity.color.clone()),
        created_at: Set(time::to_storage(entity.audit.created_at)),
        updated_at: Set(entity.audit.updated_at.map(time::to_storage)),
        row_version: Set(entity.audit.row_version.as_bytes().to_vec()),
        is_deleted: Set(entity.audit.is_deleted),
        deleted_at: Set(entity.audit.deleted_at.map(time::to_storage)),
    }
}

// --- Recurso ---

pub fn recurso_to_domain(model: calendario_recurso::Model) -> Result<CalendarioRecurso, AppError> {
    let tipo = TipoRecurso::parse(&model.tipo).map_err(AppError::from)?;

    Ok(CalendarioRecurso {
        id: mappers::uuid(&model.id)?,
        grupo_id: mappers::uuid_opt(model.grupo_id.as_deref())?,
        nombre: model.nombre,
        tipo,
        empleado_id: mappers::uuid_opt(model.empleado_id.as_deref())?,
        color: model.color,
        activo: model.activo,
        audit: mappers::audit(
            &model.created_at,
            model.updated_at.as_deref(),
            &model.row_version,
            model.is_deleted,
            model.deleted_at.as_deref(),
        )?,
    })
}

pub fn recurso_to_active(entity: &CalendarioRecurso) -> calendario_recurso::ActiveModel {
    calendario_recurso::ActiveModel {
        id: Set(entity.id.to_string()),
        grupo_id: Set(entity.grupo_id.map(|id| id.to_string())),
        nombre: Set(entity.nombre.clone()),
        tipo: Set(entity.tipo.as_str().to_string()),
        empleado_id: Set(entity.empleado_id.map(|id| id.to_string())),
        color: Set(entity.color.clone()),
        activo: Set(entity.activo),
        created_at: Set(time::to_storage(entity.audit.created_at)),
        updated_at: Set(entity.audit.updated_at.map(time::to_storage)),
        row_version: Set(entity.audit.row_version.as_bytes().to_vec()),
        is_deleted: Set(entity.audit.is_deleted),
        deleted_at: Set(entity.audit.deleted_at.map(time::to_storage)),
    }
}

// --- Evento ---

pub fn evento_to_domain(model: calendario_evento::Model) -> Result<CalendarioEvento, AppError> {
    let tipo = TipoEvento::parse(&model.tipo).map_err(AppError::from)?;

    Ok(CalendarioEvento {
        id: mappers::uuid(&model.id)?,
        titulo: model.titulo,
        descripcion: model.descripcion,
        tipo,
        inicio: mappers::instant(&model.inicio)?,
        fin: mappers::instant(&model.fin)?,
        todo_el_dia: model.todo_el_dia,
        color: model.color,
        trabajo_id: mappers::uuid_opt(model.trabajo_id.as_deref())?,
        kanban_tarjeta_id: mappers::uuid_opt(model.kanban_tarjeta_id.as_deref())?,
        audit: mappers::audit(
            &model.created_at,
            model.updated_at.as_deref(),
            &model.row_version,
            model.is_deleted,
            model.deleted_at.as_deref(),
        )?,
    })
}

pub fn evento_to_active(entity: &CalendarioEvento) -> calendario_evento::ActiveModel {
    calendario_evento::ActiveModel {
        id: Set(entity.id.to_string()),
        titulo: Set(entity.titulo.clone()),
        descripcion: Set(entity.descripcion.clone()),
        tipo: Set(entity.tipo.as_str().to_string()),
        inicio: Set(time::to_storage(entity.inicio)),
        fin: Set(time::to_storage(entity.fin)),
        todo_el_dia: Set(entity.todo_el_dia),
        color: Set(entity.color.clone()),
        trabajo_id: Set(entity.trabajo_id.map(|id| id.to_string())),
        kanban_tarjeta_id: Set(entity.kanban_tarjeta_id.map(|id| id.to_string())),
        created_at: Set(time::to_storage(entity.audit.created_at)),
        updated_at: Set(entity.audit.updated_at.map(time::to_storage)),
        row_version: Set(entity.audit.row_version.as_bytes().to_vec()),
        is_deleted: Set(entity.audit.is_deleted),
        deleted_at: Set(entity.audit.deleted_at.map(time::to_storage)),
    }
}
