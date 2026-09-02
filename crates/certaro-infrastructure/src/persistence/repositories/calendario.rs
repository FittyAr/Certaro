//! SeaORM repositories for the Calendar module.

use std::sync::Arc;
use async_trait::async_trait;
use certaro_application::ports::repositories::{
    CalendarioEventoRepository, CalendarioGrupoRecursoRepository, CalendarioRecursoRepository,
};
use certaro_application::{AppError, AppResult};
use certaro_domain::entities::calendario::{
    CalendarioEvento, CalendarioGrupoRecurso, CalendarioRecurso,
};
use certaro_domain::time;
use certaro_domain::RowVersion;
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder,
    Set,
};
use uuid::Uuid;

use crate::persistence::mappers::calendario::{
    evento_to_active, evento_to_domain, grupo_recurso_to_active, grupo_recurso_to_domain,
    recurso_to_active, recurso_to_domain,
};
use crate::persistence::models::{
    calendario_evento, calendario_evento_recurso, calendario_grupo_recurso, calendario_recurso,
};

// =========================================================================
// SeaOrmCalendarioGrupoRecursoRepository
// =========================================================================

pub struct SeaOrmCalendarioGrupoRecursoRepository {
    tx: Arc<DatabaseTransaction>,
}

impl SeaOrmCalendarioGrupoRecursoRepository {
    pub fn new(tx: Arc<DatabaseTransaction>) -> Self {
        Self { tx }
    }

    fn conn(&self) -> &DatabaseTransaction {
        &self.tx
    }
}

#[async_trait]
impl CalendarioGrupoRecursoRepository for SeaOrmCalendarioGrupoRecursoRepository {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<CalendarioGrupoRecurso>> {
        let id_str = id.to_string();
        let model = calendario_grupo_recurso::Entity::find_by_id(&id_str)
            .filter(calendario_grupo_recurso::Column::IsDeleted.eq(false))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;

        model.map(grupo_recurso_to_domain).transpose()
    }

    async fn list_all(&self) -> AppResult<Vec<CalendarioGrupoRecurso>> {
        let models = calendario_grupo_recurso::Entity::find()
            .filter(calendario_grupo_recurso::Column::IsDeleted.eq(false))
            .order_by_asc(calendario_grupo_recurso::Column::Nombre)
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;

        models.into_iter().map(grupo_recurso_to_domain).collect()
    }

