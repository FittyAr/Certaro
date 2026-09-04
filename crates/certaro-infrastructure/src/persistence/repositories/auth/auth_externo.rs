use std::sync::Arc;
use async_trait::async_trait;
use certaro_application::ports::repositories::AuthExternoRepository;
use certaro_application::{AppError, AppResult};
use certaro_domain::entities::AuthExterno;
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use uuid::Uuid;
use crate::persistence::mappers::auth::{auth_externo_to_active, auth_externo_to_domain};
use crate::persistence::models::auth_externo;

pub struct SeaOrmAuthExternoRepository {
    tx: Arc<DatabaseTransaction>,
}

impl SeaOrmAuthExternoRepository {
    pub fn new(tx: Arc<DatabaseTransaction>) -> Self {
        Self { tx }
    }

    fn conn(&self) -> &DatabaseTransaction {
        self.tx.as_ref()
    }
}

#[async_trait]
impl AuthExternoRepository for SeaOrmAuthExternoRepository {
    async fn find_by_proveedor_user_id(
        &self,
        proveedor: &str,
        proveedor_user_id: &str,
    ) -> AppResult<Option<AuthExterno>> {
        let found = auth_externo::Entity::find()
            .filter(auth_externo::Column::Proveedor.eq(proveedor))
            .filter(auth_externo::Column::ProveedorUserId.eq(proveedor_user_id))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;
        found.map(auth_externo_to_domain).transpose()
    }

    async fn list_by_usuario(&self, usuario_id: Uuid) -> AppResult<Vec<AuthExterno>> {
        let rows = auth_externo::Entity::find()
            .filter(auth_externo::Column::UsuarioId.eq(usuario_id.to_string()))
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;
        rows.into_iter().map(auth_externo_to_domain).collect()
    }

    async fn insert(&self, entity: &AuthExterno) -> AppResult<()> {
        let active = auth_externo_to_active(entity);
        auth_externo::Entity::insert(active)
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> AppResult<()> {
        auth_externo::Entity::delete_by_id(id.to_string())
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }
}
