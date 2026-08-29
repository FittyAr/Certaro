use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "categorias")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub nombre: String,
    pub descripcion: Option<String>,
    pub color_hex: Option<String>,
    pub icono: Option<String>,
    pub categoria_padre_id: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub row_version: Vec<u8>,
    pub is_deleted: bool,
    pub deleted_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
