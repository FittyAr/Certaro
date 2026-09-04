use std::sync::Arc;
use async_trait::async_trait;
use certaro_application::ports::repositories::KanbanColumnaRepository;
use certaro_application::{AppError, AppResult};
use certaro_domain::entities::kanban::KanbanColumna;
use certaro_domain::RowVersion;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder, Set};
use uuid::Uuid;
use crate::persistence::mappers::kanban::{columna_to_active, columna_to_domain};
use crate::persistence::models::kanban_columna;

pub struct SeaOrmKanbanColumnaRepository {
    tx: Arc<DatabaseTransaction>,
}

impl SeaOrmKanbanColumnaRepository {
    pub fn new(tx: Arc<DatabaseTransaction>) -> Self {
        Self { tx }
    }

    fn conn(&self) -> &DatabaseTransaction {
        self.tx.as_ref()
    }
}

#[async_trait]
impl KanbanColumnaRepository for SeaOrmKanbanColumnaRepository {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<KanbanColumna>> {
        let found = kanban_columna::Entity::find_by_id(id.to_string())
            .filter(kanban_columna::Column::IsDeleted.eq(false))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;
        found.map(columna_to_domain).transpose()
    }

    async fn list_by_tablero(&self, tablero_id: Uuid) -> AppResult<Vec<KanbanColumna>> {
        let models = kanban_columna::Entity::find()
            .filter(kanban_columna::Column::TableroId.eq(tablero_id.to_string()))
            .filter(kanban_columna::Column::IsDeleted.eq(false))
            .order_by_asc(kanban_columna::Column::Orden)
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;

        models.into_iter().map(columna_to_domain).collect()
    }

    async fn insert(&self, entity: &KanbanColumna) -> AppResult<()> {
        let active = columna_to_active(entity);
        active
            .insert(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn update(&self, entity: &KanbanColumna) -> AppResult<()> {
        let active = columna_to_active(entity);
        active
            .update(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn delete(&self, id: Uuid, row_version: &RowVersion) -> AppResult<()> {
        let current = kanban_columna::Entity::find_by_id(id.to_string())
            .filter(kanban_columna::Column::IsDeleted.eq(false))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?
            .ok_or_else(|| AppError::NotFound {
                entity: "kanban_columnas",
                id: id.to_string(),
            })?;

        if current.row_version != row_version.as_bytes() {
            return Err(AppError::Concurrency {
                entity: "kanban_columnas",
            });
        }

        let mut active: kanban_columna::ActiveModel = current.into();
        active.is_deleted = Set(true);
        active.deleted_at = Set(Some(chrono::Utc::now().to_rfc3339()));
        active
            .update(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }
}
