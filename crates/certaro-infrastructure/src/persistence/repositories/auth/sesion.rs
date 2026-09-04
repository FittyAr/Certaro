use std::sync::Arc;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use certaro_application::ports::repositories::SesionRepository;
use certaro_application::{AppError, AppResult};
use certaro_domain::entities::Sesion;
use certaro_domain::time;
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use uuid::Uuid;
use crate::persistence::mappers::auth::{sesion_to_active, sesion_to_domain};
use crate::persistence::models::sesion;

pub struct SeaOrmSesionRepository {
    tx: Arc<DatabaseTransaction>,
}

impl SeaOrmSesionRepository {
    pub fn new(tx: Arc<DatabaseTransaction>) -> Self {
        Self { tx }
    }

    fn conn(&self) -> &DatabaseTransaction {
        self.tx.as_ref()
    }
}

#[async_trait]
impl SesionRepository for SeaOrmSesionRepository {
    async fn find_by_token_hash(&self, token_hash: &str) -> AppResult<Option<Sesion>> {
        let found = sesion::Entity::find()
            .filter(sesion::Column::TokenHash.eq(token_hash))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;
        found.map(sesion_to_domain).transpose()
    }

    async fn insert(&self, entity: &Sesion) -> AppResult<()> {
        let active = sesion_to_active(entity);
        sesion::Entity::insert(active)
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn delete_by_token_hash(&self, token_hash: &str) -> AppResult<()> {
        sesion::Entity::delete_many()
            .filter(sesion::Column::TokenHash.eq(token_hash))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn delete_expired(&self, now: DateTime<Utc>) -> AppResult<u64> {
        let res = sesion::Entity::delete_many()
            .filter(sesion::Column::ExpiraEn.lt(time::to_storage(now)))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(res.rows_affected)
    }

    async fn delete_by_usuario(&self, usuario_id: Uuid) -> AppResult<()> {
        sesion::Entity::delete_many()
            .filter(sesion::Column::UsuarioId.eq(usuario_id.to_string()))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }
}
