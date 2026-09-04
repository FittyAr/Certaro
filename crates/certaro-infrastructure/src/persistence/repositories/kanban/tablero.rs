use std::sync::Arc;
use async_trait::async_trait;
use certaro_application::ports::repositories::KanbanTableroRepository;
use certaro_application::{AppError, AppResult};
use certaro_domain::entities::kanban::KanbanTablero;
use certaro_domain::RowVersion;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder, Set};
use uuid::Uuid;
use crate::persistence::mappers::kanban::{tablero_to_active, tablero_to_domain};
use crate::persistence::models::kanban_tablero;

pub struct SeaOrmKanbanTableroRepository {
    tx: Arc<DatabaseTransaction>,
}

impl SeaOrmKanbanTableroRepository {
    pub fn new(tx: Arc<DatabaseTransaction>) -> Self {
        Self { tx }
    }

    fn conn(&self) -> &DatabaseTransaction {
        self.tx.as_ref()
    }
}

#[async_trait]
impl KanbanTableroRepository for SeaOrmKanbanTableroRepository {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<KanbanTablero>> {
        let found = kanban_tablero::Entity::find_by_id(id.to_string())
            .filter(kanban_tablero::Column::IsDeleted.eq(false))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;
        found.map(tablero_to_domain).transpose()
    }

    async fn list_all(&self) -> AppResult<Vec<KanbanTablero>> {
        let models = kanban_tablero::Entity::find()
            .filter(kanban_tablero::Column::IsDeleted.eq(false))
            .order_by_asc(kanban_tablero::Column::Nombre)
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;

        models.into_iter().map(tablero_to_domain).collect()
    }

    async fn insert(&self, entity: &KanbanTablero) -> AppResult<()> {
        let active = tablero_to_active(entity);
        active
            .insert(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn update(&self, entity: &KanbanTablero) -> AppResult<()> {
        let active = tablero_to_active(entity);
        active
            .update(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn delete(&self, id: Uuid, row_version: &RowVersion) -> AppResult<()> {
        let current = kanban_tablero::Entity::find_by_id(id.to_string())
            .filter(kanban_tablero::Column::IsDeleted.eq(false))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?
            .ok_or_else(|| AppError::NotFound {
                entity: "kanban_tableros",
                id: id.to_string(),
            })?;

        if current.row_version != row_version.as_bytes() {
            return Err(AppError::Concurrency {
                entity: "kanban_tableros",
            });
        }

        let mut active: kanban_tablero::ActiveModel = current.into();
        active.is_deleted = Set(true);
        active.deleted_at = Set(Some(chrono::Utc::now().to_rfc3339()));
        active
            .update(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }
}
