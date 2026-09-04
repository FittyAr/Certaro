use std::sync::Arc;
use async_trait::async_trait;
use certaro_application::ports::repositories::KanbanTarjetaRepository;
use certaro_application::{AppError, AppResult};
use certaro_domain::entities::kanban::KanbanTarjeta;
use certaro_domain::RowVersion;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set};
use uuid::Uuid;
use crate::persistence::mappers::kanban::{tarjeta_to_active, tarjeta_to_domain};
use crate::persistence::models::{kanban_columna, kanban_tarjeta};

pub struct SeaOrmKanbanTarjetaRepository {
    tx: Arc<DatabaseTransaction>,
}

impl SeaOrmKanbanTarjetaRepository {
    pub fn new(tx: Arc<DatabaseTransaction>) -> Self {
        Self { tx }
    }

    fn conn(&self) -> &DatabaseTransaction {
        self.tx.as_ref()
    }
}

#[async_trait]
impl KanbanTarjetaRepository for SeaOrmKanbanTarjetaRepository {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<KanbanTarjeta>> {
        let found = kanban_tarjeta::Entity::find_by_id(id.to_string())
            .filter(kanban_tarjeta::Column::IsDeleted.eq(false))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;
        found.map(tarjeta_to_domain).transpose()
    }

    async fn list_by_tablero(&self, tablero_id: Uuid) -> AppResult<Vec<KanbanTarjeta>> {
        // Find columns for this board first
        let column_ids: Vec<String> = kanban_columna::Entity::find()
            .select_only()
            .column(kanban_columna::Column::Id)
            .filter(kanban_columna::Column::TableroId.eq(tablero_id.to_string()))
            .filter(kanban_columna::Column::IsDeleted.eq(false))
            .into_tuple()
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;

        if column_ids.is_empty() {
            return Ok(Vec::new());
        }

        let models = kanban_tarjeta::Entity::find()
            .filter(kanban_tarjeta::Column::ColumnaId.is_in(column_ids))
            .filter(kanban_tarjeta::Column::IsDeleted.eq(false))
            .filter(kanban_tarjeta::Column::Archivada.eq(false))
            .order_by_asc(kanban_tarjeta::Column::Orden)
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;

        models.into_iter().map(tarjeta_to_domain).collect()
    }

    async fn list_by_columna(&self, columna_id: Uuid) -> AppResult<Vec<KanbanTarjeta>> {
        let models = kanban_tarjeta::Entity::find()
            .filter(kanban_tarjeta::Column::ColumnaId.eq(columna_id.to_string()))
            .filter(kanban_tarjeta::Column::IsDeleted.eq(false))
            .filter(kanban_tarjeta::Column::Archivada.eq(false))
            .order_by_asc(kanban_tarjeta::Column::Orden)
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;

        models.into_iter().map(tarjeta_to_domain).collect()
    }

    async fn find_by_trabajo_id(&self, trabajo_id: Uuid) -> AppResult<Option<KanbanTarjeta>> {
        let found = kanban_tarjeta::Entity::find()
            .filter(kanban_tarjeta::Column::TrabajoId.eq(Some(trabajo_id.to_string())))
            .filter(kanban_tarjeta::Column::IsDeleted.eq(false))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;
        found.map(tarjeta_to_domain).transpose()
    }

    async fn find_by_orden_trabajo_id(&self, orden_id: Uuid) -> AppResult<Option<KanbanTarjeta>> {
        let found = kanban_tarjeta::Entity::find()
            .filter(kanban_tarjeta::Column::OrdenTrabajoId.eq(Some(orden_id.to_string())))
            .filter(kanban_tarjeta::Column::IsDeleted.eq(false))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;
        found.map(tarjeta_to_domain).transpose()
    }

    async fn insert(&self, entity: &KanbanTarjeta) -> AppResult<()> {
        let active = tarjeta_to_active(entity);
        active
            .insert(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn update(&self, entity: &KanbanTarjeta) -> AppResult<()> {
        let active = tarjeta_to_active(entity);
        active
            .update(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn delete(&self, id: Uuid, row_version: &RowVersion) -> AppResult<()> {
        let current = kanban_tarjeta::Entity::find_by_id(id.to_string())
            .filter(kanban_tarjeta::Column::IsDeleted.eq(false))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?
            .ok_or_else(|| AppError::NotFound {
                entity: "kanban_tarjetas",
                id: id.to_string(),
            })?;

        if current.row_version != row_version.as_bytes() {
            return Err(AppError::Concurrency {
                entity: "kanban_tarjetas",
            });
        }

        let mut active: kanban_tarjeta::ActiveModel = current.into();
        active.is_deleted = Set(true);
        active.deleted_at = Set(Some(chrono::Utc::now().to_rfc3339()));
        active
            .update(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }
}
