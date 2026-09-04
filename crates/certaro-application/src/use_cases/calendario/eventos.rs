use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use certaro_domain::entities::audit::Audit;
use certaro_domain::entities::calendario::{CalendarioEvento, CalendarioRecurso, TipoEvento};
use certaro_domain::time;
use certaro_domain::RowVersion;

use crate::dtos::calendario::{
    ActualizarEventoInput, CalendarioEventoDto, CalendarioRecursoDto, CrearEventoInput,
};
use crate::error::{AppError, FieldError};
use crate::result::AppResult;

use super::CalendarioService;

impl CalendarioService {
    // =========================================================================
    // Eventos (con Proyección Virtual de Feriados, Trabajos y Facturas)
    // =========================================================================

    pub async fn list_eventos(
        &self,
        desde_iso: &str,
        hasta_iso: &str,
    ) -> AppResult<Vec<CalendarioEventoDto>> {
        let desde: DateTime<Utc> = time::from_storage(desde_iso)
            .map_err(|e| AppError::Validation(vec![FieldError::new("desde", e.to_string())]))?;
        let hasta: DateTime<Utc> = time::from_storage(hasta_iso)
            .map_err(|e| AppError::Validation(vec![FieldError::new("hasta", e.to_string())]))?;

        let tx = self.uow.begin().await?;

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
            let id = self.id_gen.new_id();

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

    pub async fn create_evento(&self, input: CrearEventoInput) -> AppResult<CalendarioEventoDto> {
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

        let id = self.id_gen.new_id();
        let now = self.clock.now_utc();
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

        let tx = self.uow.begin().await?;
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

    pub async fn update_evento(
        &self,
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

        let tx = self.uow.begin().await?;
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

        let now = self.clock.now_utc();
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

    pub async fn mover_evento(
        &self,
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

        let tx = self.uow.begin().await?;
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

        let now = self.clock.now_utc();
        entity.inicio = inicio;
        entity.fin = fin;
        entity.audit.touch(now);

        tx.calendario_eventos().update(&entity).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn delete_evento(&self, id: Uuid, row_version: RowVersion) -> AppResult<()> {
        let tx = self.uow.begin().await?;
        tx.calendario_eventos().delete(id, &row_version).await?;
        tx.commit().await?;
        Ok(())
    }
}
