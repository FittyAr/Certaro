use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "cliente_contactos")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub cliente_id: String,
    pub etiqueta: String,
    pub email: String,
    pub nombre: Option<String>,
    pub telefono: Option<String>,
    pub es_principal: bool,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub row_version: Vec<u8>,
    pub is_deleted: bool,
    pub deleted_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
