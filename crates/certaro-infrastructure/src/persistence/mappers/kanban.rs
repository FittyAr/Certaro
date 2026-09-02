use certaro_application::AppError;
use certaro_domain::entities::kanban::{
    KanbanColumna, KanbanEtiqueta, KanbanTablero, KanbanTarjeta, KanbanTarjetaChecklist,
    PrioridadTarjeta, TipoPresetTablero,
};
use certaro_domain::time;
use sea_orm::ActiveValue::Set;

use crate::persistence::mappers;
use crate::persistence::models::{
    kanban_columna, kanban_etiqueta, kanban_tablero, kanban_tarjeta, kanban_tarjeta_checklist,
};

// --- Tablero ---

pub fn tablero_to_domain(model: kanban_tablero::Model) -> Result<KanbanTablero, AppError> {
    Ok(KanbanTablero {
        id: mappers::uuid(&model.id)?,
        nombre: model.nombre,
        descripcion: model.descripcion,
        color: model.color,
        es_preset: model.es_preset,
        tipo_preset: model.tipo_preset.as_deref().and_then(TipoPresetTablero::from_str),
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

pub fn tablero_to_active(entity: &KanbanTablero) -> kanban_tablero::ActiveModel {
    kanban_tablero::ActiveModel {
        id: Set(entity.id.to_string()),
        nombre: Set(entity.nombre.clone()),
        descripcion: Set(entity.descripcion.clone()),
        color: Set(entity.color.clone()),
        es_preset: Set(entity.es_preset),
        tipo_preset: Set(entity.tipo_preset.map(|tp| tp.as_str().to_string())),
        activo: Set(entity.activo),
        created_at: Set(time::to_storage(entity.audit.created_at)),
        updated_at: Set(entity.audit.updated_at.map(time::to_storage)),
        row_version: Set(entity.audit.row_version.as_bytes().to_vec()),
        is_deleted: Set(entity.audit.is_deleted),
        deleted_at: Set(entity.audit.deleted_at.map(time::to_storage)),
    }
}

// --- Columna ---

pub fn columna_to_domain(model: kanban_columna::Model) -> Result<KanbanColumna, AppError> {
    Ok(KanbanColumna {
        id: mappers::uuid(&model.id)?,
        tablero_id: mappers::uuid(&model.tablero_id)?,
        nombre: model.nombre,
        color: model.color,
        orden: model.orden,
        limite_wip: model.limite_wip,
        estado_mapeado: model.estado_mapeado,
        audit: mappers::audit(
            &model.created_at,
            model.updated_at.as_deref(),
            &model.row_version,
            model.is_deleted,
            model.deleted_at.as_deref(),
        )?,
    })
}

pub fn columna_to_active(entity: &KanbanColumna) -> kanban_columna::ActiveModel {
    kanban_columna::ActiveModel {
        id: Set(entity.id.to_string()),
        tablero_id: Set(entity.tablero_id.to_string()),
        nombre: Set(entity.nombre.clone()),
        color: Set(entity.color.clone()),
        orden: Set(entity.orden),
        limite_wip: Set(entity.limite_wip),
        estado_mapeado: Set(entity.estado_mapeado),
        created_at: Set(time::to_storage(entity.audit.created_at)),
        updated_at: Set(entity.audit.updated_at.map(time::to_storage)),
        row_version: Set(entity.audit.row_version.as_bytes().to_vec()),
        is_deleted: Set(entity.audit.is_deleted),
        deleted_at: Set(entity.audit.deleted_at.map(time::to_storage)),
    }
}

// --- Tarjeta ---

pub fn tarjeta_to_domain(model: kanban_tarjeta::Model) -> Result<KanbanTarjeta, AppError> {
    let prioridad = PrioridadTarjeta::from_i32(model.prioridad)
        .unwrap_or(PrioridadTarjeta::Normal);

    let fecha_vencimiento = mappers::civil_opt(model.fecha_vencimiento.as_deref())?;

    Ok(KanbanTarjeta {
        id: mappers::uuid(&model.id)?,
        columna_id: mappers::uuid(&model.columna_id)?,
        titulo: model.titulo,
        descripcion: model.descripcion,
        prioridad,
        fecha_vencimiento,
        orden: model.orden,
        trabajo_id: mappers::uuid_opt(model.trabajo_id.as_deref())?,
        orden_trabajo_id: mappers::uuid_opt(model.orden_trabajo_id.as_deref())?,
        archivada: model.archivada,
        audit: mappers::audit(
            &model.created_at,
            model.updated_at.as_deref(),
            &model.row_version,
            model.is_deleted,
            model.deleted_at.as_deref(),
        )?,
    })
}

pub fn tarjeta_to_active(entity: &KanbanTarjeta) -> kanban_tarjeta::ActiveModel {
    kanban_tarjeta::ActiveModel {
        id: Set(entity.id.to_string()),
        columna_id: Set(entity.columna_id.to_string()),
        titulo: Set(entity.titulo.clone()),
        descripcion: Set(entity.descripcion.clone()),
        prioridad: Set(entity.prioridad.as_i32()),
        fecha_vencimiento: Set(entity.fecha_vencimiento.map(|d| d.format("%Y-%m-%d").to_string())),
        orden: Set(entity.orden),
        trabajo_id: Set(entity.trabajo_id.map(|id| id.to_string())),
        orden_trabajo_id: Set(entity.orden_trabajo_id.map(|id| id.to_string())),
        archivada: Set(entity.archivada),
        created_at: Set(time::to_storage(entity.audit.created_at)),
        updated_at: Set(entity.audit.updated_at.map(time::to_storage)),
        row_version: Set(entity.audit.row_version.as_bytes().to_vec()),
        is_deleted: Set(entity.audit.is_deleted),
        deleted_at: Set(entity.audit.deleted_at.map(time::to_storage)),
    }
}

// --- Etiqueta ---

pub fn etiqueta_to_domain(model: kanban_etiqueta::Model) -> Result<KanbanEtiqueta, AppError> {
    Ok(KanbanEtiqueta {
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

pub fn etiqueta_to_active(entity: &KanbanEtiqueta) -> kanban_etiqueta::ActiveModel {
    kanban_etiqueta::ActiveModel {
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

// --- Checklist ---

pub fn checklist_to_domain(
    model: kanban_tarjeta_checklist::Model,
) -> Result<KanbanTarjetaChecklist, AppError> {
    Ok(KanbanTarjetaChecklist {
        id: mappers::uuid(&model.id)?,
        tarjeta_id: mappers::uuid(&model.tarjeta_id)?,
        titulo: model.titulo,
        completada: model.completada,
        orden: model.orden,
        audit: mappers::audit(
            &model.created_at,
            model.updated_at.as_deref(),
            &model.row_version,
            model.is_deleted,
            model.deleted_at.as_deref(),
        )?,
    })
}

pub fn checklist_to_active(
    entity: &KanbanTarjetaChecklist,
) -> kanban_tarjeta_checklist::ActiveModel {
    kanban_tarjeta_checklist::ActiveModel {
        id: Set(entity.id.to_string()),
        tarjeta_id: Set(entity.tarjeta_id.to_string()),
        titulo: Set(entity.titulo.clone()),
        completada: Set(entity.completada),
        orden: Set(entity.orden),
        created_at: Set(time::to_storage(entity.audit.created_at)),
        updated_at: Set(entity.audit.updated_at.map(time::to_storage)),
        row_version: Set(entity.audit.row_version.as_bytes().to_vec()),
        is_deleted: Set(entity.audit.is_deleted),
        deleted_at: Set(entity.audit.deleted_at.map(time::to_storage)),
    }
}
