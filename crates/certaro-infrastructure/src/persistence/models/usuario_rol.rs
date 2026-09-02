use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "usuario_roles")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub usuario_id: String,
    pub rol_id: String,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub row_version: Vec<u8>,
    pub is_deleted: bool,
    pub deleted_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::usuario::Entity",
        from = "Column::UsuarioId",
        to = "super::usuario::Column::Id"
    )]
    Usuario,
    #[sea_orm(
        belongs_to = "super::rol::Entity",
        from = "Column::RolId",
        to = "super::rol::Column::Id"
    )]
    Rol,
}

impl Related<super::usuario::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Usuario.def()
    }
}

impl Related<super::rol::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Rol.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
