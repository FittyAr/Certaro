use std::sync::Arc;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use certaro_domain::entities::audit::Audit;
use certaro_domain::entities::calendario::{
    CalendarioEvento, CalendarioGrupoRecurso, CalendarioRecurso, TipoEvento, TipoRecurso,
};
use certaro_domain::time;
use certaro_domain::RowVersion;

use crate::dtos::calendario::{
    ActualizarEventoInput, ActualizarGrupoRecursoInput, ActualizarRecursoInput,
    CalendarioEventoDto, CalendarioGrupoRecursoDto, CalendarioRecursoDto, CrearEventoInput,
    CrearGrupoRecursoInput, CrearRecursoInput,
};
use crate::error::{AppError, FieldError};
use crate::ports::clock::ClockPort;
use crate::ports::id_generator::IdGeneratorPort;
use crate::ports::repositories::UnitOfWork;
use crate::result::AppResult;

pub struct CalendarioService {
    uow: Arc<dyn UnitOfWork>,
    clock: Arc<dyn ClockPort>,
    id_gen: Arc<dyn IdGeneratorPort>,
}

impl CalendarioService {
    pub fn new(
        uow: Arc<dyn UnitOfWork>,
        clock: Arc<dyn ClockPort>,
        id_gen: Arc<dyn IdGeneratorPort>,
    ) -> Self {
        Self { uow, clock, id_gen }
    }

    // =========================================================================
    // Grupos de Recurso
    // =========================================================================

    pub async fn list_grupos(&self) -> AppResult<Vec<CalendarioGrupoRecursoDto>> {
        let tx = self.uow.begin().await?;
        let grupos = tx.calendario_grupos_recurso().list_all().await?;
        Ok(grupos
            .into_iter()
            .map(|g| CalendarioGrupoRecursoDto {
                id: g.id,
                nombre: g.nombre,
                color: g.color,
                row_version: g.audit.row_version,
            })
            .collect())
    }

    pub async fn create_grupo(
        &self,
        input: CrearGrupoRecursoInput,
    ) -> AppResult<CalendarioGrupoRecursoDto> {
        let nombre = input.nombre.trim();
        if nombre.is_empty() {
            return Err(AppError::Validation(vec![FieldError::new(
                "nombre",
                "Validation.Calendario.NombreRequerido",
            )]));
        }

        let id = self.id_gen.new_id();
        let now = self.clock.now_utc();
        let audit = Audit::new(now);

        let entity = CalendarioGrupoRecurso {
            id,
            nombre: nombre.to_string(),
            color: input.color,
            audit,
        };

        let tx = self.uow.begin().await?;
        tx.calendario_grupos_recurso().insert(&entity).await?;
        tx.commit().await?;

        Ok(CalendarioGrupoRecursoDto {
            id: entity.id,
            nombre: entity.nombre,
            color: entity.color,
            row_version: entity.audit.row_version,
        })
    }

    pub async fn update_grupo(
        &self,
        id: Uuid,
        input: ActualizarGrupoRecursoInput,
    ) -> AppResult<CalendarioGrupoRecursoDto> {
        let nombre = input.nombre.trim();
        if nombre.is_empty() {
            return Err(AppError::Validation(vec![FieldError::new(
                "nombre",
                "Validation.Calendario.NombreRequerido",
            )]));
        }

        let tx = self.uow.begin().await?;
        let mut entity = tx
            .calendario_grupos_recurso()
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound {
                entity: "CalendarioGrupoRecurso",
                id: id.to_string(),
            })?;

        if entity.audit.row_version != input.row_version {
            return Err(AppError::Concurrency {
                entity: "calendario_grupos_recurso",
            });
        }

        let now = self.clock.now_utc();
        entity.nombre = nombre.to_string();
        entity.color = input.color;
        entity.audit.touch(now);

        tx.calendario_grupos_recurso().update(&entity).await?;
        tx.commit().await?;

