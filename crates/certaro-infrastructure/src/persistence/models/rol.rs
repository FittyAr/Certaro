use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "roles")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(unique)]
    pub nombre: String,
    pub descripcion: Option<String>,
    pub es_sistema: bool,
    pub prioridad: i32,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub row_version: Vec<u8>,
    pub is_deleted: bool,
    pub deleted_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::usuario_rol::Entity")]
    UsuarioRol,
    #[sea_orm(has_many = "super::rol_permiso::Entity")]
    RolPermiso,
}

impl Related<super::usuario_rol::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UsuarioRol.def()
    }
}

impl Related<super::rol_permiso::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RolPermiso.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
