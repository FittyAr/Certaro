use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "kanban_columnas")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub tablero_id: String,
    pub nombre: String,
    pub color: Option<String>,
    pub orden: i32,
    pub limite_wip: Option<i32>,
    pub estado_mapeado: Option<i32>,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub row_version: Vec<u8>,
    pub is_deleted: bool,
    pub deleted_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::kanban_tablero::Entity",
        from = "Column::TableroId",
        to = "super::kanban_tablero::Column::Id"
    )]
    Tablero,
    #[sea_orm(has_many = "super::kanban_tarjeta::Entity")]
    Tarjetas,
}

impl Related<super::kanban_tablero::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tablero.def()
    }
}

impl Related<super::kanban_tarjeta::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tarjetas.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
