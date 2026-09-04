//! Resource and Resource Group repository implementations for Calendar.

use std::sync::Arc;
use async_trait::async_trait;
use certaro_application::ports::repositories::{
    CalendarioGrupoRecursoRepository, CalendarioRecursoRepository,
};
use certaro_application::{AppError, AppResult};
use certaro_domain::entities::calendario::{CalendarioGrupoRecurso, CalendarioRecurso};
use certaro_domain::time;
use certaro_domain::RowVersion;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder,
    Set,
};
use uuid::Uuid;

use crate::persistence::mappers::calendario::{
    grupo_recurso_to_active, grupo_recurso_to_domain, recurso_to_active, recurso_to_domain,
};
use crate::persistence::models::{calendario_grupo_recurso, calendario_recurso};

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
