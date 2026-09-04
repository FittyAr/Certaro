use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use certaro_domain::entities::calendario::{CalendarioRecurso, TipoEvento};
use certaro_domain::time;
use certaro_domain::RowVersion;

use crate::dtos::calendario::{CalendarioEventoDto, CalendarioRecursoDto};
use crate::error::{AppError, FieldError};
use crate::ports::id_generator::IdGeneratorPort;
use crate::ports::repositories::UnitOfWork;
use crate::result::AppResult;

pub async fn list_eventos_impl(
    uow: &dyn UnitOfWork,
    id_gen: &dyn IdGeneratorPort,
    desde_iso: &str,
    hasta_iso: &str,
) -> AppResult<Vec<CalendarioEventoDto>> {
    let desde: DateTime<Utc> = time::from_storage(desde_iso)
        .map_err(|e| AppError::Validation(vec![FieldError::new("desde", e.to_string())]))?;
    let hasta: DateTime<Utc> = time::from_storage(hasta_iso)
        .map_err(|e| AppError::Validation(vec![FieldError::new("hasta", e.to_string())]))?;

    let tx = uow.begin().await?;

    // 1. Native calendar events
    let eventos = tx.calendario_eventos().list_en_rango(desde, hasta).await?;
    let all_recursos = tx.calendario_recursos().list_all().await?;
    let recurso_map: HashMap<Uuid, CalendarioRecurso> =
        all_recursos.into_iter().map(|r| (r.id, r)).collect();

    let mut dtos = Vec::new();

    for ev in eventos {
        let rec_ids = tx.calendario_eventos().get_recursos_ids(ev.id).await?;
        let recursos_dto: Vec<CalendarioRecursoDto> = rec_ids
            .into_iter()
            .filter_map(|rid| recurso_map.get(&rid))
            .map(|r| CalendarioRecursoDto {
                id: r.id,
                grupo_id: r.grupo_id,
                grupo_nombre: None,
                nombre: r.nombre.clone(),
                tipo: r.tipo,
                empleado_id: r.empleado_id,
                color: r.color.clone(),
                activo: r.activo,
                row_version: r.audit.row_version,
            })
            .collect();

        dtos.push(CalendarioEventoDto {
            id: ev.id,
            titulo: ev.titulo,
            descripcion: ev.descripcion,
            tipo: ev.tipo,
            inicio: time::to_storage(ev.inicio),
            fin: time::to_storage(ev.fin),
            todo_el_dia: ev.todo_el_dia,
            color: ev.color,
            trabajo_id: ev.trabajo_id,
            kanban_tarjeta_id: ev.kanban_tarjeta_id,
            recursos: recursos_dto,
            es_virtual: false,
            row_version: ev.audit.row_version,
        });
    }

    // 2. Virtual Projection: Feriados
    let desde_date = desde.date_naive();
    let hasta_date = hasta.date_naive();
    let feriados = tx.feriados().del_rango(desde_date, hasta_date).await?;

    for f in feriados {
        let fecha_str = f.fecha.format("%Y-%m-%d").to_string();
        let inicio = format!("{}T00:00:00.000Z", fecha_str);
        let fin = format!("{}T23:59:59.999Z", fecha_str);
        let id = id_gen.new_id();

        dtos.push(CalendarioEventoDto {
            id,
            titulo: format!("Feriado: {}", f.nombre),
            descripcion: f.tipo,
            tipo: TipoEvento::Otro,
            inicio,
            fin,
            todo_el_dia: true,
            color: Some("#ef4444".to_string()),
            trabajo_id: None,
            kanban_tarjeta_id: None,
            recursos: Vec::new(),
            es_virtual: true,
            row_version: RowVersion::from_bytes([0; 8]),
        });
    }

    // 3. Virtual Projection: Trabajos (con fecha de inicio o fecha fin)
    let trabajos = tx.trabajos().lookup(None, None, 1000).await?;
    for tr in trabajos {
        let fecha_inicio = tr.fecha_inicio;
        if fecha_inicio >= desde_date && fecha_inicio <= hasta_date {
            let inicio = format!("{}T08:00:00.000Z", fecha_inicio.format("%Y-%m-%d"));
            let fin = if let Some(ff) = tr.fecha_fin {
                format!("{}T18:00:00.000Z", ff.format("%Y-%m-%d"))
            } else {
                format!("{}T17:00:00.000Z", fecha_inicio.format("%Y-%m-%d"))
            };

            dtos.push(CalendarioEventoDto {
                id: tr.id,
                titulo: format!("Trabajo: {}", tr.descripcion),
                descripcion: Some(tr.descripcion),
                tipo: TipoEvento::Trabajo,
                inicio,
                fin,
                todo_el_dia: false,
                color: Some("#3b82f6".to_string()),
                trabajo_id: Some(tr.id),
                kanban_tarjeta_id: None,
                recursos: Vec::new(),
                es_virtual: true,
                row_version: tr.audit.row_version,
            });
        }
    }

    // Sort by start time ascending
    dtos.sort_by(|a, b| a.inicio.cmp(&b.inicio));

    Ok(dtos)
}
