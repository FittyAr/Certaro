use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "kanban_tarjeta_checklist")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub tarjeta_id: String,
    pub titulo: String,
    pub completada: bool,
    pub orden: i32,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub row_version: Vec<u8>,
    pub is_deleted: bool,
    pub deleted_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::kanban_tarjeta::Entity",
        from = "Column::TarjetaId",
        to = "super::kanban_tarjeta::Column::Id"
    )]
    Tarjeta,
}

impl Related<super::kanban_tarjeta::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tarjeta.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
