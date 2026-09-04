use std::sync::Arc;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use certaro_application::ports::repositories::UsuarioRepository;
use certaro_application::{AppError, AppResult};
use certaro_domain::entities::Usuario;
use certaro_domain::RowVersion;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder};
use uuid::Uuid;
use crate::persistence::mappers::auth::{usuario_to_active, usuario_to_domain};
use crate::persistence::models::usuario;

pub struct SeaOrmUsuarioRepository {
    tx: Arc<DatabaseTransaction>,
}

impl SeaOrmUsuarioRepository {
    pub fn new(tx: Arc<DatabaseTransaction>) -> Self {
        Self { tx }
    }

    fn conn(&self) -> &DatabaseTransaction {
        self.tx.as_ref()
    }
}

#[async_trait]
impl UsuarioRepository for SeaOrmUsuarioRepository {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Usuario>> {
        let found = usuario::Entity::find_by_id(id.to_string())
            .filter(usuario::Column::IsDeleted.eq(false))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;
        found.map(usuario_to_domain).transpose()
    }

    async fn find_by_email(&self, email: &str) -> AppResult<Option<Usuario>> {
        let found = usuario::Entity::find()
            .filter(usuario::Column::Email.eq(email))
            .filter(usuario::Column::IsDeleted.eq(false))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;
        found.map(usuario_to_domain).transpose()
    }

    async fn list_all(&self) -> AppResult<Vec<Usuario>> {
        let rows = usuario::Entity::find()
            .filter(usuario::Column::IsDeleted.eq(false))
            .order_by_asc(usuario::Column::NombreCompleto)
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;
        rows.into_iter().map(usuario_to_domain).collect()
    }

    async fn insert(&self, entity: &Usuario) -> AppResult<()> {
        let active = usuario_to_active(entity);
        usuario::Entity::insert(active)
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn update(&self, entity: &Usuario, esperado: RowVersion) -> AppResult<()> {
        let current = usuario::Entity::find_by_id(entity.id.to_string())
            .filter(usuario::Column::IsDeleted.eq(false))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?
            .ok_or_else(|| AppError::not_found("Usuario", entity.id))?;

        if current.row_version != esperado.as_bytes() {
            return Err(AppError::Concurrency { entity: "Usuario" });
        }

        let active = usuario_to_active(entity);
        active.update(self.conn()).await.map_err(AppError::persistence)?;
        Ok(())
    }

    async fn soft_delete(&self, id: Uuid, esperado: RowVersion, at: DateTime<Utc>) -> AppResult<()> {
        let current = usuario::Entity::find_by_id(id.to_string())
            .filter(usuario::Column::IsDeleted.eq(false))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?
            .ok_or_else(|| AppError::not_found("Usuario", id))?;

        if current.row_version != esperado.as_bytes() {
            return Err(AppError::Concurrency { entity: "Usuario" });
        }

        let mut domain = usuario_to_domain(current)?;
        domain.audit.soft_delete(at);
        let active = usuario_to_active(&domain);
        active.update(self.conn()).await.map_err(AppError::persistence)?;
        Ok(())
    }
}
