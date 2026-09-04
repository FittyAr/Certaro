use std::sync::Arc;
use async_trait::async_trait;
use certaro_application::ports::repositories::KanbanChecklistRepository;
use certaro_application::{AppError, AppResult};
use certaro_domain::entities::kanban::KanbanTarjetaChecklist;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder, Set};
use uuid::Uuid;
use crate::persistence::mappers::kanban::{checklist_to_active, checklist_to_domain};
use crate::persistence::models::kanban_tarjeta_checklist;

pub struct SeaOrmKanbanChecklistRepository {
    tx: Arc<DatabaseTransaction>,
}

impl SeaOrmKanbanChecklistRepository {
    pub fn new(tx: Arc<DatabaseTransaction>) -> Self {
        Self { tx }
    }

    fn conn(&self) -> &DatabaseTransaction {
        self.tx.as_ref()
    }
}

#[async_trait]
impl KanbanChecklistRepository for SeaOrmKanbanChecklistRepository {
    async fn list_by_tarjeta(&self, tarjeta_id: Uuid) -> AppResult<Vec<KanbanTarjetaChecklist>> {
        let models = kanban_tarjeta_checklist::Entity::find()
            .filter(kanban_tarjeta_checklist::Column::TarjetaId.eq(tarjeta_id.to_string()))
            .filter(kanban_tarjeta_checklist::Column::IsDeleted.eq(false))
            .order_by_asc(kanban_tarjeta_checklist::Column::Orden)
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;

        models.into_iter().map(checklist_to_domain).collect()
    }

    async fn insert(&self, entity: &KanbanTarjetaChecklist) -> AppResult<()> {
        let active = checklist_to_active(entity);
        active
            .insert(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn update(&self, entity: &KanbanTarjetaChecklist) -> AppResult<()> {
        let active = checklist_to_active(entity);
        active
            .update(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> AppResult<()> {
        let current = kanban_tarjeta_checklist::Entity::find_by_id(id.to_string())
            .filter(kanban_tarjeta_checklist::Column::IsDeleted.eq(false))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;

        if let Some(c) = current {
            let mut active: kanban_tarjeta_checklist::ActiveModel = c.into();
            active.is_deleted = Set(true);
            active.deleted_at = Set(Some(chrono::Utc::now().to_rfc3339()));
            active
                .update(self.conn())
                .await
                .map_err(AppError::persistence)?;
        }
        Ok(())
    }
}