    async fn insert(&self, entity: &CalendarioGrupoRecurso) -> AppResult<()> {
        let active = grupo_recurso_to_active(entity);
        active
            .insert(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn update(&self, entity: &CalendarioGrupoRecurso) -> AppResult<()> {
        let active = grupo_recurso_to_active(entity);
        active
            .update(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn delete(&self, id: Uuid, row_version: &RowVersion) -> AppResult<()> {
        let id_str = id.to_string();
        let model = calendario_grupo_recurso::Entity::find_by_id(&id_str)
            .filter(calendario_grupo_recurso::Column::IsDeleted.eq(false))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?
            .ok_or_else(|| AppError::NotFound {
                entity: "CalendarioGrupoRecurso",
                id: id_str.clone(),
            })?;

        if model.row_version != row_version.as_bytes() {
            return Err(AppError::Concurrency {
                entity: "calendario_grupos_recurso",
            });
        }

        let mut active: calendario_grupo_recurso::ActiveModel = model.into();
        active.is_deleted = Set(true);
        active.deleted_at = Set(Some(time::to_storage(Utc::now())));
        active
            .update(self.conn())
            .await
            .map_err(AppError::persistence)?;

        Ok(())
    }
}

// =========================================================================
// SeaOrmCalendarioRecursoRepository
// =========================================================================

pub struct SeaOrmCalendarioRecursoRepository {
    tx: Arc<DatabaseTransaction>,
}

impl SeaOrmCalendarioRecursoRepository {
    pub fn new(tx: Arc<DatabaseTransaction>) -> Self {
        Self { tx }
    }

    fn conn(&self) -> &DatabaseTransaction {
        &self.tx
    }
}

#[async_trait]
impl CalendarioRecursoRepository for SeaOrmCalendarioRecursoRepository {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<CalendarioRecurso>> {
        let id_str = id.to_string();
        let model = calendario_recurso::Entity::find_by_id(&id_str)
            .filter(calendario_recurso::Column::IsDeleted.eq(false))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;

        model.map(recurso_to_domain).transpose()
    }

    async fn find_by_empleado_id(&self, empleado_id: Uuid) -> AppResult<Option<CalendarioRecurso>> {
        let emp_str = empleado_id.to_string();
        let model = calendario_recurso::Entity::find()
            .filter(calendario_recurso::Column::EmpleadoId.eq(Some(emp_str)))
            .filter(calendario_recurso::Column::IsDeleted.eq(false))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;

        model.map(recurso_to_domain).transpose()
    }

    async fn list_all(&self) -> AppResult<Vec<CalendarioRecurso>> {
        let models = calendario_recurso::Entity::find()
            .filter(calendario_recurso::Column::IsDeleted.eq(false))
            .order_by_asc(calendario_recurso::Column::Nombre)
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;

        models.into_iter().map(recurso_to_domain).collect()
    }

    async fn list_activos(&self) -> AppResult<Vec<CalendarioRecurso>> {
        let models = calendario_recurso::Entity::find()
            .filter(calendario_recurso::Column::IsDeleted.eq(false))
            .filter(calendario_recurso::Column::Activo.eq(true))
            .order_by_asc(calendario_recurso::Column::Nombre)
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;

        models.into_iter().map(recurso_to_domain).collect()
    }

    async fn insert(&self, entity: &CalendarioRecurso) -> AppResult<()> {
        let active = recurso_to_active(entity);
        active
            .insert(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn update(&self, entity: &CalendarioRecurso) -> AppResult<()> {
        let active = recurso_to_active(entity);
        active
            .update(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn delete(&self, id: Uuid, row_version: &RowVersion) -> AppResult<()> {
        let id_str = id.to_string();
        let model = calendario_recurso::Entity::find_by_id(&id_str)
            .filter(calendario_recurso::Column::IsDeleted.eq(false))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?
            .ok_or_else(|| AppError::NotFound {
                entity: "CalendarioRecurso",
                id: id_str.clone(),
            })?;

        if model.row_version != row_version.as_bytes() {
            return Err(AppError::Concurrency {
                entity: "calendario_recursos",
            });
        }

        let mut active: calendario_recurso::ActiveModel = model.into();
        active.is_deleted = Set(true);
        active.deleted_at = Set(Some(time::to_storage(Utc::now())));
        active
            .update(self.conn())
            .await
            .map_err(AppError::persistence)?;

        Ok(())
    }
}

// =========================================================================
// SeaOrmCalendarioEventoRepository
// =========================================================================

pub struct SeaOrmCalendarioEventoRepository {
    tx: Arc<DatabaseTransaction>,
}

impl SeaOrmCalendarioEventoRepository {
    pub fn new(tx: Arc<DatabaseTransaction>) -> Self {
        Self { tx }
    }

    fn conn(&self) -> &DatabaseTransaction {
        &self.tx
    }
}

#[async_trait]
impl CalendarioEventoRepository for SeaOrmCalendarioEventoRepository {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<CalendarioEvento>> {
        let id_str = id.to_string();
        let model = calendario_evento::Entity::find_by_id(&id_str)
            .filter(calendario_evento::Column::IsDeleted.eq(false))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;

        model.map(evento_to_domain).transpose()
    }

    async fn list_en_rango(
        &self,
        desde: DateTime<Utc>,
        hasta: DateTime<Utc>,
    ) -> AppResult<Vec<CalendarioEvento>> {
        let desde_str = time::to_storage(desde);
        let hasta_str = time::to_storage(hasta);

        let models = calendario_evento::Entity::find()
            .filter(calendario_evento::Column::IsDeleted.eq(false))
            .filter(calendario_evento::Column::Inicio.lte(hasta_str))
            .filter(calendario_evento::Column::Fin.gte(desde_str))
            .order_by_asc(calendario_evento::Column::Inicio)
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;

        models.into_iter().map(evento_to_domain).collect()
    }

    async fn list_por_recurso(
        &self,
        recurso_id: Uuid,
        desde: DateTime<Utc>,
        hasta: DateTime<Utc>,
    ) -> AppResult<Vec<CalendarioEvento>> {
        let recurso_str = recurso_id.to_string();
        let links = calendario_evento_recurso::Entity::find()
            .filter(calendario_evento_recurso::Column::RecursoId.eq(recurso_str))
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;

        let evento_ids: Vec<String> = links.into_iter().map(|l| l.evento_id).collect();
        if evento_ids.is_empty() {
            return Ok(Vec::new());
        }

        let desde_str = time::to_storage(desde);
        let hasta_str = time::to_storage(hasta);

        let models = calendario_evento::Entity::find()
            .filter(calendario_evento::Column::Id.is_in(evento_ids))
            .filter(calendario_evento::Column::IsDeleted.eq(false))
            .filter(calendario_evento::Column::Inicio.lte(hasta_str))
            .filter(calendario_evento::Column::Fin.gte(desde_str))
            .order_by_asc(calendario_evento::Column::Inicio)
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;

        models.into_iter().map(evento_to_domain).collect()
    }

    async fn insert(&self, entity: &CalendarioEvento) -> AppResult<()> {
        let active = evento_to_active(entity);
        active
            .insert(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn update(&self, entity: &CalendarioEvento) -> AppResult<()> {
        let active = evento_to_active(entity);
        active
            .update(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn delete(&self, id: Uuid, row_version: &RowVersion) -> AppResult<()> {
        let id_str = id.to_string();
        let model = calendario_evento::Entity::find_by_id(&id_str)
            .filter(calendario_evento::Column::IsDeleted.eq(false))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?
            .ok_or_else(|| AppError::NotFound {
                entity: "CalendarioEvento",
                id: id_str.clone(),
            })?;

        if model.row_version != row_version.as_bytes() {
            return Err(AppError::Concurrency {
                entity: "calendario_eventos",
            });
        }

        let mut active: calendario_evento::ActiveModel = model.into();
        active.is_deleted = Set(true);
        active.deleted_at = Set(Some(time::to_storage(Utc::now())));
        active
            .update(self.conn())
            .await
            .map_err(AppError::persistence)?;

        Ok(())
    }

    async fn assign_recurso(&self, evento_id: Uuid, recurso_id: Uuid) -> AppResult<()> {
        let active = calendario_evento_recurso::ActiveModel {
            evento_id: Set(evento_id.to_string()),
            recurso_id: Set(recurso_id.to_string()),
        };
        let _ = active.insert(self.conn()).await;
        Ok(())
    }

    async fn unassign_recursos(&self, evento_id: Uuid) -> AppResult<()> {
        let evento_str = evento_id.to_string();
        calendario_evento_recurso::Entity::delete_many()
            .filter(calendario_evento_recurso::Column::EventoId.eq(evento_str))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn get_recursos_ids(&self, evento_id: Uuid) -> AppResult<Vec<Uuid>> {
        let evento_str = evento_id.to_string();
        let links = calendario_evento_recurso::Entity::find()
            .filter(calendario_evento_recurso::Column::EventoId.eq(evento_str))
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;

        let mut ids = Vec::new();
        for l in links {
            if let Ok(u) = Uuid::parse_str(&l.recurso_id) {
                ids.push(u);
            }
        }
        Ok(ids)
    }
}
