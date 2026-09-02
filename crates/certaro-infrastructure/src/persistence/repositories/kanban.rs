//! SeaORM repositories for the Kanban module.

use std::sync::Arc;
use async_trait::async_trait;
use certaro_application::ports::repositories::{
    KanbanChecklistRepository, KanbanColumnaRepository, KanbanEtiquetaRepository,
    KanbanTableroRepository, KanbanTarjetaRepository,
};
use certaro_application::{AppError, AppResult};
use certaro_domain::entities::kanban::{
    KanbanColumna, KanbanEtiqueta, KanbanTablero, KanbanTarjeta, KanbanTarjetaChecklist,
};
use certaro_domain::RowVersion;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use uuid::Uuid;

use crate::persistence::mappers::kanban::{
    checklist_to_active, checklist_to_domain, columna_to_active, columna_to_domain,
    etiqueta_to_active, etiqueta_to_domain, tablero_to_active, tablero_to_domain,
    tarjeta_to_active, tarjeta_to_domain,
};
use crate::persistence::models::{
    kanban_columna, kanban_etiqueta, kanban_tablero, kanban_tarjeta, kanban_tarjeta_checklist,
    kanban_tarjeta_etiqueta,
};

// =========================================================================
// SeaOrmKanbanTableroRepository
// =========================================================================

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
        let current = kanban_tablero::Entity::find_by_id(entity.id.to_string())
            .filter(kanban_tablero::Column::IsDeleted.eq(false))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?
            .ok_or_else(|| AppError::NotFound {
                entity: "kanban_tableros",
                id: entity.id.to_string(),
            })?;

        if current.row_version != entity.audit.row_version.as_bytes() {
            return Err(AppError::Concurrency {
                entity: "kanban_tableros",
            });
        }

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

// =========================================================================
// SeaOrmKanbanColumnaRepository
// =========================================================================

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
        let current = kanban_columna::Entity::find_by_id(entity.id.to_string())
            .filter(kanban_columna::Column::IsDeleted.eq(false))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?
            .ok_or_else(|| AppError::NotFound {
                entity: "kanban_columnas",
                id: entity.id.to_string(),
            })?;

        if current.row_version != entity.audit.row_version.as_bytes() {
            return Err(AppError::Concurrency {
                entity: "kanban_columnas",
            });
        }

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

// =========================================================================
// SeaOrmKanbanTarjetaRepository
// =========================================================================

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
        let current = kanban_tarjeta::Entity::find_by_id(entity.id.to_string())
            .filter(kanban_tarjeta::Column::IsDeleted.eq(false))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?
            .ok_or_else(|| AppError::NotFound {
                entity: "kanban_tarjetas",
                id: entity.id.to_string(),
            })?;

        if current.row_version != entity.audit.row_version.as_bytes() {
            return Err(AppError::Concurrency {
                entity: "kanban_tarjetas",
            });
        }

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

// =========================================================================
// SeaOrmKanbanEtiquetaRepository
// =========================================================================

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
        let current = kanban_etiqueta::Entity::find_by_id(entity.id.to_string())
            .filter(kanban_etiqueta::Column::IsDeleted.eq(false))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?
            .ok_or_else(|| AppError::NotFound {
                entity: "kanban_etiquetas",
                id: entity.id.to_string(),
            })?;

        if current.row_version != entity.audit.row_version.as_bytes() {
            return Err(AppError::Concurrency {
                entity: "kanban_etiquetas",
            });
        }

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

// =========================================================================
// SeaOrmKanbanChecklistRepository
// =========================================================================

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
