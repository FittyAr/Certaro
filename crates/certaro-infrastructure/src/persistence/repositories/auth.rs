//! Repositories for Auth, Users, Roles, Permissions, Sessions, and External SSO.

use std::sync::Arc;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use certaro_application::ports::repositories::{
    AuthExternoRepository, PermisoRepository, RolRepository, SesionRepository, UsuarioRepository,
};
use certaro_application::{AppError, AppResult};
use certaro_domain::entities::{AuthExterno, Permiso, Rol, Sesion, Usuario};
use certaro_domain::{time, RowVersion};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, RelationTrait,
};
use uuid::Uuid;

use crate::persistence::mappers::auth::{
    auth_externo_to_active, auth_externo_to_domain, permiso_to_active, permiso_to_domain,
    rol_permiso_to_active, rol_to_active, rol_to_domain, sesion_to_active, sesion_to_domain,
    usuario_rol_to_active, usuario_to_active, usuario_to_domain,
};
use crate::persistence::models::{
    auth_externo, permiso, rol, rol_permiso, sesion, usuario, usuario_rol,
};

// =========================================================================
// SeaOrmUsuarioRepository
// =========================================================================

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

// =========================================================================
// SeaOrmRolRepository
// =========================================================================

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

// =========================================================================
// SeaOrmPermisoRepository
// =========================================================================

pub struct SeaOrmPermisoRepository {
    tx: Arc<DatabaseTransaction>,
}

impl SeaOrmPermisoRepository {
    pub fn new(tx: Arc<DatabaseTransaction>) -> Self {
        Self { tx }
    }

    fn conn(&self) -> &DatabaseTransaction {
        self.tx.as_ref()
    }
}

#[async_trait]
impl PermisoRepository for SeaOrmPermisoRepository {
    async fn list_all(&self) -> AppResult<Vec<Permiso>> {
        let rows = permiso::Entity::find()
            .order_by_asc(permiso::Column::Modulo)
            .order_by_asc(permiso::Column::Accion)
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;
        rows.into_iter().map(permiso_to_domain).collect()
    }

    async fn find_by_clave(&self, clave: &str) -> AppResult<Option<Permiso>> {
        let found = permiso::Entity::find()
            .filter(permiso::Column::Clave.eq(clave))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;
        found.map(permiso_to_domain).transpose()
    }

    async fn insert(&self, entity: &Permiso) -> AppResult<()> {
        let active = permiso_to_active(entity);
        permiso::Entity::insert(active)
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn get_permisos_for_rol(&self, rol_id: Uuid) -> AppResult<Vec<Permiso>> {
        let rows = permiso::Entity::find()
            .join(
                sea_orm::JoinType::InnerJoin,
                permiso::Relation::RolPermiso.def(),
            )
            .filter(rol_permiso::Column::RolId.eq(rol_id.to_string()))
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;
        rows.into_iter().map(permiso_to_domain).collect()
    }

    async fn get_permisos_for_usuario(&self, usuario_id: Uuid) -> AppResult<Vec<Permiso>> {
        let user_roles = usuario_rol::Entity::find()
            .filter(usuario_rol::Column::UsuarioId.eq(usuario_id.to_string()))
            .filter(usuario_rol::Column::IsDeleted.eq(false))
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;

        let role_ids: Vec<String> = user_roles.into_iter().map(|ur| ur.rol_id).collect();
        if role_ids.is_empty() {
            return Ok(Vec::new());
        }

        let rows = permiso::Entity::find()
            .join(
                sea_orm::JoinType::InnerJoin,
                permiso::Relation::RolPermiso.def(),
            )
            .filter(rol_permiso::Column::RolId.is_in(role_ids))
            .distinct()
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;

        rows.into_iter().map(permiso_to_domain).collect()
    }

    async fn assign_permiso_to_rol(&self, rol_id: Uuid, permiso_id: Uuid) -> AppResult<()> {
        let exists = rol_permiso::Entity::find()
            .filter(rol_permiso::Column::RolId.eq(rol_id.to_string()))
            .filter(rol_permiso::Column::PermisoId.eq(permiso_id.to_string()))
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;

        if exists.is_some() {
            return Ok(());
        }

        let link = certaro_domain::entities::RolPermiso {
            id: Uuid::now_v7(),
            rol_id,
            permiso_id,
        };
        let active = rol_permiso_to_active(&link);
        rol_permiso::Entity::insert(active)
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn remove_permiso_from_rol(&self, rol_id: Uuid, permiso_id: Uuid) -> AppResult<()> {
        rol_permiso::Entity::delete_many()
            .filter(rol_permiso::Column::RolId.eq(rol_id.to_string()))
            .filter(rol_permiso::Column::PermisoId.eq(permiso_id.to_string()))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }
}

// =========================================================================
// SeaOrmSesionRepository
// =========================================================================

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

// =========================================================================
// SeaOrmAuthExternoRepository
// =========================================================================

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