        Ok(CalendarioGrupoRecursoDto {
            id: entity.id,
            nombre: entity.nombre,
            color: entity.color,
            row_version: entity.audit.row_version,
        })
    }

    pub async fn delete_grupo(&self, id: Uuid, row_version: RowVersion) -> AppResult<()> {
        let tx = self.uow.begin().await?;
        tx.calendario_grupos_recurso().delete(id, &row_version).await?;
        tx.commit().await?;
        Ok(())
    }

    // =========================================================================
    // Recursos
    // =========================================================================

    pub async fn list_recursos(&self) -> AppResult<Vec<CalendarioRecursoDto>> {
        let tx = self.uow.begin().await?;
        let recursos = tx.calendario_recursos().list_all().await?;
        let grupos = tx.calendario_grupos_recurso().list_all().await?;

        let grupo_map: std::collections::HashMap<Uuid, String> =
            grupos.into_iter().map(|g| (g.id, g.nombre)).collect();

        Ok(recursos
            .into_iter()
            .map(|r| CalendarioRecursoDto {
                id: r.id,
                grupo_id: r.grupo_id,
                grupo_nombre: r.grupo_id.and_then(|gid| grupo_map.get(&gid).cloned()),
                nombre: r.nombre,
                tipo: r.tipo,
                empleado_id: r.empleado_id,
                color: r.color,
                activo: r.activo,
                row_version: r.audit.row_version,
            })
            .collect())
    }

    pub async fn create_recurso(
        &self,
        input: CrearRecursoInput,
    ) -> AppResult<CalendarioRecursoDto> {
        let nombre = input.nombre.trim();
        if nombre.is_empty() {
            return Err(AppError::Validation(vec![FieldError::new(
                "nombre",
                "Validation.Calendario.NombreRequerido",
            )]));
        }

        let id = self.id_gen.new_id();
        let now = self.clock.now_utc();
        let audit = Audit::new(now);

        let entity = CalendarioRecurso {
            id,
            grupo_id: input.grupo_id,
            nombre: nombre.to_string(),
            tipo: input.tipo,
            empleado_id: input.empleado_id,
            color: input.color,
            activo: true,
            audit,
        };

        let tx = self.uow.begin().await?;
        tx.calendario_recursos().insert(&entity).await?;

        let grupo_nombre = if let Some(gid) = entity.grupo_id {
            tx.calendario_grupos_recurso()
                .find_by_id(gid)
                .await?
                .map(|g| g.nombre)
        } else {
            None
        };

        tx.commit().await?;

        Ok(CalendarioRecursoDto {
            id: entity.id,
            grupo_id: entity.grupo_id,
            grupo_nombre,
            nombre: entity.nombre,
            tipo: entity.tipo,
            empleado_id: entity.empleado_id,
            color: entity.color,
            activo: entity.activo,
            row_version: entity.audit.row_version,
        })
    }

    pub async fn update_recurso(
        &self,
        id: Uuid,
        input: ActualizarRecursoInput,
    ) -> AppResult<CalendarioRecursoDto> {
        let nombre = input.nombre.trim();
        if nombre.is_empty() {
            return Err(AppError::Validation(vec![FieldError::new(
                "nombre",
                "Validation.Calendario.NombreRequerido",
            )]));
        }

        let tx = self.uow.begin().await?;
        let mut entity = tx
            .calendario_recursos()
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound {
                entity: "CalendarioRecurso",
                id: id.to_string(),
            })?;

        if entity.audit.row_version != input.row_version {
            return Err(AppError::Concurrency {
                entity: "calendario_recursos",
            });
        }

        let now = self.clock.now_utc();
        entity.grupo_id = input.grupo_id;
        entity.nombre = nombre.to_string();
        entity.tipo = input.tipo;
        entity.empleado_id = input.empleado_id;
        entity.color = input.color;
        entity.activo = input.activo;
        entity.audit.touch(now);

        tx.calendario_recursos().update(&entity).await?;

        let grupo_nombre = if let Some(gid) = entity.grupo_id {
            tx.calendario_grupos_recurso()
                .find_by_id(gid)
                .await?
                .map(|g| g.nombre)
        } else {
            None
        };

        tx.commit().await?;

        Ok(CalendarioRecursoDto {
            id: entity.id,
            grupo_id: entity.grupo_id,
            grupo_nombre,
            nombre: entity.nombre,
            tipo: entity.tipo,
            empleado_id: entity.empleado_id,
            color: entity.color,
            activo: entity.activo,
            row_version: entity.audit.row_version,
        })
    }

    pub async fn delete_recurso(&self, id: Uuid, row_version: RowVersion) -> AppResult<()> {
        let tx = self.uow.begin().await?;
        tx.calendario_recursos().delete(id, &row_version).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn sincronizar_empleados_a_recursos(&self) -> AppResult<()> {
        let tx = self.uow.begin().await?;
        let empleados = tx.empleados().activos().await?;
        let personal_grupo = tx
            .calendario_grupos_recurso()
            .list_all()
            .await?
            .into_iter()
            .find(|g| g.nombre.to_lowercase() == "personal");

        let grupo_id = personal_grupo.map(|g| g.id);
        let now = self.clock.now_utc();

        for emp in empleados {
            let existing = tx.calendario_recursos().find_by_empleado_id(emp.id).await?;
            if existing.is_none() {
                let rec = CalendarioRecurso {
                    id: self.id_gen.new_id(),
                    grupo_id,
                    nombre: emp.nombre,
                    tipo: TipoRecurso::Empleado,
                    empleado_id: Some(emp.id),
                    color: None,
                    activo: emp.activo,
                    audit: Audit::new(now),
                };
                tx.calendario_recursos().insert(&rec).await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

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
        let recurso_map: std::collections::HashMap<Uuid, CalendarioRecurso> =
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
