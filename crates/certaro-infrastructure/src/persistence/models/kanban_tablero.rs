use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "kanban_tableros")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub nombre: String,
    pub descripcion: Option<String>,
    pub color: Option<String>,
    pub es_preset: bool,
    pub tipo_preset: Option<String>,
    pub activo: bool,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub row_version: Vec<u8>,
    pub is_deleted: bool,
    pub deleted_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::kanban_columna::Entity")]
    Columnas,
}

impl Related<super::kanban_columna::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Columnas.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
