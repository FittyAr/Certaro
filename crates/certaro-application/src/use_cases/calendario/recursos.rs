use std::collections::HashMap;
use uuid::Uuid;

use certaro_domain::entities::audit::Audit;
use certaro_domain::entities::calendario::{CalendarioGrupoRecurso, CalendarioRecurso, TipoRecurso};
use certaro_domain::RowVersion;

use crate::dtos::calendario::{
    ActualizarGrupoRecursoInput, ActualizarRecursoInput, CalendarioGrupoRecursoDto,
    CalendarioRecursoDto, CrearGrupoRecursoInput, CrearRecursoInput,
};
use crate::error::{AppError, FieldError};
use crate::result::AppResult;

use super::CalendarioService;

impl CalendarioService {
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

        let grupo_map: HashMap<Uuid, String> =
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
}
