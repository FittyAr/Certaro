use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;
use certaro_domain::entities::*;
use certaro_domain::{Decimal4, EstadoFactura, EstadoProyecto, EstadoTrabajo, Moneda, Money, RowVersion};
use crate::paging::{PageRequest, PagedResult};
use crate::result::AppResult;
use super::common::*;

#[async_trait]
pub trait UsuarioRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Usuario>>;
    async fn find_by_email(&self, email: &str) -> AppResult<Option<Usuario>>;
    async fn list_all(&self) -> AppResult<Vec<Usuario>>;
    async fn insert(&self, entity: &Usuario) -> AppResult<()>;
    async fn update(&self, entity: &Usuario, esperado: RowVersion) -> AppResult<()>;
    async fn soft_delete(&self, id: Uuid, esperado: RowVersion, at: DateTime<Utc>) -> AppResult<()>;
}

#[async_trait]
pub trait RolRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Rol>>;
    async fn find_by_nombre(&self, nombre: &str) -> AppResult<Option<Rol>>;
    async fn list_all(&self) -> AppResult<Vec<Rol>>;
    async fn insert(&self, entity: &Rol) -> AppResult<()>;
    async fn update(&self, entity: &Rol, esperado: RowVersion) -> AppResult<()>;
    async fn soft_delete(&self, id: Uuid, esperado: RowVersion, at: DateTime<Utc>) -> AppResult<()>;
    async fn get_roles_for_usuario(&self, usuario_id: Uuid) -> AppResult<Vec<Rol>>;
    async fn assign_rol_to_usuario(&self, usuario_id: Uuid, rol_id: Uuid, now: DateTime<Utc>) -> AppResult<()>;
    async fn remove_rol_from_usuario(&self, usuario_id: Uuid, rol_id: Uuid) -> AppResult<()>;
}

#[async_trait]
pub trait PermisoRepository: Send + Sync {
    async fn list_all(&self) -> AppResult<Vec<Permiso>>;
    async fn find_by_clave(&self, clave: &str) -> AppResult<Option<Permiso>>;
    async fn insert(&self, entity: &Permiso) -> AppResult<()>;
    async fn get_permisos_for_rol(&self, rol_id: Uuid) -> AppResult<Vec<Permiso>>;
    async fn get_permisos_for_usuario(&self, usuario_id: Uuid) -> AppResult<Vec<Permiso>>;
    async fn assign_permiso_to_rol(&self, rol_id: Uuid, permiso_id: Uuid) -> AppResult<()>;
    async fn remove_permiso_from_rol(&self, rol_id: Uuid, permiso_id: Uuid) -> AppResult<()>;
}

#[async_trait]
pub trait SesionRepository: Send + Sync {
    async fn find_by_token_hash(&self, token_hash: &str) -> AppResult<Option<Sesion>>;
    async fn insert(&self, entity: &Sesion) -> AppResult<()>;
    async fn delete_by_token_hash(&self, token_hash: &str) -> AppResult<()>;
    async fn delete_expired(&self, now: DateTime<Utc>) -> AppResult<u64>;
    async fn delete_by_usuario(&self, usuario_id: Uuid) -> AppResult<()>;
}

#[async_trait]
pub trait AuthExternoRepository: Send + Sync {
    async fn find_by_proveedor_user_id(&self, proveedor: &str, proveedor_user_id: &str) -> AppResult<Option<AuthExterno>>;
    async fn list_by_usuario(&self, usuario_id: Uuid) -> AppResult<Vec<AuthExterno>>;
    async fn insert(&self, entity: &AuthExterno) -> AppResult<()>;
    async fn delete_by_id(&self, id: Uuid) -> AppResult<()>;
}

