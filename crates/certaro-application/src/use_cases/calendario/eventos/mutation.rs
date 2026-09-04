use chrono::{DateTime, Utc};
use uuid::Uuid;

use certaro_domain::entities::audit::Audit;
use certaro_domain::entities::calendario::CalendarioEvento;
use certaro_domain::time;
use certaro_domain::RowVersion;

use crate::dtos::calendario::{
    ActualizarEventoInput, CalendarioEventoDto, CalendarioRecursoDto, CrearEventoInput,
};
use crate::error::{AppError, FieldError};
use crate::ports::clock::ClockPort;
use crate::ports::id_generator::IdGeneratorPort;
use crate::ports::repositories::UnitOfWork;
use crate::result::AppResult;

pub async fn create_evento_impl(
    uow: &dyn UnitOfWork,
    clock: &dyn ClockPort,
    id_gen: &dyn IdGeneratorPort,
    input: CrearEventoInput,
) -> AppResult<CalendarioEventoDto> {
    let titulo = input.titulo.trim();
    if titulo.is_empty() {
        return Err(AppError::Validation(vec![FieldError::new(
            "titulo",
            "Validation.Calendario.TituloRequerido",
        )]));
    }

    let inicio: DateTime<Utc> = time::from_storage(&input.inicio)
        .map_err(|e| AppError::Validation(vec![FieldError::new("inicio", e.to_string())]))?;
    let fin: DateTime<Utc> = time::from_storage(&input.fin)
        .map_err(|e| AppError::Validation(vec![FieldError::new("fin", e.to_string())]))?;

    if fin < inicio {
        return Err(AppError::Validation(vec![FieldError::new(
            "fin",
            "Validation.Calendario.FinMenorQueInicio",
        )]));
    }

    let id = id_gen.new_id();
    let now = clock.now_utc();
    let audit = Audit::new(now);

    let entity = CalendarioEvento {
        id,
        titulo: titulo.to_string(),
        descripcion: input.descripcion,
        tipo: input.tipo,
        inicio,
        fin,
        todo_el_dia: input.todo_el_dia,
        color: input.color,
        trabajo_id: input.trabajo_id,
        kanban_tarjeta_id: input.kanban_tarjeta_id,
        audit,
    };

    let tx = uow.begin().await?;
    tx.calendario_eventos().insert(&entity).await?;

    let mut recursos_dto = Vec::new();
    if let Some(rids) = input.recurso_ids {
        for rid in rids {
            tx.calendario_eventos().assign_recurso(entity.id, rid).await?;
            if let Some(r) = tx.calendario_recursos().find_by_id(rid).await? {
                recursos_dto.push(CalendarioRecursoDto {
                    id: r.id,
                    grupo_id: r.grupo_id,
                    grupo_nombre: None,
                    nombre: r.nombre,
                    tipo: r.tipo,
                    empleado_id: r.empleado_id,
                    color: r.color,
                    activo: r.activo,
                    row_version: r.audit.row_version,
                });
            }
        }
    }

    tx.commit().await?;

    Ok(CalendarioEventoDto {
        id: entity.id,
        titulo: entity.titulo,
        descripcion: entity.descripcion,
        tipo: entity.tipo,
        inicio: time::to_storage(entity.inicio),
        fin: time::to_storage(entity.fin),
        todo_el_dia: entity.todo_el_dia,
        color: entity.color,
        trabajo_id: entity.trabajo_id,
        kanban_tarjeta_id: entity.kanban_tarjeta_id,
        recursos: recursos_dto,
        es_virtual: false,
        row_version: entity.audit.row_version,
    })
}

