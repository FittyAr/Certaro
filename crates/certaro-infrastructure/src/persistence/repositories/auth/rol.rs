use std::sync::Arc;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use certaro_application::ports::repositories::RolRepository;
use certaro_application::{AppError, AppResult};
use certaro_domain::entities::Rol;
use certaro_domain::RowVersion;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder, QuerySelect, RelationTrait};
use uuid::Uuid;
use crate::persistence::mappers::auth::{rol_to_active, rol_to_domain, usuario_rol_to_active};
use crate::persistence::models::{rol, usuario_rol};

pub struct SeaOrmRolRepository {
    tx: Arc<DatabaseTransaction>,
}

impl SeaOrmRolRepository {
    pub fn new(tx: Arc<DatabaseTransaction>) -> Self {
        Self { tx }
    }

    fn conn(&self) -> &DatabaseTransaction {
        self.tx.as_ref()
    }
}

#[async_trait]
impl RolRepository for SeaOrmRolRepository {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Rol>> {
        let found = rol::Entity::find_by_id(id.to_string())
            .filter(rol::Column::IsDeleted.eq(false))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;
        found.map(rol_to_domain).transpose()
    }

    async fn find_by_nombre(&self, nombre: &str) -> AppResult<Option<Rol>> {
        let found = rol::Entity::find()
            .filter(rol::Column::Nombre.eq(nombre))
            .filter(rol::Column::IsDeleted.eq(false))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;
        found.map(rol_to_domain).transpose()
    }

    async fn list_all(&self) -> AppResult<Vec<Rol>> {
        let rows = rol::Entity::find()
            .filter(rol::Column::IsDeleted.eq(false))
            .order_by_desc(rol::Column::Prioridad)
            .order_by_asc(rol::Column::Nombre)
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;
        rows.into_iter().map(rol_to_domain).collect()
    }

    async fn insert(&self, entity: &Rol) -> AppResult<()> {
        let active = rol_to_active(entity);
        rol::Entity::insert(active)
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn update(&self, entity: &Rol, esperado: RowVersion) -> AppResult<()> {
        let current = rol::Entity::find_by_id(entity.id.to_string())
            .filter(rol::Column::IsDeleted.eq(false))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?
            .ok_or_else(|| AppError::not_found("Rol", entity.id))?;

        if current.row_version != esperado.as_bytes() {
            return Err(AppError::Concurrency { entity: "Rol" });
        }

        let active = rol_to_active(entity);
        active.update(self.conn()).await.map_err(AppError::persistence)?;
        Ok(())
    }

    async fn soft_delete(&self, id: Uuid, esperado: RowVersion, at: DateTime<Utc>) -> AppResult<()> {
        let current = rol::Entity::find_by_id(id.to_string())
            .filter(rol::Column::IsDeleted.eq(false))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?
            .ok_or_else(|| AppError::not_found("Rol", id))?;

        if current.es_sistema {
            return Err(AppError::conflict("SYSTEM_ROLE", "Validation.Role.SystemRoleImmutable"));
        }

        if current.row_version != esperado.as_bytes() {
            return Err(AppError::Concurrency { entity: "Rol" });
        }

        let mut domain = rol_to_domain(current)?;
        domain.audit.soft_delete(at);
        let active = rol_to_active(&domain);
        active.update(self.conn()).await.map_err(AppError::persistence)?;
        Ok(())
    }

    async fn get_roles_for_usuario(&self, usuario_id: Uuid) -> AppResult<Vec<Rol>> {
        let roles = rol::Entity::find()
            .join(
                sea_orm::JoinType::InnerJoin,
                rol::Relation::UsuarioRol.def(),
            )
            .filter(usuario_rol::Column::UsuarioId.eq(usuario_id.to_string()))
            .filter(usuario_rol::Column::IsDeleted.eq(false))
            .filter(rol::Column::IsDeleted.eq(false))
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;
        roles.into_iter().map(rol_to_domain).collect()
    }

    async fn assign_rol_to_usuario(&self, usuario_id: Uuid, rol_id: Uuid, now: DateTime<Utc>) -> AppResult<()> {
        let exists = usuario_rol::Entity::find()
            .filter(usuario_rol::Column::UsuarioId.eq(usuario_id.to_string()))
            .filter(usuario_rol::Column::RolId.eq(rol_id.to_string()))
            .filter(usuario_rol::Column::IsDeleted.eq(false))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;

        if exists.is_some() {
            return Ok(());
        }

        let link = certaro_domain::entities::UsuarioRol {
            id: Uuid::now_v7(),
            usuario_id,
            rol_id,
            audit: certaro_domain::entities::Audit::new(now),
        };
        let active = usuario_rol_to_active(&link);
        usuario_rol::Entity::insert(active)
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn remove_rol_from_usuario(&self, usuario_id: Uuid, rol_id: Uuid) -> AppResult<()> {
        usuario_rol::Entity::delete_many()
            .filter(usuario_rol::Column::UsuarioId.eq(usuario_id.to_string()))
            .filter(usuario_rol::Column::RolId.eq(rol_id.to_string()))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }
}
