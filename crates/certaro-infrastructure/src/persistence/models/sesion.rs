use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "sesiones")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub usuario_id: String,
    #[sea_orm(unique)]
    pub token_hash: String,
    pub expira_en: String,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: String,
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
