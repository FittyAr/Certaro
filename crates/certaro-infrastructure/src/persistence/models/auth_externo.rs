use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "auth_externo")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub usuario_id: String,
    pub proveedor: String,
    pub proveedor_user_id: String,
    pub email: Option<String>,
    pub vinculado_en: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::usuario::Entity",
        from = "Column::UsuarioId",
        to = "super::usuario::Column::Id"
    )]
    Usuario,
}

impl Related<super::usuario::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Usuario.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
