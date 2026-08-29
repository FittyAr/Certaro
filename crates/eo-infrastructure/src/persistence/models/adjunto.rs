use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "adjuntos")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// Polymorphic reference with no foreign key; the accepted values are a domain constant.
    pub entidad_tipo: String,
    pub entidad_id: String,
    pub nombre_archivo: String,
    pub ruta_relativa: String,
    pub mime: String,
    /// Size in bytes, not scaled.
    pub tamano: i64,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub row_version: Vec<u8>,
    pub is_deleted: bool,
    pub deleted_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
