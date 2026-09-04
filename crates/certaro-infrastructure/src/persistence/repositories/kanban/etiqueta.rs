use std::sync::Arc;
use async_trait::async_trait;
use certaro_application::ports::repositories::KanbanEtiquetaRepository;
use certaro_application::{AppError, AppResult};
use certaro_domain::entities::kanban::KanbanEtiqueta;
use certaro_domain::RowVersion;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set};
use uuid::Uuid;
use crate::persistence::mappers::kanban::{etiqueta_to_active, etiqueta_to_domain};
use crate::persistence::models::{kanban_etiqueta, kanban_tarjeta_etiqueta};

pub struct SeaOrmKanbanEtiquetaRepository {
    tx: Arc<DatabaseTransaction>,
}

impl SeaOrmKanbanEtiquetaRepository {
    pub fn new(tx: Arc<DatabaseTransaction>) -> Self {
        Self { tx }
    }

    fn conn(&self) -> &DatabaseTransaction {
        self.tx.as_ref()
    }
}

#[async_trait]
impl KanbanEtiquetaRepository for SeaOrmKanbanEtiquetaRepository {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<KanbanEtiqueta>> {
        let found = kanban_etiqueta::Entity::find_by_id(id.to_string())
            .filter(kanban_etiqueta::Column::IsDeleted.eq(false))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;
        found.map(etiqueta_to_domain).transpose()
    }

    async fn list_all(&self) -> AppResult<Vec<KanbanEtiqueta>> {
        let models = kanban_etiqueta::Entity::find()
            .filter(kanban_etiqueta::Column::IsDeleted.eq(false))
            .order_by_asc(kanban_etiqueta::Column::Nombre)
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;

        models.into_iter().map(etiqueta_to_domain).collect()
    }

    async fn list_by_tarjeta(&self, tarjeta_id: Uuid) -> AppResult<Vec<KanbanEtiqueta>> {
        let tag_ids: Vec<String> = kanban_tarjeta_etiqueta::Entity::find()
            .select_only()
            .column(kanban_tarjeta_etiqueta::Column::EtiquetaId)
            .filter(kanban_tarjeta_etiqueta::Column::TarjetaId.eq(tarjeta_id.to_string()))
            .into_tuple()
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;

        if tag_ids.is_empty() {
            return Ok(Vec::new());
        }

        let models = kanban_etiqueta::Entity::find()
            .filter(kanban_etiqueta::Column::Id.is_in(tag_ids))
            .filter(kanban_etiqueta::Column::IsDeleted.eq(false))
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;

        models.into_iter().map(etiqueta_to_domain).collect()
    }

    async fn assign(&self, tarjeta_id: Uuid, etiqueta_id: Uuid) -> AppResult<()> {
        let exists = kanban_tarjeta_etiqueta::Entity::find()
            .filter(kanban_tarjeta_etiqueta::Column::TarjetaId.eq(tarjeta_id.to_string()))
            .filter(kanban_tarjeta_etiqueta::Column::EtiquetaId.eq(etiqueta_id.to_string()))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;

        if exists.is_none() {
            let active = kanban_tarjeta_etiqueta::ActiveModel {
                tarjeta_id: Set(tarjeta_id.to_string()),
                etiqueta_id: Set(etiqueta_id.to_string()),
            };
            active
                .insert(self.conn())
                .await
                .map_err(AppError::persistence)?;
        }
        Ok(())
    }

    async fn unassign(&self, tarjeta_id: Uuid, etiqueta_id: Uuid) -> AppResult<()> {
        kanban_tarjeta_etiqueta::Entity::delete_many()
            .filter(kanban_tarjeta_etiqueta::Column::TarjetaId.eq(tarjeta_id.to_string()))
            .filter(kanban_tarjeta_etiqueta::Column::EtiquetaId.eq(etiqueta_id.to_string()))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn insert(&self, entity: &KanbanEtiqueta) -> AppResult<()> {
        let active = etiqueta_to_active(entity);
        active
            .insert(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn update(&self, entity: &KanbanEtiqueta) -> AppResult<()> {
        let active = etiqueta_to_active(entity);
        active
            .update(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn delete(&self, id: Uuid, row_version: &RowVersion) -> AppResult<()> {
        let current = kanban_etiqueta::Entity::find_by_id(id.to_string())
            .filter(kanban_etiqueta::Column::IsDeleted.eq(false))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?
            .ok_or_else(|| AppError::NotFound {
                entity: "kanban_etiquetas",
                id: id.to_string(),
            })?;

        if current.row_version != row_version.as_bytes() {
            return Err(AppError::Concurrency {
                entity: "kanban_etiquetas",
            });
        }

        let mut active: kanban_etiqueta::ActiveModel = current.into();
        active.is_deleted = Set(true);
        active.deleted_at = Set(Some(chrono::Utc::now().to_rfc3339()));
        active
            .update(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }
}
