use std::sync::Arc;
use async_trait::async_trait;
use certaro_application::ports::repositories::PermisoRepository;
use certaro_application::{AppError, AppResult};
use certaro_domain::entities::Permiso;
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder, QuerySelect, RelationTrait};
use uuid::Uuid;
use crate::persistence::mappers::auth::{permiso_to_active, permiso_to_domain, rol_permiso_to_active};
use crate::persistence::models::{permiso, rol_permiso, usuario_rol};

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
