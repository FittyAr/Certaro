use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "usuarios")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(unique)]
    pub email: String,
    pub nombre_completo: String,
    pub password_hash: Option<String>,
    pub activo: bool,
    pub requiere_2fa: bool,
    pub totp_secret: Option<String>,
    pub ultimo_login: Option<String>,
    pub intentos_fallidos: i32,
    pub bloqueado_hasta: Option<String>,
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
    #[sea_orm(has_many = "super::sesion::Entity")]
    Sesion,
    #[sea_orm(has_many = "super::auth_externo::Entity")]
    AuthExterno,
}

impl Related<super::usuario_rol::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UsuarioRol.def()
    }
}

impl Related<super::sesion::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sesion.def()
    }
}

impl Related<super::auth_externo::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AuthExterno.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