pub async fn update_evento_impl(
    uow: &dyn UnitOfWork,
    clock: &dyn ClockPort,
    id: Uuid,
    input: ActualizarEventoInput,
) -> AppResult<CalendarioEventoDto> {
    let titulo = input.titulo.trim();
    if titulo.is_empty() {
        return Err(AppError::Validation(vec![FieldError::new(
            "titulo",
            "Validation.Calendario.TituloRequerido",
        )]));
    }

    let inicio: DateTime<Utc> = time::from_storage(&input.inicio)
        .map_err(|e| AppError::Validation(vec![FieldError::new("inicio", e.to_string())]))?;
    let fin: DateTime<Utc> = time::from_storage(&input.fin)
        .map_err(|e| AppError::Validation(vec![FieldError::new("fin", e.to_string())]))?;

    if fin < inicio {
        return Err(AppError::Validation(vec![FieldError::new(
            "fin",
            "Validation.Calendario.FinMenorQueInicio",
        )]));
    }

    let tx = uow.begin().await?;
    let mut entity = tx
        .calendario_eventos()
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound {
            entity: "CalendarioEvento",
            id: id.to_string(),
        })?;

    if entity.audit.row_version != input.row_version {
        return Err(AppError::Concurrency {
            entity: "calendario_eventos",
        });
    }

    let now = clock.now_utc();
    entity.titulo = titulo.to_string();
    entity.descripcion = input.descripcion;
    entity.tipo = input.tipo;
    entity.inicio = inicio;
    entity.fin = fin;
    entity.todo_el_dia = input.todo_el_dia;
    entity.color = input.color;
    entity.trabajo_id = input.trabajo_id;
    entity.audit.touch(now);

    tx.calendario_eventos().update(&entity).await?;

    if let Some(rids) = input.recurso_ids {
        tx.calendario_eventos().unassign_recursos(entity.id).await?;
        for rid in rids {
            tx.calendario_eventos().assign_recurso(entity.id, rid).await?;
        }
    }

    let rec_ids = tx.calendario_eventos().get_recursos_ids(entity.id).await?;
    let mut recursos_dto = Vec::new();
    for rid in rec_ids {
        if let Some(r) = tx.calendario_recursos().find_by_id(rid).await? {
            recursos_dto.push(CalendarioRecursoDto {
                id: r.id,
                grupo_id: r.grupo_id,
                grupo_nombre: None,
                nombre: r.nombre,
                tipo: r.tipo,
                empleado_id: r.empleado_id,
                color: r.color,
                activo: r.activo,
                row_version: r.audit.row_version,
            });
        }
    }

    tx.commit().await?;

    Ok(CalendarioEventoDto {
        id: entity.id,
        titulo: entity.titulo,
        descripcion: entity.descripcion,
        tipo: entity.tipo,
        inicio: time::to_storage(entity.inicio),
        fin: time::to_storage(entity.fin),
        todo_el_dia: entity.todo_el_dia,
        color: entity.color,
        trabajo_id: entity.trabajo_id,
        kanban_tarjeta_id: entity.kanban_tarjeta_id,
        recursos: recursos_dto,
        es_virtual: false,
        row_version: entity.audit.row_version,
    })
}

pub async fn mover_evento_impl(
    uow: &dyn UnitOfWork,
    clock: &dyn ClockPort,
    id: Uuid,
    nuevo_inicio: &str,
    nuevo_fin: &str,
    row_version: RowVersion,
) -> AppResult<()> {
    let inicio: DateTime<Utc> = time::from_storage(nuevo_inicio)
        .map_err(|e| AppError::Validation(vec![FieldError::new("inicio", e.to_string())]))?;
    let fin: DateTime<Utc> = time::from_storage(nuevo_fin)
        .map_err(|e| AppError::Validation(vec![FieldError::new("fin", e.to_string())]))?;

    if fin < inicio {
        return Err(AppError::Validation(vec![FieldError::new(
            "fin",
            "Validation.Calendario.FinMenorQueInicio",
        )]));
    }

    let tx = uow.begin().await?;
    let mut entity = tx
        .calendario_eventos()
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound {
            entity: "CalendarioEvento",
            id: id.to_string(),
        })?;

    if entity.audit.row_version != row_version {
        return Err(AppError::Concurrency {
            entity: "calendario_eventos",
        });
    }

    let now = clock.now_utc();
    entity.inicio = inicio;
    entity.fin = fin;
    entity.audit.touch(now);

    tx.calendario_eventos().update(&entity).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn delete_evento_impl(
    uow: &dyn UnitOfWork,
    id: Uuid,
    row_version: RowVersion,
) -> AppResult<()> {
    let tx = uow.begin().await?;
    tx.calendario_eventos().delete(id, &row_version).await?;
    tx.commit().await?;
    Ok(())
}
